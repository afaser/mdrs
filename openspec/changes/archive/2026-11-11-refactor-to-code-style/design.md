## Context

См. `proposal.md` — Why. Изменение — приведение `src` к `AGENTS.md` без изменения наблюдаемого поведения. Спецификации `markdown-viewer` и `text-selection` описывают поведение, которого рефакторинг не касается; в `.openspec.yaml` стоит `skip_specs: true`.

Ограничения:

- только std и уже подключённые зависимости; новых крейтов нет;
- `mdrs::run` — единственный внешний API, он не меняется;
- база до правок: `cargo fmt --check` — чисто, `cargo clippy --all-targets -- -D warnings` — чисто, `cargo test` — 43 passed;
- тесты сценариев спецификаций сохраняются; удаляются только проверки «ничего не произошло» и проверки внутренней идентичности;
- наблюдаемое поведение, включая выделение, копирование, колесо, live reload и старт, должно остаться прежним.

## Goals / Non-Goals

**Goals:**

- Файл — один концепт, имя — по главному типу; реализация без тестов — в районе сотни строк.
- Именованные типы вместо кортежей и сырых индексов; у промежуточного состояния есть конструктор и операции.
- Однонаправленный поток, владение по значению, побочные эффекты только в `update`/`run`/потоковых границах.
- Один проход на фазу, один кэш на производные данные, никаких мёртвых полей и сеттеров.
- Тела: guard → плоские шаги → сборка; развилки — `match`, сравнения — именованные предикаты.
- Индекс `mod.rs` отражает состав каталога; внутренние межфайловые элементы скрыты на уровне файла.

**Non-Goals:**

- Новые возможности, оптимизация производительности, обновление iced, смена рендерера.
- Правки `openspec/specs` и любые изменения наблюдаемого поведения.
- Разделение одного `impl Widget` на несколько файлов через `pub(super)`-поля на весь виджет.
- Переписывание markdown-парсера, системы раскладки iced и cosmic-text.

## Результаты ревью по алгоритму AGENTS.md

### 1. Файлы

| Место | Находка | Решение |
|---|---|---|
| `document/source.rs` | файл назван по `source`, главный тип — `Document` | → `document/document.rs` |
| `selection/range.rs` | файл назван по `range`, главный тип — `Selection`; внутри безымянный диапазон | → `selection/selection.rs`, диапазон — `SelectionRange` |
| `theme/highlight.rs` | файл назван по действию, главный тип — `Highlighter` | → `theme/highlighter.rs` |
| `markdown/resolve.rs` | две темы: URI-кодек и переписывание путей изображений; 178 строк реализации | → `markdown/uri.rs` + `markdown/resolve.rs` |
| `selection/selectable_text.rs` | 459 строк, четыре темы: протокол виджета, кэш `Paragraph`, жест, отрисовка украшений | → `selectable_text.rs`, `text_layout.rs`, `state.rs`, `decoration.rs` |
| `app/program.rs` | 229 строк, три темы: состояние, подписки, маппинг событий | → `program.rs`, `subscription.rs`, `selection.rs` |
| `app/scroll.rs` | 151 строка; `Motion` — концепт с данными, но операции живут в `Scroll` | → `app/motion.rs` |
| `app/view.rs` | композиция + типографика + настройки markdown | константы → `theme/typography.rs`, настройки → `theme/style.rs` |
| `theme/viewer.rs` | `Viewer` + стилевые функции `code_style`, `heading_size`, `heading_top` | стилевые функции → `theme/style.rs` |
| `open/argument.rs` | файл для голого вычисления (чтение аргумента) | удалить, чтение — в `program.rs` |
| `app/selection.rs` | пара `selection + selection_epoch` без имени и инварианта | новый концепт `SelectionState` |
| `selection/generation.rs` | `u64`-эпоха растекается между `Program`, `Viewer`, `SelectableText` | новый концепт `Generation` |
| `theme/typography.rs` | `TEXT_SIZE`/`TEXT_LINE_HEIGHT` в `view`, `CODE_SIZE` — литералом | новый концепт с размерами текста и кода |
| индексы `mod.rs` | `Gpu`, `GpuState`, `Highlighter`, палитра, `Startup`, `SCROLL_ID`, `logging::init` — `pub` без реэкспорта | политика видимости (решение 16) |

