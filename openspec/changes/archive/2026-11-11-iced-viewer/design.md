## Context

Репозиторий пустой: `src/main.rs` с `println!`, ни зависимостей, ни коммитов. Rust 1.96, edition 2024. Машина: NixOS 26.05, сессия Wayland, AMD Cezanne, Mesa 26.0.2, Vulkan-устройства RADV (аппаратное) и llvmpipe (программное), Mesa shader cache прогрет.

Замеры, на которые опирается дизайн:

| Операция | Тёплый кеш | Без Mesa shader cache |
|---|---|---|
| `vulkaninfo` (instance + device) | 65 мс | 1.27 с |
| `vkcube`: окно + swapchain + 1 кадр | 47–64 мс | 1.20 с |

Ограничения платформы: `winit` 0.30 не доставляет `DroppedFile` на Wayland. Это не особенность iced: то же у eframe/egui и любого winit-стека. На X11/XWayland перетаскивание работает.

Ограничения репозитория: `AGENTS.md` требует модульную раскладку, отсутствие комментариев, отсутствие логики в `main.rs`, `Option` вместо паники, один концепт на файл. Правило «только стандартная библиотека» несовместимо с iced; его переформулировка — отдельное решение, в изменение не входит.

Спецификация: `specs/markdown-viewer/spec.md`.

## Goals / Non-Goals

**Goals:**

- Нативный бинарь `mdrs` на iced 0.14 с GPU-рендером через wgpu/Vulkan.
- Тёплый старт до первого кадра не более 150 мс.
- GFM-рендер, тема Catppuccin Mocha Green, встроенный JetBrains Mono Nerd Font.
- Live reload с сохранением позиции прокрутки.
- Открытие аргументом, диалогом и перетаскиванием (последнее — X11/XWayland).

**Non-Goals:**

- tree-sitter, mermaid/PlantUML, TOC, поиск, вкладки, редактирование, экспорт.
- Windows/macOS; первый срез — Linux/NixOS.
- JS/WASM/вебвью; WASM-таргет iced не используется.
- Собственный виджет-фреймворк: только `Viewer` и существующие виджеты.

## Decisions

### 1. iced 0.14, а не egui

Выбран iced: встроенный markdown-виджет с `Item::{Heading, Paragraph, CodeBlock, List, Image, Quote, Rule, Table}`, `Viewer` trait для переопределения любого элемента и `selectable(true)` у текста. Это даёт типографику и точку расширения под tree-sitter/mermaid без форка. Цена — Vulkan-инициализация вместо OpenGL и новизна markdown-виджета.

Альтернатива egui+glow дешевле по старту, но требует `egui_commonmark` с фиксированными стилями и форка для будущих расширений. Остаётся планом отступления, если спайк не уложится в 150 мс.

Пин: `iced = "0.14"` с crates.io. Ветка `master` — 0.15-dev с git-зависимостями (`winit`, `cosmic-text` форки); её не берём.

### 2. Только wgpu, без tiny-skia

`default-features = false`, включаются `wgpu`, `x11`, `wayland`, `wayland-dlopen`, `wayland-decorations`, `markdown`, `highlighter`, `image`. Отключены `tiny-skia`, `linux-theme-detection`, `unconditional-rendering`, `debug`, `fira-sans`, `web-colors`.

`tiny-skia` — программный рендер всего кадра, исключён требованием GPU. `linux-theme-detection` читает системную тему на старте и не нужен: тема фиксирована.

### 3. Аппаратный адаптер

wgpu должен выбрать RADV. В системе два устройства, и llvmpipe — валидный Vulkan-адаптер, поэтому выбор проверяется явно: лог выбранного адаптера на старте, при `llvmpipe` — экран ошибки, а не тихий CPU-рендер. Если iced не даёт управлять выбором, используется переменная окружения wgpu-util (`WGPU_BACKEND`, `WGPU_ADAPTER_NAME`) или точечный патч; это проверяется спайком.

### 4. Шрифт

`JetBrainsMonoNerdFont-Regular.ttf` встраивается `include_bytes!` и регистрируется через `Application::font` + `default_font(Font::with_name(...))`. Системный fontconfig для выбора шрифта не используется.

Ограничение: iced/cosmic-text строит `FontSystem` и сканирует системные шрифты независимо от встроенного. Публичного выключателя нет; стоимость измеряется. Если скан съедает бюджет старта, это аргумент вернуться к egui, а не патчить cosmic-text.

### 5. Рендер markdown

`document::load` → `Source` → `markdown::resolve` → `Article(Vec<markdown::Item>)`. `resolve` переписывает относительные URI изображений в абсолютные `file://` через offset-парсер pulldown-cmark; всё остальное отдаётся `markdown::parse` без изменений.

Рендер — через `markdown::view_with` и собственный `theme::Viewer`:

- `heading` — зелёный `#a6e3a1`, размер по уровню;
- `paragraph`, `lists`, `quote`, `rule`, `table` — дефолтные виджеты с `Settings`;
- `code_block` — собственная подсветка `two-face`/syntect с темой Catppuccin Mocha (встроенный `iced_highlighter::Theme` знает только 5 тем, Catppuccin среди них нет);
- `image` — `iced::widget::image`, URI уже абсолютный;
- `on_link_click` → `Message::Link(uri)`.

Подсветка блока кода кэшируется по `(language, hash(code))`: результат строится один раз на документ, а не каждый кадр. Набор синтаксисов `two-face` загружается лениво при первом блоке кода; если он попадает в первый кадр, стоимость измеряется.

### 6. Открытие

