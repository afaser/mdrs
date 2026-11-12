## MODIFIED Requirements

### Requirement: Выделение и подсветка

Цвет, радиус скругления и высота подсветки выделения, а также смещения линий подчёркивания и зачёркивания ДОЛЖНЫ (MUST) задаваться конфигом в `theme.selection`. Высота ДОЛЖНА (MUST) применяться ко всем не-кодовым блокам единообразно; для каждого уровня заголовка ДОЛЖНА (MUST) использоваться своя высота `theme.selection.headings.h1`…`h6`, а для строк многострочного fenced-блока кода — `code-height`. Тема подсветки синтаксиса ДОЛЖНА (MUST) выбираться встроенным именем или файлом `.tmTheme`, а цвета классов токенов (keyword, string, comment, number, function, type, operator, punctuation) ДОЛЖНЫ (MUST) переопределяться поверх темы.

Отсутствующие поля ДОЛЖНЫ (MUST) брать дефолты: `color "selection"`, `radius 3`, `height 22`, `headings.h1 44`, `headings.h2 39`, `headings.h3 33`, `headings.h4 28`, `headings.h5 22`, `headings.h6 22`, `code-height 17`, `underline-offset 0.08`, `strikethrough-offset 0.5`. Неизвестные поля и секции ДОЛЖНЫ (MUST) игнорироваться.

#### Scenario: Цвет выделения

- **WHEN** `theme.selection.color` равен `"#ff0000"`
- **THEN** выделенный мышью текст подсвечен красным во всех блоках документа

#### Scenario: Скругление выделения

- **WHEN** `theme.selection.radius` равен `6`
- **THEN** углы каждого прямоугольника подсветки скруглены радиусом 6 px

#### Scenario: Высота выделения

- **WHEN** `theme.selection.height` равен `28`
- **THEN** подсветка обычного текста, заголовков, списков, цитат и таблиц имеет высоту 28 px

#### Scenario: Высота заголовка

- **WHEN** `theme.selection.headings.h1` равен `46`, а `theme.selection.headings.h2` равен `40`
- **THEN** подсветка заголовка H1 имеет высоту 46 px, H2 — 40 px, а остальные не-кодовые блоки продолжают использовать `height`

#### Scenario: Высота строки кода

- **WHEN** `theme.selection.code-height` равен `20`
- **THEN** подсветка строк многострочного fenced-блока кода имеет высоту 20 px, а остальные блоки продолжают использовать `height`

#### Scenario: Дефолты секции

- **WHEN** конфиг не задаёт `theme.selection.radius`, `height`, `headings` и `code-height`
- **THEN** применяются радиус 3 px, высота 22 px, высоты заголовков 44/39/33/28/22/22 px и высота кода 17 px

#### Scenario: Переопределение токена

- **WHEN** `theme.highlighting.tokens.keyword` равен `"#ff0000"`
- **THEN** ключевые слова в код-блоках красные, остальные токены используют выбранную тему

#### Scenario: Файл темы

- **WHEN** `theme.highlighting.theme-file` указывает на существующий `.tmTheme`
- **THEN** подсветка использует цвета из файла, а отсутствующие классы берутся из дефолтной темы