### 2. Сущности

- `Selection::range` возвращает `Option<(Cursor, Cursor)>` — роль в данных без имени; порядок нормализуется вручную. → `SelectionRange { from, to }` с конструктором, `min/max`, отсечением равных курсоров; `Selection::range` становится приватным.
- `Rewrite { start, end, replacement }` — имена по роли, не по содержимому. → `from`/`to`.
- `Highlighter` держит два кэша (`lines`, `blocks`), второй выводится из первого; `type Line` — промежуточное представление. → один кэш блоков, `Line` удаляется.
- `SelectableText` содержит поля `align_x`, `align_y`, `line_height`, `width`, `height`, которые нигде не настраиваются; `width()` не вызывается, `height(Length::Shrink)` дублирует дефолт; дефолт `selection_color` дублирует `palette::SELECTION`. → `TextConfig` с дефолтами, `TextColors` как обязательный аргумент, мёртвые поля/сеттеры удаляются.
- `Drag { moved: bool }` — bool-флаг поведения. → `enum Drag { Pressed { origin, link_span }, Selecting }`.
- `Startup { enabled: bool }` дублирует `Option<Startup>` в `Program`. → `Startup::new -> Option<Self>`.
- `Program.selection: Option<String>` и `selection_epoch: u64` ходят вместе, имеют инвариант «очистка увеличивает поколение». → `SelectionState` с методами.
- Четыре поля-эпохи (`Program`, `Viewer`, `SelectableText`, `State`) — один механизм без имени. → `Generation`.
- `Scroll` содержит `Motion`, но `Motion` не имеет операций; переходы анимации размазаны по `Scroll`. → `Motion` с `start/retarget/advance/settled`.
- `Gpu::detect` смешивает IO (создание `Instance`, перечисление) и выбор адаптера; предикат «не CPU» повторён в `find` и в guard; `GpuState::apply` логирует внутри перехода. → чистая `select` над `Gpu`, логирование в `detect`/`update`.
- `WheelScroll::State` и `SelectableText::State` — родовые имена. → `WheelState`, `TextState`.
- `Rule` и `Range` — родовые имена, конфликтующие с std/предметной областью. → `LineRule`, `SelectionRange`.
- `Document` владеет `String`, но не является value-типом; неиспользуемый `Clone` удаляется. `Selection`/`Dialog` имеют неиспользуемый `PartialEq` — удаляется.
- `Highlighter::lines` — публичная операция, живущая только для тестов. → удаляется вместе с кэшем строк.
- `Article` — не контейнер-обёртка, а концепт разбора; `items()` достаточно, consuming-геттер не добавляется (нет потребителя).

### 3. Дубль и состояние