Аргумент — `std::env::args_os().nth(1)` до запуска iced. Диалог — `rfd` 0.18 (XDG-portal, без GTK) в отдельном потоке, результат приходит через канал и `Subscription`. Перетаскивание — `iced::event::listen_with` по `window::Event::FileDropped`; на Wayland событие не приходит, приложение остаётся работоспособным.

### 7. Live reload и позиция

`notify` 8.2 следит за каталогом файла (атомарное сохранение заменяет inode), дебаунс 100 мс, событие уходит в iced `Subscription`. При чтении после события: успех — пересборка `Article`; ошибка — сохраняется предыдущее содержимое.

Позиция: `scrollable` хранит состояние в дереве виджетов, и при замене содержимого внутри того же `scrollable` offset должен сохраниться. Это проверяется спайком. Если сбросится — запасной вариант: подписка на `scrollable::on_scroll` запоминает `AbsoluteOffset`, после пересборки отправляется `scrollable::scroll_to`.

### 8. Тема

`Theme::custom("Catppuccin Mocha Green", Palette { background: #1e1e2e, text: #cdd6f4, primary: #a6e3a1, danger: #f38ba8, success: #a6e3a1 })`; палитра дублируется константами в `theme::palette` для `Viewer` и для syntect-темы. Код — Catppuccin Mocha из `two-face` (frappe/latte/macchiato/mocha присутствуют в наборе `two_face::theme::extra()`).

### 9. Замер старта

`Instant::now()` первой строкой `main`. При `MDRS_STARTUP=1` первый отрисованный кадр печатает в stderr время от старта процесса и завершает приложение — это диагностический режим для `hyperfine`, не пользовательский флаг. Отдельно измеряется `vkcube` как нижняя граница Vulkan-инициализации.

### 10. Раскладка

```
src/main.rs          fn main() { mdrs::run() }
src/lib.rs           pub fn run()
src/app/             Program, Message, update, view
src/document/        Document { path, source }, load
src/markdown/        resolve, Article
src/theme/           palette, theme, viewer, highlight
src/open/            args, dialog, drop
src/watch/           FileWatcher, debounce
```

`mod.rs` — только индекс (`mod` + `pub use`, алфавитно). Логика — в файлах концептов. Тесты — внизу своего файла.

### 11. Тесты и проверки

Модульные: загрузка несуществующего/недоступного файла; переписывание относительных и абсолютных URI изображений; сборка `Article` на документе со всеми `Item`; дебаунс watcher на временном каталоге.

Проверки вручную/спайком: warm-старт, выбор RADV, сохранение offset при reload, отрисовка каждого `Item`, X11-перетаскивание.

## Risks / Trade-offs

- **iced 0.14 экспериментальный, markdown-виджет новый** → пин версии, проверка всех `Item` вручную; при провале — egui+glow.
- **cosmic-text сканирует системные шрифты** → измерить; выключателя нет; при выходе за бюджет — смена фреймворка.
- **wgpu может выбрать llvmpipe** → логировать адаптер, падать с понятным сообщением, форсить RADV.
- **Wayland не доставляет файлы** → задокументировано; drop работает в X11/XWayland; спека это учитывает.
- **Холодный shader cache ~1.2 с** → платформенное поведение Mesa, приложением не лечится.
- **`iced_highlighter` не умеет Catppuccin** → свой `code_block` на `two-face`; при отсутствии темы в 0.4 — встроить `.tmTheme` строкой.
- **Загрузка `two-face`/syntect на первом блоке кода** → ленивая, кэш по блоку; при необходимости — предзагрузка в фоне после первого кадра.
- **`AGENTS.md` требует std-only** → правило будет переформулировано отдельно; код при этом соблюдает остальные правила документа.

## Migration Plan

Пустой репозиторий: заглушка `src/main.rs` заменяется, `Cargo.toml` получает зависимости, данных для миграции нет. Откат — `git revert` коммита; внешнего состояния приложение не пишет (только читает документ и кеширует в памяти).

## Open Questions

- Умеет ли iced 0.14 форсить адаптер без патча?
- Сохраняется ли offset `scrollable` при замене контента?
- Есть ли Catppuccin Mocha в теме `two-face` 0.4 (а не только в master)?
- Как `markdown::Item::Image` трактует `file://` и относительные пути после `resolve`?

## Результаты реализации

- Адаптер: iced логирует `Selected: AMD Radeon Graphics (RADV RENOIR)`; llvmpipe остаётся в списке, но не выбирается. Проверка адаптеров вынесена в задачу после первого кадра, чтобы не удлинять старт; при выборе программного адаптера показывается экран ошибки.
- Шрифты: `fontdb` сканировал системные каталоги шрифтов и на NixOS читал сотни мегабайт, задерживая окно. Системный скан отключён через `FONTCONFIG_FILE` с конфигом, указывающим на несуществующий каталог; встроенный JetBrains Mono Nerd Font остаётся единственным шрифтом приложения.
- Позиция прокрутки: при обычной перезаписи файла состояние `scrollable` сохранялось, при атомарной замене через `rename` сбрасывалось. Реализован запасной путь: `on_scroll` запоминает `AbsoluteOffset`, после перезагрузки отправляется `scroll_to`.
- Watcher: события `Access` от чтения файла отфильтрованы; без этого чтение после reload порождало бесконечный цикл перезагрузок.
- Перетаскивание в X11 проверено синтетическим XDND-источником (`XdndEnter`/`XdndPosition`/`XdndDrop`, `text/uri-list`); заголовок окна сменился на имя перетащенного файла.
- Выделение текста: в iced 0.14 у `text`/`rich_text` нет готового API, поэтому выделение и Ctrl+C реализованы отдельным изменением `selectable-text` на собственном виджете.
