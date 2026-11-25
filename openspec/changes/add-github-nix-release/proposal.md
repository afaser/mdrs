## Why

`flake.nix` предоставляет только `devShells.x86_64-linux.default`, поэтому `nix profile add github:afaser/mdrs` не находит `packages.default`, а любая локальная установка требует сборки `iced`/`wgpu`. Нужна доставка готового бинаря через GitHub Releases: Nix скачивает релизный ELF и патчит его под библиотеки из `flake.lock`, Rust не компилируется. Внешний бинарный кеш (Cachix, Attic, кеш GitHub Actions) не используется: ассеты Releases не являются Nix-substituter'ом.

## What Changes

- `flake.nix`: общий набор runtime-библиотек для `devShell` и пакетов; `packages.x86_64-linux.source` (`rustPlatform.buildRustPackage` из `Cargo.lock`), `packages.x86_64-linux.default`/`mdrs` из `release.json`, bootstrap-фолбэк на `source` при отсутствии пина, `lib.mkReleasePackage`, `checks.source`/`checks.default`, `apps.default`.
- `nix/release-package.nix`: конструктор пакета из `{version, src}` — распакованный ELF, `autoPatchelfHook`, `runtimeDependencies` для dlopen-библиотек, `installCheckPhase`, общий для установки и предпубликационной проверки.
- `release.json`: версия, HTTPS-URL и SRI-хеш релизного архива `mdrs-x86_64-linux.tar.gz`; появляется после первого релиза и обновляется только release-workflow.
- `.github/workflows/ci.yml`: push в `trunk` и pull request — проверка `flake.lock` на `path`-входы, `nix flake check`, `nix build .#default`, проверка RPATH установленного бинаря.
- `.github/workflows/release.yml`: ручной запуск с версией — preflight, сборка `.#source`, детерминированный архив, локальная проверка через `lib.mkReleasePackage`, пин-коммит, неизменяемый тег `vX.Y.Z`, draft-релиз, публикация и досылка пина в ветку.
- `README.md`: установка `nix profile add github:afaser/mdrs` без компиляции, отличие `default` от `.#source`, пин тега, `nix profile upgrade --refresh`, системные требования (Vulkan ICD, GPU), процесс релиза и обязательный bump версии при обновлении `flake.lock`.
- `.omp/skills/release-mdrs/SKILL.md`: ритуал выпуска версии, проверки релиза и обновления установки без запуска GUI.

Код приложения (`src/`) не меняется: `--version`/`--help` не добавляются, install-check проверяет упаковку (существование ELF и наличие runtime-каталогов в RPATH), а версия гарантируется preflight'ом и тем, что архив собран из того же `.#source`.

## Capabilities

### New Capabilities

- `distribution`: Nix-пакет `mdrs` из исходников и из релизного ELF, установка из флейк-ссылки без компиляции Rust, CI-проверки упаковки, неизменяемая публикация ассетов и пин версии артефакта.

### Modified Capabilities

Нет. Поведение `mdrs` как программы не меняется.

## Impact

- `flake.nix`, `flake.lock` — новый `packages`/`checks`/`apps`, единый набор runtime-библиотек; вход nixpkgs остаётся `github:NixOS/nixpkgs/nixos-26.05` без `path`-ссылок.
- `nix/release-package.nix` — новый файл.
- `release.json` — новый файл в корне; после первого релиза определяет версию и содержимое `default`.
- `.github/workflows/ci.yml`, `.github/workflows/release.yml` — новые файлы; release требует `contents: write` и права пушить тег и пин-коммит.
- `README.md`, `.omp/skills/release-mdrs/SKILL.md` — документация доставки и ритуал релиза.
- Внешние зависимости: GitHub Releases как источник ELF и `cache.nixos.org` для библиотек; сторонние кеши и секреты не нужны.
- Ограничение: установка из `trunk` ставит последний релиз, а не текущие исходники; до первого релиза `default` собирается из `source`. Обновление `flake.lock` требует новой версии и нового релиза.
- Визуальный прогон в CI невозможен: сборка проверяет только компиляцию, тесты и патчинг ELF; рендер требует hardware GPU и остаётся локальной проверкой.
