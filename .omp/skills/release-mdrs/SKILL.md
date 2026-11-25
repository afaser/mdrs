---
name: release-mdrs
description: Выпустить новую версию mdrs и обновить установку из GitHub. Использовать, когда фикс или фича есть в исходниках, но отсутствует в установленной версии, либо когда нужно опубликовать релиз.
metadata:
  author: mdrs
  version: "1.0"
---

# Выпуск релиза mdrs

`flake.nix` отдаёт `packages.default` из `release.json`: это готовый ELF из
GitHub Release, а не сборка из исходников. Поэтому `nix profile add
github:afaser/mdrs` на любом коммите ставит версию из `release.json`.
Исходники попадают к пользователям только через новый релиз.

## 1. Диагностика

```sh
command -v mdrs
readlink -f "$(command -v mdrs)"   # store path и версия
jq -r .version release.json
nix profile list | grep -A4 -E 'Name:[[:space:]]+mdrs'
gh release list
git log --oneline -5
```

`mdrs` не имеет `--version`: версию показывают store path установленного
бинаря и `release.json`. Агент не запускает `mdrs`, `cargo run` и не открывает
окно: запуск требует display и hardware GPU. Визуальная проверка — только
`scripts/sandbox.sh` на машине с `/dev/dri`.

Признак «нужен релиз»: фикс есть в `trunk`, но `release.json` и store path
называют старую версию. Локальная сборка `.#source` проверяет код, но в PATH не
попадает и `release.json` не меняет.

## 2. Подготовка версии

- `Cargo.toml`: `version = "X.Y.Z"` без `v`. Версия обязана совпасть с input
  workflow, иначе preflight упадёт.
- Опубликованные тег и ассет неизменяемы: любое изменение артефакта — новая
  версия, перезапись запрещена.
- `direnv exec . cargo check -j 10` обновит `Cargo.lock` (версия пакета).
- Прогнать `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`;
  тяжёлые команды — `chrt -i 0 nice -n 19 ionice -c 3 direnv exec . …`.
- Коммит: `chore(release): bump version to X.Y.Z`.

Обновление `flake.lock` делается вместе с повышением версии: release-workflow
публикует бинарь из новых зависимостей, а `checks.default` на ветке проверяет
его до слияния. Пересборка того же номера версии под новый nixpkgs запрещена.

## 3. Push

Интерактивный SSH в агенте часто недоступен. Рабочий вариант — HTTPS с
токеном `gh`:

```sh
gh auth status
git -c credential.helper='!gh auth git-credential' \
  push https://github.com/afaser/mdrs.git trunk
```

Push по URL не обновляет tracking-ref; чтобы `git status` не показывал
«ahead», после push выполнить:

```sh
git -c credential.helper='!gh auth git-credential' \
  fetch https://github.com/afaser/mdrs.git trunk:refs/remotes/origin/trunk
```

## 4. Запуск Release

```sh
gh workflow run release.yml --ref trunk -f version=X.Y.Z
gh run list --workflow=release.yml --limit 3     # взять id запуска
gh run watch <id> --exit-status --interval 15
```

Холодная сборка занимает десятки минут: не поллить запуск вручную и не
запускать второй Release параллельно. Workflow проверяет `version ==
Cargo.toml`, собирает `.#source`, публикует неизменяемый релиз `vX.Y.Z` с
ассетом `mdrs-x86_64-linux.tar.gz`, коммитит `release vX.Y.Z` с новым
`release.json` (url и hash) и только после публикации отправляет пин в ветку.

## 5. Проверка релиза и пина

```sh
gh release view vX.Y.Z --json tagName,isDraft,assets
git -c credential.helper='!gh auth git-credential' \
  fetch https://github.com/afaser/mdrs.git trunk
git merge --ff-only FETCH_HEAD
git show HEAD:release.json        # version/url/hash = vX.Y.Z
nix build github:afaser/mdrs/vX.Y.Z#default
readelf -d result/bin/mdrs | grep RUNPATH   # vulkan-loader и wayland
```

Если релиз опубликован, но пин не попал в ветку (push отклонён), повторный
запуск той же версии не собирает и не загружает ничего: он проверяет
существующий пин-коммит и делает только fast-forward push. Force не
используется; если ветка ушла вперёд, нужна новая версия.

## 6. Обновление установки

```sh
nix profile upgrade --refresh mdrs
readlink -f "$(command -v mdrs)"   # …-mdrs-X.Y.Z/…
```

`--refresh` обязателен: без него Nix оставляет прежний залоченный ревизион и
пишет `upgrading … to …` с тем же коммитом, а store path не меняется.

## Ограничения

- Релиз выполняется только из ветки; тег и ассет опубликованного релиза не
  перезаписываются и не переносятся.
- Коммит пина может не пройти при branch protection или без `contents:
  write`; тогда workflow падает на push, а тег нужно разбирать по его логу.
- Если после публикации нашлась ошибка в артефакте — bump версии и новый
  релиз, а не перезапись тега.