- `Highlighter::block`/`lines` дважды строят `Key` и дважды делают lookup/insert одного и того же; `block` аллоцирует `language.to_owned()` на каждый кадр. → один кэш `HashMap<String, HashMap<u64, Arc<[Span]>>>`, lookup по `&str`, один проход сборки spans.
- `SelectableText::layout` дважды пишет литерал `Text { content, bounds, size, ... }`, различающийся только `content`. → `text_layout::text(content, bounds, config)`.
- `SelectableText::layout` трижды повторяет `selection.clear(); drag = None` (смена эпохи, смена spans, `Difference::Shape`), но не сбрасывает `hovered_span`; `state.spans` хранит глубокую копию spans (`to_vec`) только ради сравнения. → `TextState::reset_interaction` (включая hover), `TextLayout` сравнивает срезы и хранит `Arc<[Span]>` без глубокого клона.
- `SelectableText::draw` два одинаковых блока `fill_quad` для underline и strikethrough, различие — смещение линии. → `LineRule` + одна функция линии.
- `Viewer` трижды повторяет цепочку `SelectableText::new(...).color(...).selection_color(...)...`, различие — роль текста. → `TextRole { Heading, Paragraph, Code }` и один конструктор.
- `Program::subscription` — три `if/else`-выражения; `keyboard_event` — две одинаковые guard-ветки Ctrl+O/Ctrl+C; `update` — две ветки `Picked` с дублированным `finish_dialog`. → `match` по `Option`/`match` по клавише/одна ветка.
- `Gpu::detect` повторяет предикат «не CPU»; `link::is_browser_link` повторяет проверку префикса; `logging` объявляет уровень дважды (`enabled` и `max_level`); `style` дважды вызывает `regular()`. → именованные предикаты/константы/одно значение.
- `update::open_document` вызывает `stop_scroll`, хотя `Program::open` уже сбрасывает `Scroll`; `view::settings` берёт `let mut` ради одного поля. → убрать дубль/`let mut`, `Settings { code_size, .. }`.
- Негативные тесты отдельно от позитивных: `resolve` (external/data/anchor), `watch` (unrelated/access), `scroll` (external/matching), `selection` (два bounds-теста), `wheel_scroll` (три предиката). → слить в позитивные сценарии (решение 17).

### 4. Поток

- `app/view.rs` пишет эпоху в `Viewer` через `Cell` во время отрисовки — боковой канал. → запись в `update`.
- `Startup::finish` завершает процесс из фазы замера. → выход в `update::first_frame`.
- `Gpu::detect` делает два прохода (сбор `Vec`, логирование, затем выбор). → один проход.
- `Scroll::wheel` читает `Instant::now()` внутри перехода состояния — скрытая зависимость от часов, тесты вынуждены ждать реальное время. → `now` параметром, как в `frame`.
- `resolve` всегда возвращает owned-копию, даже когда переписываний нет. → `Cow<str>`.
- `font::isolate_system_fonts` пишет env при каждом вызове; тесты `selection` вызывают её параллельно. → `Once`.
- `WheelScroll` реализует `diff`, но регрессионного теста нет; дефолт `Widget::diff` очищает `tree.children`, а `layout/update/operate` обращаются к `tree.children[0]`. → сохранить `diff` через `tree.diff_children`, добавить регрессионный тест.
- `Highlighter` использует `RefCell` для кэша во время `view` — принято как memoization без наблюдаемого состояния (исключение, решение 18).
- `viewer::image` делает `Path::is_file()` на каждый кадр — IO в отрисовке; принято как граница проверки alt-текста, пока markdown-трейт не даёт асинхронного статуса загрузки (исключение, решение 18).
- `Content::open`/`Program::open`/`Content::reload` копируют `PathBuf` — принято: у документа и watcher независимые владения, копия происходит на открытии/перезагрузке, не на кадре (исключение, решение 18).

### 5. Тела и числа

- If/else-выражения: `Scroll::frame` (позиция), `resolve_uri` (абсолютность), `Selection::text` (`from`/`to`), `viewer::heading` (top), `encode_percent` (запись байта), `Selection::range` (порядок). → guard-выход в именованных функциях, `min/max`, `base.join`.
- `Selection::text` содержит составные условия в глубине цикла (`line == start.line`, `line == end.line`). → `line_from`/`line_to` с ранним возвратом.
- `SelectableText::update` содержит `if let … else` и вложенные `if` в ветке drag; `inside` и `position` двумя путями вычисляют одну относительную точку. → guard + enum `Drag`, общий helper относительной точки.
- Сравнения `f32` без допуска: `Scroll::wheel` (`distance == 0.0`), `WheelScroll` (`y != 0.0`), `Selection::bounds` (`width <= 0.0`), `cursor_at` (`bounds.width <= 0.0`). → предикаты `is_zero`/`has_area` с `f32::EPSILON`.
- `elapsed_ms` возвращает `f64` при `f32` во всём проекте. → `f32`.
- `decode_percent` — индексный цикл с ручными `index += 3/1`. → обход через `split_first`.
- `encode_percent` аллоцирует `String` на каждый байт через `format!`. → `write!` в общий буфер.
- `image_rewrites` и `Selection::bounds` прячут вычисления в больших `filter_map`-замыканиях. → именованные `rewrite`/`run_rect`.
- `cursor_at` использует магический зазор `0.01`. → именованная константа/предикат.
- `SelectableText::layout` содержит `let _ = renderer;` — мёртвая строка; `state.hovered`/`Drag.link` названы по роли, а не по содержимому (индексы span). → удалить/переименовать `hovered_span`/`link_span`.
- `view`/`run` содержат магические размеры окна/скроллбара/паддинга; это презентационные константы композиции — принято, выносятся только размеры текста/кода (исключение, решение 18).

