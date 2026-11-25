## Context

См. `proposal.md` — Why. Текущее состояние и ограничения, которые определяют подход:

- `flake.nix` содержит только `devShells.x86_64-linux.default`; `packages`, `checks`, `apps` отсутствуют. `flake.lock` уже ссылается на `github:NixOS/nixpkgs/nixos-26.05`, `path`-входов нет.
- Репозиторий публичный, default-ветка `trunk`, GitHub Actions включены, релизов ещё нет. Push по SSH из агента отбивается (`Permission denied`); рабочий путь — HTTPS с `gh auth git-credential`.
- Ассеты (`assets/fonts`, svg) вкомпилированы через `include_bytes!`: артефакт — один ELF `mdrs`.
- `readelf -d` установленного бинаря: `NEEDED` только `libc`, `libm`, `libgcc_s`; GUI-библиотеки грузятся через `dlopen` (`libvulkan`, `libwayland-client`, `libwayland-egl`, `libxkbcommon`, `libX11`, `libxcb`, `libXcursor`, `libXrandr`, `libXi`, `libXinerama`, `libdbus-1`). Секции `.note.dlopen` нет.
- `mdrs` не имеет `--version`/`--help`: первый аргумент трактуется как путь к файлу, любой запуск открывает окно и требует hardware GPU. CI не может запускать приложение.
- Пробная сборка `rustPlatform.buildRustPackage` из текущего дерева прошла: 214 тестов, `autoPatchelfHook` записал в RPATH каталоги `dbus`, `vulkan-loader`, `wayland`, `libxkbcommon`, `libx11`, `libxcb`, `libxcursor`, `libxrandr`, `libxi`, `libxinerama`, `libgcc`; «0 dependencies could not be satisfied». Холодная сборка — около 21 минуты на 10 ядрах.

Требования — `specs/distribution/spec.md`; мотивация — `proposal.md`.

## Goals / Non-Goals

**Goals:**

- `default`/`mdrs` ставят ELF из GitHub Release; в установке нет `cargo`/`rustc`.
- `source` собирает текущие исходники с тестами для CI, release-workflow и разработки.
- Единый набор runtime-библиотек для `devShell`, `source` и релизного пакета; dlopen-библиотеки находятся через RPATH.
- Один номер версии — один неизменяемый релиз: без перезаписи ассетов и без переноса тегов.
- Релиз проверяется до публикации и доводится до ветки только после публикации.
- Ритуал релиза зафиксирован скиллом `.omp/skills/release-mdrs/SKILL.md`.

**Non-Goals:**

- Бинарный кеш, Cachix, Attic, self-hosted substituter и кеш CI: установка идёт только через `fetchurl` релизного ассета.
- Добавление `--version`/`--help` и любые правки `src/`.
- Статическая сборка, AppImage, `nix bundle`, кросс-сборка, aarch64, musl.
- Прогон GPU/визуального сценария в CI.
- Обновление `flake.lock` в рамках этого изменения.
- Упаковка системного Vulkan ICD и драйвера GPU.

## Decisions

**Релизный ассет вместо кеша.** `default` — `fetchurl` архива из `release.json` плюс конструктор `nix/release-package.nix`: `stdenv.mkDerivation`, `dontBuild = true`, `autoPatchelfHook`, `runtimeDependencies`, `installCheckPhase`. Альтернативы (Cachix/Attic, кеш GitHub Actions, tar в git) отклонены: внешний сервис с секретом, нерешаемая подстановка store-путей или рост репозитория. Цена — холодная сборка CI около 20+ минут; кеш не вводится.

**Один конструктор для установки и проверки.** `nix/release-package.nix` принимает `{version, src}`, где `src` — уже скачанный или локальный архив; `flake.nix` передаёт `fetchurl`, release-workflow — путь к локальному `dist/`. `default` и предпубликационная проверка используют один и тот же набор `buildInputs`, `runtimeDependencies` и install-check. Иначе workflow доказывал бы не тот пакет, который ставит пользователь. Конструктор доступен как `lib.mkReleasePackage` для `nix build --impure --expr`.

