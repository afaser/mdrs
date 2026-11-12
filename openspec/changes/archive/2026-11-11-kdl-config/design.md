## Context

Сейчас все видимые параметры — константы в `src/theme` и `src/app`: палитра, размеры, шрифт, геометрия скроллбаров, сглаживание колеса, отступы и цвета markdown. `Program` создаётся без доступа к конфигу, `Viewer` получает только `markdown::Settings`, а `run()` жёстко задаёт тему, шрифт и размер окна. Код-блок рисуется стандартной реализацией `Viewer::code_block`. Спецификации требований — в `specs/configuration/spec.md` и дельте `specs/markdown-viewer/spec.md`; мотивация — в `proposal.md`.

Ограничения, формирующие решение:
- проект следует стилю `AGENTS.md`: std + минимум зависимостей, подсистема — каталог с `mod.rs`-индексом, один концепт — один файл, поток данных фазами, без паник в рабочем коде;
- iced 0.14 не умеет выбирать шрифт по script автоматически, а `Font::with_name` требует `&'static str`;
- iced по умолчанию не включает `advanced-shaping`; без него ASCII идёт через Basic-шейпинг без лигатур;
- прозрачность фона возможна только через `window::Settings::transparent`; alpha палитры попадает в clear color;
- требование первого кадра ≤150 мс сохраняется.

## Goals / Non-Goals

**Goals:**
- Один KDL-файл задаёт все видимые параметры; отсутствие файла даёт документированный дефолт.
- Новые секции добавляются без изменения парсера и без поломки старых файлов.
- Шрифты, script-runs, лица bold/italic и лигатуры работают предсказуемо на машине без системных шрифтов.
- Код-блок получает рамку, шапку с языком/prompt и копирование.
- Конфиг читается один раз на старте; ошибки не блокируют запуск.

**Non-Goals:**
- Горячая перезагрузка конфига и нескольких именованных тем/профилей.
- Настройка невидимых параметров: drag-threshold выделения, cursor inset, debounce watcher, уровень логов, префиксы ссылок.
- Собственный движок шейпинга или замена cosmic-text.
- Редактор конфига внутри приложения.

## Decisions

### 1. Крейт `kdl`, ручной декод

Зависимость: `kdl = { version = "6", default-features = false, features = ["span"] }`. Крейт даёт парсер KDL v2 и дерево `KdlDocument`; serde не включаем.

Декод ручной, через обёртку `Node<'a>` над `KdlNode` с типизированными доступами (`child`, `text`, `number`, `boolean`, `color`, `last_child`). Причины:
- нужен контроль «последнее повторение побеждает»;
- нужен покомпонентный откат к дефолту при неверном типе;
- serde-маппинг KDL неоднозначен (arguments vs properties vs children) и дал бы скрытые правила;
- ручной декод укладывается в стиль проекта: фаза читается последовательностью имён.

Альтернатива — `kdl` + serde/serde-kdl2 — меньше кода, но больше зависимостей и непрозрачное поведение при дублях и неизвестных узлах. Отклонено.

### 2. Фазы загрузки

```
Source (путь) -> Text (содержимое) -> Document (KdlDocument) -> Config (резолв)
```

- `Source::discover()` — `MDRS_CONFIG`, затем `$XDG_CONFIG_HOME/mdrs/config.kdl`, затем `~/.config/mdrs/config.kdl`.
- `Source::read()` — `Option<String>`, ошибка логируется.
- `Document::parse(&str)` — `Option<Document>`, ошибка `kdl` с позицией логируется.
- `Config::from_document(&Document)` — каждая секция накладывает заданные поля на `Default`.
- `Config::load()` — оркестратор; всегда возвращает валидный `Config`.

`Config` — корневой value-тип с `pub`-полями секций; инварианта нет. Секции — отдельные типы в своих файлах. Контейнеры (шрифты, токены) владеют `Vec`/`BTreeMap`, поля приватные, доступ через срезы/итераторы и consuming-геттеры.

### 3. Модульная раскладка