### 6. Прогон и тесты

- База: 43 теста. Слияние/удаление: `resolve` −3 (три «unchanged» входят в позитивный), `watch` −2, `scroll` −1, `selection` −1, `highlight` −1, `wheel_scroll` −2.
- Добавление: `gpu::select` +2 (приоритет железа, пустой список), `WheelScroll::diff` +1, не-UTF-8 файл +1, `file_path` для не-file URI +1, пустой `resolve` +1, несколько изображений +1. Итог — 40.
- Тест `editing_file_produces_debounced_event` дублирует production-обвязку notify и зависит от таймингов; debounce-логика им не проверяется. → детерминированный тест `touches` (позитив + отрицательные), реальный watcher — в ручном smoke.
- `repeated_lookup_returns_identical_cache` проверяет идентичность `Arc`, а не наблюдаемое поведение. → удалить.
- Обязательные границы сохраняются: пустой файл/код/выделение/список адаптеров, равные курсоры, отсутствующий путь, лимит прокрутки 0.
- Продовый код не содержит `unwrap`/`expect` — проверено grep; это сохраняется.

## Decisions

### 1. Чистый рефакторинг без дельта-спецификации

`openspec/specs` описывает поведение, структура кода — предмет `AGENTS.md`. Альтернатива — capability «code-style» — отклонена: дублирует инструкцию и создаёт не-поведенческие требования. `skip_specs: true` уже выставлен.

### 2. Итоговая карта файлов

| Было | Стало |
|---|---|
| `document/source.rs` | `document/document.rs` |
| `selection/range.rs` | `selection/selection.rs` (`Selection`, приватный `SelectionRange`) |
| `theme/highlight.rs` | `theme/highlighter.rs` |
| `markdown/resolve.rs` | `markdown/resolve.rs` + `markdown/uri.rs` |
| `selection/selectable_text.rs` | `selectable_text.rs` + `state.rs` + `text_layout.rs` + `decoration.rs` |
| `app/program.rs` | `program.rs` + `subscription.rs` + `selection.rs` |
| `app/scroll.rs` | `scroll.rs` + `motion.rs` |
| `app/view.rs` | `view.rs` + `theme/typography.rs` + `theme/style.rs` |
| `theme/viewer.rs` | `viewer.rs` + стилевые функции в `theme/style.rs` |
| `open/argument.rs` | удаляется, ввод — в `app/program.rs` |
| `app/program.rs::SCROLL_ID` | `app/view.rs::SCROLL_ID` |
| — | `selection/generation.rs` |

### 3. Модель выделения

- `SelectionRange { from: Cursor, to: Cursor }`: конструктор возвращает `None` при равных курсорах, иначе нормализует порядок через `min/max`; приватный тип в `selection.rs`.
- `Generation(u64)`: `new`, `advance` (wrapping), сравнение; живёт в `selection/generation.rs` и реэкспортируется. Заменяет четыре `u64`-эпохи.
- `SelectionState { text: Option<String>, generation: Generation }` в `app/selection.rs`: `set(Some)` меняет текст, `clear()` обнуляет текст и продвигает поколение; `Program` делегирует.
- `Viewer` хранит `Generation` и получает его через `set_generation(&mut self)` в `update`; `view` только читает.
- `TextState::sync_generation(generation)`: при смене поколения очищает выделение, **если drag не активен**, и всегда принимает новое поколение. Это убирает ручной `wrapping_add(1)` в виджете: собственное сообщение об очистке не сбрасывает начатый drag, а внешняя очистка сбрасывает. Правило формулируется одной фразой.