**RPATH вместо обёртки.** `runtimeDependencies` покрывает dlopen-библиотеки: `auto-patchelf` добавляет их каталоги в `DT_RUNPATH` (проверено на копии бинаря: vulkan-loader и wayland попали в RUNPATH, интерпретатор переписан, недостающих зависимостей нет). Обёртка с `LD_LIBRARY_PATH` отклонена: она меняет окружение процесса и не нужна для dlopen из главного ELF. `autoPatchelfHook` патчит `$out/bin/mdrs` в `postFixup`; `installCheckPhase` поэтому проверяет только `test -x $out/bin/mdrs`, а RPATH проверяется снаружи после сборки (`readelf -d result/bin/mdrs | grep RUNPATH`).

**Набор runtime-библиотек не меняется.** Пакеты используют тот же список, что текущий `devShell`: `dbus`, `vulkan-loader`, `wayland`, `libxkbcommon`, `libx11`, `libxcb`, `libxcursor`, `libxrandr`, `libxi`, `libxinerama`. `libglvnd`/`mesa` не добавляются: рендер идёт через Vulkan, программный adapter — ошибка, а GL/EGL-библиотеки остаются системными. Это сохраняет проверенное локальное поведение и не подменяет драйвер.

**Артефакт — несобранный ELF.** Архив `mdrs-x86_64-linux.tar.gz` содержит файл `mdrs` в корне. `source` ставит его в `$out/bin/mdrs` через `cargoInstallHook`; релизный конструктор распаковывает архив и кладёт ELF в `$out/bin/mdrs`. Store-пути CI-замыкания в ассет не попадают, повторной обёртки нет.

**Версия принадлежит артефакту.** `source.version` — из `Cargo.toml`; `default.version` — из `release.json`. После повышения `Cargo.toml` до следующей версии `default` намеренно отдаёт предыдущий бинарь и предыдущую версию. Совпадение обязательно на релизном коммите, и release-workflow это проверяет. Bootstrap: пока `release.json` нет, `default` указывает на `source`.

**Install-check без запуска приложения.** Приложение нельзя запустить без display и GPU, а `--version` нет. Проверка ограничена упаковкой: `test -x $out/bin/mdrs` в пакете, `readelf -d result/bin/mdrs` с `vulkan-loader` и `wayland` в RUNPATH в CI и release-workflow, плюс preflight-сверка версии. Добавление CLI отклонено как выходящее за рамки доставки; провенанс обеспечивается тем, что архив собран из того же `.#source` и пин-хеш лежит в `release.json`.

**Неизменяемый релиз.** Имя ассета фиксировано, потому что версия уже уникальна. `--clobber`, `git push --force`, повторная загрузка и перенос тега запрещены. Последовательность первого релиза:

1. Проверить semver, ветку запуска, равенство версии в `Cargo.toml`, отсутствие опубликованного `vX`.
2. Собрать `.#source` (тесты внутри сборки).
3. Создать детерминированный архив ELF (`tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner`, `gzip -n`) и вычислить SRI-хеш.
4. Собрать бинарный пакет из локального архива через `lib.mkReleasePackage`, проверить `test -x` и RUNPATH. Публичный URL ещё не нужен.
5. Записать `release.json`, создать локальный пин-коммит и тег `vX`.
6. Отправить только тег; ветка продолжает работать по прежнему релизу.
7. Создать draft `vX` с `--verify-tag`, загрузить ассет, опубликовать (`gh release edit vX --draft=false`).
8. Проверить скачивание по публичному URL и `nix build .#default` из пин-коммита.
9. Отправить пин-коммит в ветку запуска fast-forward.

Draft создаётся после тега и публикуется после загрузки. Ошибка до шага 7 удаляет draft и тег, созданные этим запуском; ветка не тронута. Ошибка после публикации не удаляет релиз и не отправляет пин: опубликованный тег неизменяем, несовместимая версия исправляется только новой версией.