```
src/config/
  mod.rs              только mod + pub use
  source.rs           Source: поиск и чтение
  document.rs         Document: обёртка KdlDocument + парсинг
  node.rs             Node: типизированные доступы, last-wins
  configuration.rs    Config: секции, Default, from_document, load
  palette.rs          Palette, PaletteKey, resolve
  colors.rs           ColorRef: литерал или имя палитры, hex-парсер
  fonts.rs            Script, FaceSource, NamedFont, Fonts
  typography.rs       FontChain, Role, Headings, Text
  code_block.rs       Frame, Header, Label, Copy, Content
  scrollbars.rs       Bar, Scrollbars
  motion.rs           Wheel
  selection.rs        Selection, Decoration
  highlighting.rs     Highlighting, TokenClass, overrides
  window.rs           Window: size, antialiasing, transparent
```

Декодеры секций живут в файле секции; общие доступы — в `node.rs`; резолв цветов — в `colors.rs`/`palette.rs`. `mod.rs` не содержит логики.

### 4. Цвета и прозрачность

`ColorRef` — enum `Palette(PaletteKey) | Literal(Color)`. Строка `#rrggbb`/`#rrggbbaa` → литерал, иначе имя палитры. Резолв идёт после декода, когда палитра уже собрана. Дефолтные роли ссылаются на имена палитры, поэтому смена `palette.accent` перекрашивает заголовки и ссылки.

`background-opacity` умножает alpha фона. Если итоговая alpha меньше `1.0`, `window::Settings.transparent` включается автоматически; явный `window.transparent` может это переопределить. На композиторе без поддержки прозрачности alpha деградирует в непрозрачный фон — это документируется.

### 5. Шрифты, script-runs и лица

`NamedFont { name, family, script, faces, ligatures }`; `Faces { regular, bold, italic, bold_italic }`; `FaceSource::Embedded | System | Path(PathBuf)`. Дефолт — встроенные Lora, Source Serif 4 и JetBrains Mono NF; `assets/` содержит static regular/bold/italic/bold-italic.

`use-system #false` сохраняет текущую изоляцию fontconfig и детерминированный старт. `#true` отключает изоляцию; явный `path` работает всегда.

`Font::with_name` требует `&'static str`: имена семейств из конфига утекаются один раз на старте через `Box::leak`; число уникальных семейств ограничено размером конфига.

Script-runs:
- `Script::of(char)` — латиница, кириллица, общий знак;
- `runs(&str)` режет строку на непрерывные участки, общие знаки и пробелы прилипают к текущему run'у;
- `FontChain` выбирает первый шрифт цепочки с подходящим script; для общего знака — шрифт текущего run'а, в начале строки — первый шрифт;
- для каждого span подставляется `Font { family, weight, style }` выбранного лица; ссылка, цвет, подчёркивание, зачёркивание, highlight и padding сохраняются;
- span'ы inline-кода и подсветки с собственным шрифтом не переразбиваются.

`Str`-разбиение и назначение шрифтов — чистые функции; тестируются без окна.

### 6. Лигатуры и shaping

Добавляется feature `advanced-shaping`. `TextConfig` получает `Shaping`:
- роль кода с `ligatures #true` → `Shaping::Advanced`;
- остальные роли → `Shaping::Auto` (Basic для ASCII, Advanced для не-ASCII).

Это даёт лигатуры только там, где они нужны, и не платит advanced-шейпингом за весь документ. Альтернатива — включить Advanced глобально — проще, но дороже на больших документах.

### 7. Роли типографики

`Text` содержит `body`, `heading` с отдельными `h1..h6`, `inline_code`, `message`, `link`. Каждая роль — `FontChain`, `size`, `weight`, `line_height`, `color`; заголовки дополнительно имеют собственный размер и вес. `Typography::settings()` собирает `markdown::Settings`/`markdown::Style` из конфига; `SelectableText` получает `TextColors` роли.

Изменение размера тела не пересчитывает H1–H6: размеры заголовков абсолютные (дефолты совпадают с прежними 34/29.75/25.5/21.25/17/17). Это осознанный выбор в пользу прямого контроля.

### 8. Кэш стилизованных spans

`Viewer` владеет `SpanStyler` с кэшем `(Arc<[Span]> as ptr, TextRole) -> Arc<[Span]>`. `markdown::Text::spans` уже кэширует результат по стилю и возвращает стабильный `Arc`, поэтому повторные кадры не режут строки заново. Кэш очищается при смене контента/конфига.

### 9. Код-блок

`Viewer::code_block` собирает:
```
frame(container)
  header(row): prompt + label + spacer + copy-button
  content(scrollable): SelectableText(code spans)
```
Шапка скрывается целиком при `header.show #false`. Язык берётся из markdown, при отсутствии — `header.label.unknown`; капитализация настраивается. Кнопка копирования отправляет `Message::CopyCode(code.to_owned())`; `update` пишет строку в clipboard. Кнопка вне текста, поэтому не конфликтует с выделением.