### 4. Декомпозиция `SelectableText`

`impl Widget` нельзя разнести по файлам без утечки приватных полей; режется не impl, а данные и чистые фазы:

- `text_layout.rs`: `TextConfig` (value-тип: размер, шрифт, высота строки, выравнивание, перенос) и `TextLayout` (кэш `TextParagraph` + `Arc<[Span]>`); `sync(config, spans, bounds) -> LayoutChange` (`Unchanged | Resized | Reshaped`), `buffer`, `span_bounds`, `hit`, `hit_span`; общий `text(content, bounds, config)`.
- `state.rs`: `TextState { layout, selection, drag, hovered_span, generation }`; `sync_generation`, `press`, `extend`, `release`, `hover`; `Drag::Pressed { origin, link_span } | Drag::Selecting`; `release` возвращает `Release { Click { link }, Selected { text } }`.
- `decoration.rs`: `LineRule { Underline, Strikethrough }` с `offset(size)`, одна функция линии; выделение по-прежнему через `Selection::bounds`.
- `selectable_text.rs`: публичный builder, `Widget`-протокол, `From`; `TextColors { text, selection }` — обязательный аргумент `new`, дефолты палитры не дублируются.
- Остаток `selectable_text.rs` — протокольная склейка около 200 строк; это осознанный предел, а не пропущенное разделение.

### 5. `Highlighter`

Один кэш `HashMap<String, HashMap<u64, Arc<[Span]>>>`: внешний ключ — язык, lookup по `&str` без аллокации; внутренний — хеш кода. `block` — единственная публичная операция; spans собираются за один проход с вставкой `\n`; `lines`, `Line`, `Key` удаляются. `Highlighter` — `pub(super)`, реэкспорт не нужен.

### 6. `Viewer`

- `TextRole { Heading, Paragraph, Code }`; один приватный конструктор `SelectableText`, различие роли — `match` (цвета, перенос, ссылка).
- `Generation` вместо `Cell`; стилевые функции `code_style`, `heading_size`, `heading_top` переезжают в `theme/style.rs`, `markdown_settings` — туда же.
- `image` оставляет проверку `is_file` в отрисовке — принятое исключение (нет синхронного статуса загрузки в markdown-трейте).

### 7. `resolve` и `uri`

- `uri.rs`: `decode_percent` (обход через `split_first`, без индексного цикла), `encode_percent` (`write!`, без `format!` на байт), `has_scheme`, `file_path`, `resolve_uri` (объединение через `base.join`).
- `resolve.rs`: `resolve(source, base) -> Cow<'_, str>`, `Rewrite { from, to, replacement }`, `rewrite(...)` вместо замыкания, `destination_range`, `apply_rewrites`.
- Тесты кодеков/`file_path` — в `uri.rs`, переписывание — в `resolve.rs`; ассерты не ослабляются.

### 8. `app`: подписки, ввод, состояние выбора

- `subscription.rs`: `subscription(&Program)` + `keyboard_event`/`redraw_event`/`program_event`; сборка подписок — `match` по `Option`; подписка на `RedrawRequested` постоянна, потому что `update::first_frame` запускает GPU-check и при выключенном `MDRS_STARTUP`; ярлыки — один `match` по клавише без аллокации.
- `SelectionState` вместо пары полей; `Program::reload` — `mem::take` (у `Content` появляется `Default::Empty`), без sentinel-`mem::replace`.
- Чтение аргумента — в `program.rs`; `open/argument.rs` удаляется. `SCROLL_ID` — в `view.rs`, `pub(super)`.
- `Program::open`/`Content::open`/`Content::reload` сохраняют копии `PathBuf`: у документа и watcher независимые владения; копия на открытии приемлема (исключение).