**Повторный запуск.** Если `vX` уже опубликован, workflow не собирает и не загружает ничего. Если пин-коммит является текущей вершиной ветки запуска — успех. Если ветка ушла вперёд, запуск падает, требуя новую версию: опубликованный тег не перезаписывается. Если пин ещё не в ветке, workflow проверяет существующий пин-коммит и пытается fast-forward отправить его; при ушедшей ветке падает без force. Если существует только draft (прерванный запуск), draft и тег удаляются и релиз начинается заново. Одновременные запуски сериализуются группой `concurrency` без отмены.

**Обновление nixpkgs — новая версия.** Обновление `flake.lock`, меняющее зависимости бинаря, делается вместе с повышением версии в `Cargo.toml`. Release-workflow запускается на ветке обновления, публикует новую версию и отправляет пин в эту ветку; тогда `checks.default` проверяет новый ассет до слияния. Пересборка того же номера версии под новый nixpkgs не предусмотрена.

**CI без секретов.** `ubuntu-latest`: checkout, установка Nix через `cachix/install-nix-action` (только установщик), проверка `jq -e '[.. | objects | select(.type? == "path")] | length == 0' flake.lock`, `nix flake check --print-build-logs`, `nix build .#default --print-build-logs`, проверка RUNPATH через `readelf`. Форк-PR не требует секретов; сеть нужна для nixpkgs и ассета. `checks.source` и `checks.default` дают обе сборки внутри `flake check`.

**Скилл релиза.** `.omp/skills/release-mdrs/SKILL.md` повторяет ритуал thepaper с поправками: диагностика по store path и `release.json` вместо `--version`; проверка релиза через `nix build .#default` и RUNPATH; агент не запускает GUI; push через HTTPS и `gh`; `nix profile upgrade --refresh mdrs`; холодная сборка — десятки минут.

## Risks / Trade-offs

- [Ошибка после публикации требует новой версии] → принято ради неизменяемости; локальная проверка до публикации снижает вероятность.
- [Обновление nixpkgs требует версии и релиза до слияния PR] → явный процесс; иначе `checks.default` красный, а установка на trunk могла бы остаться с несовместимым бинарём.
- [`default` отстаёт от `Cargo.toml`] → README различает `.#default` (последний релиз) и `.#source` (исходники); release-workflow сверяет версию.
- [Установка требует сеть: ассет и библиотеки из `cache.nixos.org`] → следствие отказа от кеша; офлайн-установка не поддерживается.
- [autoPatchelf не проверяет версии glibc-символов] → ассет и установка используют один `flake.lock`; после смены lock нужна новая версия и проверка `checks.default`.
- [Vulkan ICD и драйвер остаются системными] → RPATH даёт loader; на NixOS он ищет `/run/opengl-driver`, на другой ОС — системный ICD. README фиксирует требование hardware GPU.
- [CI не запускает приложение] → проверка ограничена компиляцией, тестами и патчингом ELF; визуальный прогон остаётся локальным (`scripts/sandbox.sh`).
- [Холодная сборка CI ~20+ минут] → принято; кеш не вводится, сокращение — отдельным изменением.
- [Push тега и пина требует прав] → `permissions: contents: write` и отсутствие блокирующей branch protection; это предусловие README, а не обход защиты.
- [Первый `nix profile add` до релиза компилирует Rust] → bootstrap-фолбэк; после `v0.1.0` и пин-коммита установка без компиляции.

## Migration Plan

1. Переписать `flake.nix`: общий `runtime`, `packages.source`, bootstrap-`default`, `lib.mkReleasePackage`, `checks`, `apps`; добавить `nix/release-package.nix`; локально `nix build .#source`, `nix flake check`.
2. Добавить CI и получить зелёный прогон на bootstrap-`default`.
3. Добавить release-workflow; выпустить `0.1.0` на `trunk`; проверить `release.json`, тег, ассет, установку из `trunk` и с тега, `upgrade --refresh`.
4. Обновить README и добавить скилл `release-mdrs`; проверить команды и имена.
5. Обновление nixpkgs проверять на следующей версии.

Откат — revert коммитов флейка и workflow. Опубликованные релизы и теги не трогаются; удаление `release.json` возвращает bootstrap-сборку.

## Open Questions

Нет.