Новый вариант `Message` и `impl From<CodeCopy> for Message`; `Viewer` требует `Message: From<CodeCopy>`.

### 10. Скроллбары и анимация

Стили скроллбаров собираются из `Scrollbars` в `scrollable::Style` (track/thumb/thumb-hover, ширины, отступы). `Scroll` получает `Wheel` из конфига: smoothing, settle, lines/pixels per notch, line height, external distance. `TEXT_LINE_HEIGHT` удаляется из `theme`; единственный источник — конфиг.

### 11. Подсветка синтаксиса

`Highlighting` задаёт встроенную тему по имени или `.tmTheme` пути. Для переопределений токенов используется собственный проход по `ParseState` + `ScopeStack` + `Highlighter::style_for_stack`: для каждого диапазона выбирается класс по самому специфичному scope, затем цвет класса заменяет цвет темы. Без переопределений работает прежний быстрый путь `HighlightLines`. Кэш блоков сохраняется; ключ включает конфиг (конфиг неизменен на процесс).

Классы: keyword, string, comment, number, function, type, operator, punctuation; список расширяется без смены формата.

### 12. Тема iced и точка входа

`run()`:
1. `Config::load()`;
2. `theme::isolate_system_fonts()` только при `fonts.use-system #false`;
3. собрать `iced::Theme::custom` из палитры;
4. зарегистрировать все `FaceSource` через `.font(bytes)`;
5. `.default_font(body.font)`, `.window(Settings { size, transparent, .. })`;
6. передать `Config` в замыкание `Program::new`.

`Program` хранит `Config`; `view`/`Viewer`/`Scroll` читают его. Конфиг не перечитывается на кадре.

### 13. Расширение новыми ветками

Добавление параметра:
1. новый файл-секция (или поле в существующей) с `Default`;
2. поле в `Config`;
3. вызов декодера в `Config::from_document`;
4. `pub use` в `config/mod.rs`;
5. потребитель читает `config.<section>`.

Парсер, `Document` и `Node` не меняются. Неизвестные узлы уже игнорируются, поэтому старые версии приложения переживают новый конфиг, а новая версия — старый.

## Risks / Trade-offs

- **Бандл шрифтов ~2 МБ** → приемлемо для нативного просмотрщика; добавить OFL-лицензии и зафиксировать версии файлов.
- **Старт с 12 лицами шрифтов** → регистрация байтов без сканирования fontconfig; замерить первый кадр и при необходимости оставить только regular/bold/italic/bold-italic для нужных script.
- **Advanced-шейпинг на всём тексте** → включать Advanced только для роли кода; остальные — Auto.
- **Source Serif 4 содержит кириллицу** → script-runs обязательны; покрыть тестом смешанного абзаца.
- **`'static` имена семейств** → один `Box::leak` на уникальное имя; число ограничено конфигом.
- **Прозрачность зависит от композитора** → авто-transparent только при alpha < 1; задокументировать деградацию до непрозрачного фона.
- **Стилизация spans на кадре** → кэш по указателю `Arc` и роли; очистка при reload.
- **Ошибка одного поля** → откат только этого поля, остальные применяются; тесты на неверный цвет/тип.
- **Смена дефолтного вида** → **BREAKING**; в README/примере конфига дать рецепт прежнего вида (JetBrains для body/code, code 14, шапка выключена).
- **Крейт `kdl` v6** → зафиксировать мажор; доступ к API изолирован в `document.rs`/`node.rs`, чтобы обновление не расползалось.

## Migration Plan

- Конфиг опционален: старые файлы отсутствуют, приложение стартует с новыми дефолтами.
- Пользователь, которому нужен прежний вид, задаёт `theme.fonts` с JetBrains для body/code, `size 14`, `header.show #false` и прежние цвета ролей.
- Откат: удалить/переименовать конфиг и вернуть предыдущий бинарь; конфиг не пишет данных и не меняет документы.
- Новые секции добавляются аддитивно; `MDRS_CONFIG` позволяет держать отдельный экспериментальный конфиг.

## Open Questions

Нет: спорные точки (крейт, глубина секций, набор видимых параметров, бандл шрифтов) закрыты до реализации; расширение классов токенов и новых секций не меняет формат и декодер.