### 9. `Scroll` и `Motion`

- `Motion` в `app/motion.rs`: `start(position, target, now)`, `retarget(target)`, `advance(now)`, `settled()`; константы `SMOOTHING_SECONDS`, `SETTLE_DISTANCE` — там.
- `Scroll` хранит `offset: AbsoluteOffset`, `limit`, `motion`; `wheel(delta, now)` получает время с границы; `frame(now)` использует `snapped`/предикат; `stop` и `reset` упрощаются, `Default`.
- `Motion.issued` и `AbsoluteOffset.x` сохраняются: `issued` — последняя выданная позиция (отличается от наблюдённой viewport в пределах допуска), `x` — часть контракта iced `AbsoluteOffset`; их удаление меняло бы поведение/API без выгоды.

### 10. `Startup`

`Startup::new(started) -> Option<Self>` через именованный предикат `measurement_requested()`; `enabled` и двойные guard удаляются; `elapsed_ms -> f32`; `finish(self) -> f32` печатает замер, а `std::process::exit(0)` выполняет `update::first_frame` — граница.

### 11. `Gpu`

- `select(adapters: impl Iterator<Item = Gpu>) -> Gpu`: первый `Hardware`, иначе первый, иначе `Software("no Vulkan adapter")`; чистая функция, тестируется без Vulkan.
- `detect` = `select(infos.map(to_gpu).inspect(log))`, один проход; `to_gpu` использует предикат `is_cpu`.
- `GpuState::apply` без логирования; `begin_check` возвращает `GpuCheck { Started, Skipped }` вместо bool; `GpuState` — `pub(super)`, `Gpu` реэкспортируется (входит в `Message`).

### 12. `WheelScroll`

- `diff` уже реализован через `tree.diff_children(std::slice::from_ref(&self.content))`; добавить регрессионный тест на сохранение дочернего дерева, чтобы дефолтная очистка `children` не вернулась.
- `WheelState` вместо `State`; `captures_wheel(modifiers)` вместо bool; составное условие — именованный предикат; нулевая дельта — `f32::EPSILON`.

### 13. Шрифты

`isolate_system_fonts` оборачивается в `Once` (идемпотентно, без гонки `set_var` из параллельных тестов); файл `font.rs` остаётся концептом шрифта.

### 14. Типографика и стиль

`theme/typography.rs`: `TEXT_SIZE`, `TEXT_LINE_HEIGHT`, `CODE_SIZE`; реэкспорт из `theme`. `markdown_settings` — в `theme/style.rs`, `view` только собирает виджеты. `scroll` зависит от `theme::TEXT_LINE_HEIGHT`, не от `view`.

### 15. Тела и числа

- If/else-выражения заменяются guard-выходом/именованными функциями; `if let … else` — guard; `Selection::text` — `line_from`/`line_to`; пустой результат — `then_some`/`filter`.
- Порядок/границы — `min/max`; абсолютность — `base.join`; сравнения — `is_zero`/`has_area`/`within` с `f32::EPSILON`; намеренные допуски анимации остаются константами.
- `let mut` только по необходимости (`settings` — struct update); аккумуляторы с явным типом (`spans`, `decoded`); крупные замыкания — именованные функции; `decode_percent` без индексов.
- Мёртвое удаляется: `let _ = renderer`, `SelectableText::width/height/align_*/line_height`, `Highlighter::lines`, неиспользуемые derive.

### 16. Видимость и индекс

