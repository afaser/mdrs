## 1. Флейк и пакеты

- [x] 1.1 Вынести общий список runtime-библиотек в `let` и использовать его в `devShells.x86_64-linux.default`; проверить `nix eval --raw .#devShells.x86_64-linux.default.drvPath` и `direnv exec . cargo test` — оба проходят без изменения локального запуска
- [x] 1.2 Добавить `packages.x86_64-linux.source`: `rustPlatform.buildRustPackage`, `src = self`, `cargoLock.lockFile`, версия из `Cargo.toml`, `autoPatchelfHook`, `patchelf`, `buildInputs = [ stdenv.cc.cc.lib ]`, `runtimeDependencies` из общего набора, `installCheckPhase` с `test -x $out/bin/mdrs`; проверить `nix build .#source`, наличие `result/bin/mdrs`, `nix eval --raw .#source.version` и `readelf -d result/bin/mdrs | grep RUNPATH` с `vulkan-loader` и `wayland`
- [x] 1.3 Добавить `nix/release-package.nix` с конструктором `{version, src}` (`dontBuild = true`, установка ELF в `$out/bin/mdrs`, тот же `autoPatchelfHook` и `runtimeDependencies`, `test -x`) и `lib.mkReleasePackage`; `default`/`mdrs` строятся через `fetchurl` из `release.json`, при отсутствии файла указывают на `source`; проверить до релиза `nix build .#default` и `nix eval --raw .#default.version`
- [x] 1.4 Добавить `checks.x86_64-linux.source`, `checks.x86_64-linux.default` и `apps.x86_64-linux.default`; проверить `nix flake check` на чистом клоне без `release.json`
- [x] 1.5 Проверить путь предпубликационной проверки: собрать `.#source`, сделать `mdrs-x86_64-linux.tar.gz` с `mdrs` в корне, выполнить `nix build --impure --expr 'let f = builtins.getFlake (toString ./.); in f.lib.mkReleasePackage { version = "0.1.0"; src = /путь/mdrs-x86_64-linux.tar.gz; }'`; убедиться, что `result/bin/mdrs` исполняется и `readelf -d` содержит `vulkan-loader` и `wayland`
- [x] 1.6 Проверить переносимость входа: `jq -e '[.. | objects | select(.type? == "path")] | length == 0' flake.lock` равно `true`, а `nix flake metadata . --json` резолвит nixpkgs как `github`

## 2. CI

- [ ] 2.1 Создать `.github/workflows/ci.yml` с триггерами `push` в `trunk` и `pull_request`, `concurrency` по ref и `permissions: contents: read`; шаги: checkout, установка Nix, проверка `flake.lock` на `"type": "path"`, `nix flake check --print-build-logs`, `nix build .#default --print-build-logs`, проверка `readelf -d result/bin/mdrs | grep RUNPATH` на `vulkan-loader` и `wayland`; проверить зелёный прогон на тестовом pull request без секретов
- [ ] 2.2 Проверить провал: ветка с некомпилируемым `src/` даёт красный workflow на сборке `source`, ветка с неверным `hash` в `release.json` — красный на сборке `default`; оба изменения откатить

## 3. Релиз и установка

- [ ] 3.1 Создать `.github/workflows/release.yml` на `workflow_dispatch` с входом версии и `permissions: contents: write`: проверка semver и равенства `Cargo.toml`, отказ при опубликованном `vX` (кроме режима доведения пина), сборка `.#source`, детерминированный архив `mdrs-x86_64-linux.tar.gz`, SRI-хеш, локальная проверка через `lib.mkReleasePackage` (`test -x`, RUNPATH), запись `release.json`, пин-коммит, тег `vX`, push только тега, draft с `--verify-tag`, загрузка, публикация через `gh release edit vX --draft=false`, проверка публичного URL и `nix build .#default` из пин-коммита, fast-forward push пина; проверить отказ до сборки при версии `9.9.9` и версии `0.1.0` в `Cargo.toml`
- [ ] 3.2 Реализовать в workflow ошибки и повтор: до публикации удалять только созданные этим запуском draft и тег; после публикации ничего не удалять и не пушить при ошибке; повторный запуск опубликованной версии только fast-forward отправляет существующий пин-коммит, при ушедшей ветке падает без force; добавить `concurrency` без отмены; проверить все ветки на тестовом репозитории/ветке
- [ ] 3.3 Запустить release-workflow с `0.1.0` на `trunk`; проверить `gh release view v0.1.0` (не draft, ассет на месте), `release.json` в `trunk` с `version/url/hash` от `v0.1.0`, тег указывает на пин-коммит, force-операций в логе нет
- [ ] 3.4 На чистом профиле установить `github:afaser/mdrs` и `github:afaser/mdrs/v0.1.0`: в логе есть загрузка ассета и `autoPatchelf`, нет `cargo`/`rustc`; `readlink -f "$(command -v mdrs)"` содержит `0.1.0`, `readelf -d` установленного ELF содержит `vulkan-loader` и `wayland`
- [ ] 3.5 Проверить повторный запуск `0.1.0` после изменения `src/`: workflow отказывает без перезаписи ассета и без переноса тега
- [ ] 3.6 Проверить режим обновления зависимостей на следующей версии: в ветке поднять `Cargo.toml` и `flake.lock`, выпустить `0.1.1` с этой ветки, убедиться, что `checks.default` зелёный до слияния, тег и ассет не перезаписываются

## 4. Документация

- [ ] 4.1 Обновить `README.md`: установка из `trunk` и тега без кеша и компиляции Rust, отличие `default` от `.#source`, версия артефакта, `nix profile upgrade --refresh`, системный Vulkan ICD и hardware GPU, процесс релиза, обязательный bump версии при обновлении nixpkgs, предусловия прав `GITHUB_TOKEN`/защиты ветки; проверить команды и имена тегов/ассетов
- [ ] 4.2 Добавить `.omp/skills/release-mdrs/SKILL.md`: диагностика по store path и `release.json`, проверка релиза через `nix build .#default` и RUNPATH, запрет запуска GUI агентом, push через HTTPS и `gh`, `upgrade --refresh`, холодная сборка десятки минут; сверить шаги с фактическим прогоном релиза

## 5. Интеграционная проверка

- [ ] 5.1 На чистом клоне `trunk` выполнить `nix flake check`, `nix build .#default` и установку из профиля; убедиться, что версия и store path совпадают с релизом, а в логе установки нет `cargo`/`rustc`
- [ ] 5.2 После рефакторинга `flake.nix` прогнать `scripts/sandbox.sh <файл.md>` на машине с `/dev/dri`; убедиться, что окно рендерится и скриншот пишется в `target/screenshot.png`, то есть общий runtime-набор не сломал devShell