`pub` + `pub use` — API подсистемы; внутренние межфайловые элементы — `pub(super)`. Под правило попадают `Highlighter`, `palette` (кроме `DANGER`), `SCROLL_ID`, `gpu::{detect,select,GpuState,GpuCheck}`, `subscription`, `Startup`, `logging::init`, `SelectionState`, `Motion`, `TextState`, `TextConfig`, `TextLayout`, `LineRule`; `SelectionRange` приватна внутри `selection.rs`. Реэкспортируются: `Gpu`, `Generation`, `TextColors`, `Selection`, `SelectableText`, `markdown_settings`, `TEXT_SIZE`/`TEXT_LINE_HEIGHT`/`CODE_SIZE`. Индексы — приватные `mod` + `pub use` в алфавитном порядке; удалённые файлы не оставляют ссылок.

### 17. Тесты

- Слияние: `resolve` external/data/anchor → в позитивный relative; `watch` unrelated/access → в детерминированный `touches`-сценарий (production watcher — ручной smoke); `scroll` external/matching → один; `selection` два bounds-теста → один; `wheel_scroll` три предиката → один.
- Удаление: `repeated_lookup_returns_identical_cache` (внутренний `Arc`), недетерминированный `editing_file_produces_debounced_event` заменяется тестом `touches`.
- Добавление: `gpu::select` (приоритет железа; пустой список), `WheelScroll::diff` (дочернее дерево), не-UTF-8 файл, `file_path` для не-file URI, пустой `resolve`, несколько изображений.
- Итог 40 тестов; каждый сценарий `markdown-viewer`/`text-selection` покрыт тестом или ручной проверкой (список фиксируется в задаче).

### 18. Принятые исключения

- `Highlighter` использует `RefCell`-кэш в `&self` — memoization, не наблюдаемое состояние.
- `viewer::image` делает `is_file` на кадр — единственный доступный синхронный способ выбрать alt-текст.
- `PathBuf`-копии при open/reload — независимые владения документом и watcher.
- `AbsoluteOffset.x` и `Motion.issued` — контракт iced и различие «выдано/наблюдено».
- Презентационные литералы (размер окна, метрики скроллбара, паддинги) остаются в `run`/`view`; выносятся только размеры текста/кода.
- `Article` остаётся концептом разбора с `items()`, без consuming-геттера.

## Risks / Trade-offs

- **Один `impl Widget` нельзя разделить** → `selectable_text.rs` ~330 строк: протокол, draw и update одного trait impl; чистые фазы вынесены, дальнейшее дробление требует `pub(super)`-полей на весь виджет (не цель).
- **Упрощение поколений может изменить очистку выделения** → правило «смена поколения очищает, кроме активного drag» проверяется матрицей: выделение в A → клик в B → клик вне текста → Ctrl+C → открытие нового файла.
- **`WheelScroll::diff` не покрыт тестом** → регрессионный тест на `children.len() == 1` после `diff`; ручной smoke на перерисовке.
- **Слияние тестов уменьшает число** → удаляются только «ничего не произошло» и внутренняя идентичность; перечень сценариев фиксируется.
- **`Cow`/`Once`/`write!` меняют типы/внутренности** → поведение сохраняется, тесты `resolve`/`selection` остаются.
- **`Gpu::select` вынесен из IO** → тесты на списках `Gpu`; реальный выбор адаптера — `MDRS_LOG=1` вручную.
- **`font` `Once`** → устраняет гонку, но делает изоляцию необратимой в процессе; это и раньше было глобальным побочным эффектом.
- **Ручной GUI-smoke не автоматизирован** → сценарии перечислены; при отсутствии дисплея фиксируется как непроверенное.

## Migration Plan

1. Переименования и разделения файлов; после каждого шага — `nice -n 19 cargo check`.
2. Перенос кода без изменения тел; `nice -n 19 cargo test` контролирует перенос.
3. Правки сущностей/потока/тел/видимости; `nice -n 19 cargo test` и `nice -n 19 cargo clippy`.
4. Слияние и добавление тестов; финальный `nice -n 19 cargo fmt --check`, `nice -n 19 cargo clippy --all-targets -- -D warnings`, `nice -n 19 cargo test` (40).
5. Ручной GUI-smoke и повторный проход «Ревью». Откат — revert коммита; данные на диске не мигрируются.

## Open Questions

Нет.
