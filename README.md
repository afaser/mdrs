# mdrs

Нативный просмотрщик markdown на iced 0.14: собственный markdown- и текстовый слой, GPU-рендер через wgpu/Vulkan.

## Возможности

- Открытие файла аргументом, через Ctrl+O (XDG Desktop Portal) и перетаскиванием в X11/XWayland.
- Выделение текста мышью через любые блоки и целые строки из левого поля; Ctrl+C копирует исходные markdown-строки с разметкой.
- GFM-рендер: заголовки, абзацы, списки, цитаты, таблицы, изображения, ссылки, блоки кода.
- Подсветка кода через two-face с темой Catppuccin Mocha.
- Тема Catppuccin Mocha Green по умолчанию, встроенные Source Serif 4, Lora и JetBrains Mono Nerd Font.
- KDL-конфиг: палитра, прозрачность фона, шрифты и роли текста, рамка/шапка код-блока, скроллбары, анимация колеса, выделение и подсветка синтаксиса.
- Live reload по изменению файла с сохранением позиции прокрутки.
- Экран ошибки с путём, если файл не открывается.

## Конфиг

Конфиг читается из `$XDG_CONFIG_HOME/mdrs/config.kdl` (или
`~/.config/mdrs/config.kdl`); путь можно переопределить переменной
`MDRS_CONFIG`. Файл опционален: без него работают значения по умолчанию.
Готовый пример со всеми секциями — `config.example.kdl` в корне репозитория;
скопируйте его в `~/.config/mdrs/config.kdl` (или в `$XDG_CONFIG_HOME/mdrs/config.kdl`),
чтобы получить редактируемый конфиг. Неизвестные секции и поля игнорируются,
поэтому новые ветки можно добавлять, не ломая старые конфиги. Последнее
повторение поля побеждает.

```kdl
window {
    width 1000
    height 750
    antialiasing #true
    padding 24
    // transparent сам включается при background-opacity < 1
}

theme {
    palette {
        background "#1e1e2e"
        background-opacity 1.0
        text "#cdd6f4"
        accent "#a6e3a1"
        danger "#f38ba8"
        border "#45475a"
        surface "#313244"
        selection "#45475a"
        primary "#a6e3a1"
        success "#a6e3a1"
        warning "#f38ba8"
    }

    fonts {
        use-system #false
        latin {
            family "Source Serif 4"
            script "latin"
            regular "embedded"
            bold "embedded"
            italic "embedded"
            bold-italic "embedded"
        }
        cyrillic {
            family "Lora"
            script "cyrillic"
            regular "embedded"
            bold "embedded"
            italic "embedded"
            bold-italic "embedded"
        }
        code {
            family "JetBrainsMono Nerd Font"
            script "any"
            regular "embedded"
            ligatures #true
        }
        sets {
            body {
                regular "latin" "cyrillic"
                bold "latin" "cyrillic"
                italic "latin" "cyrillic"
                bold-italic "latin" "cyrillic"
            }
            heading {
                regular "latin" "cyrillic"
                bold "latin" "cyrillic"
                italic "latin" "cyrillic"
                bold-italic "latin" "cyrillic"
            }
            code {
                regular "code"
                bold "code"
                italic "code"
                bold-italic "code"
            }
        }
    }

    text {
        body {
            font "body"
            size 17
            weight "normal"
            line-height 1.3
            letter-spacing 0
            color "text"
        }
        heading {
            font "heading"
            color "accent"
            heading-gap 8.5
            letter-spacing 0
            h1 { size 34 }
            h2 { size 29.75 }
            h3 { size 25.5 }
            h4 { size 21.25 }
            h5 { size 17 }
            h6 { size 17 }
        }
        inline-code {
            font "code"
            size 13
            letter-spacing 0.5
            color "text"
            background "surface"
            padding { top 4; right 4; bottom 4; left 4 }
            radius 4
        }
        message { font "body"; size 18; color "danger" }
        link { color "accent"; underline #false; letter-spacing 0 }
        spacing 14.875
    }

    code-block {
        text { font "code"; size 13; line-height 1.3; color "text"; ligatures #true }
        frame {
            background "surface"
            border { color "border"; width 1; radius 12 }
            margin 8
        }
        header {
            show #true
            height 32
            background "background"
            padding { top 0; right 0; bottom 0; left 12 }

            logo { fonts "code"; size 16 }

            logos {
                rust { icon "\u{e7a8}"; color "#fab387" }
                bash { icon "\u{e795}"; color "accent" }
                unknown { icon "\u{f0f6}"; color "text" }
            }

            label {
                fonts "code"
                size 13
                color "text"
                capitalize #true
                unknown "Text"
            }

            copy {
                icon "embedded"            // или icon-path "/path/copy.svg"
                check-icon "embedded"      // или check-icon-path "/path/check.svg"
                size 20
                button-size 28
                color "text"
                hover-color "accent"
                background "#00000000"
                hover-background "surface"
                radius 8
                check-color "success"
                check-duration 0.9
                check-easing "0.25 0.1 0.25 1"
                hover-duration 0.15
                hover-easing "0.25 0.1 0.25 1"
            }
        }
        content {
            padding 12
            scrollbar { track "background"; thumb "border"; thumb-hover "accent"; width 7; scroller-width 7; spacing 3.5 }
        }
    }

    scrollbars {
        document { track "background"; thumb "border"; thumb-hover "accent"; width 8; scroller-width 6; spacing 8 }
    }

    motion {
        wheel {
            smoothing 0.03
            settle-distance 0.5
            lines-per-notch 8
            pixels-per-notch 15
            line-height 24
            external-distance 0.5
        }
    }

    selection {
        color "selection"
        radius 3
        height 22
        headings { h1 44; h2 39; h3 33; h4 28; h5 22; h6 22 }
        code-height 17
        decoration { underline-offset 0.08; strikethrough-offset 0.5 }
    }

    highlighting {
        theme "CatppuccinMocha"
        // theme-file "/path/custom.tmTheme"
        tokens {
            keyword "#cba6f7"
            string "#a6e3a1"
            comment "#6c7086"
            number "#fab387"
            function "#89b4fa"
            type "#f9e2af"
            operator "#89dceb"
            punctuation "#9399b2"
        }
    }
}
```

Цвет задаётся как `"#rrggbb"`, `"#rrggbbaa"` или именем из `palette`.
`inline-code` задаёт фон, `padding` (от текста кода до границы фона) и
`radius`. Пробелы исходного markdown вокруг inline-кода сохраняются: они
стоят снаружи границы чипа, и расстояние до соседнего слова равно ширине
этих пробелов. По умолчанию `padding` 4 px и `radius` 4 px.
Inline-код рисуется атомарным чипом: он не разрывается между строками,
переносы пересчитываются при изменении ширины окна.
`letter-spacing` задаёт межбуквенный интервал роли в пикселях и применяется
к телу, заголовкам, inline-коду, коду, сообщению и ссылкам.
Блоки с inline-кодом используют атомарную раскладку по словам; для них
bidi/CJK-переносы упрощены по сравнению с обычными блоками.
Наборы `fonts.sets` задают цепочки шрифтов для regular, bold, italic и
bold-italic один раз; роль выбирает набор через `font "body"`/`font "heading"`,
а конкретный уровень заголовка может переопределить его (`h1 { font "..." }`).
Кривая анимации задаётся именем (`linear`, `ease-in`, `ease-out`, `ease-in-out`)
или четырьмя числами cubic-bezier, например `"0.25 0.1 0.25 1"`.
Иконки: `icon "embedded"` — встроенная SVG, `icon "\u{f0c5}"` — glyph,
`icon-path "/path/icon.svg"` — внешний SVG.
`fonts.<name>` задаёт семейство, script (`latin`, `cyrillic`, `common`,
`any`) и файлы лиц (`regular`, `bold`, `italic`, `bold-italic`);
`"embedded"` — встроенный файл, `"system"` — поиск в системе,
иначе абсолютный путь. `use-system #true` отключает изоляцию системных
шрифтов.

Прежний вид (моноширинный JetBrains везде, код 14, без шапки):

```kdl
theme {
    fonts {
        latin { family "JetBrainsMono Nerd Font" }
        cyrillic { family "JetBrainsMono Nerd Font" }
    }
    text {
        body { size 17 }
        heading { color "accent" }
    }
    code-block {
        text { size 14; ligatures #false }
        header { show #false }
    }
}
```

Полупрозрачность требует поддержки со стороны композитора; на системе без
неё фон остаётся непрозрачным.

## Окружение

Зависимости окружения описаны в `flake.nix`; Wayland/X11, Vulkan loader и libdbus
попадают в `LD_LIBRARY_PATH` через direnv, `sway`, `Xwayland`, `grim`, `xdotool`
и `vulkan-tools` — в `PATH`. Контейнеры запускаются системным rootless podman.

```
direnv allow
```

## Сборка и запуск

Приложение запускается только в контейнере с вложенным headless-композитором и
hardware GPU; окно в сессии пользователя не открывается.

```
direnv allow
scripts/sandbox.sh path/to/file.md
```

Скрипт собирает release-бинарь с низким приоритетом, запускает его в rootless
podman без сети и хостовых сокетов под вложенным sway. Окно приложения — X11-клиент
rootless Xwayland внутри этого композитора: так колесо и drag-выделение подаются
настоящими событиями `xdotool`, а не заглушкой. Скриншоты — `target/screenshot.png`,
`target/screenshot-scroll.png`, `target/screenshot-selection.png`; замеры
прокрутки и drag — `target/perf.txt`. Логи — `target/sandbox-app.log` и
`target/sandbox-sway.log`. Без `/dev/dri/renderD*` запуск не выполняется,
программный рендер запрещён.

Приёмка производительности прокрутки и drag:

```
scripts/sandbox-perf.sh [path/to/file.md]
```

Скрипт повторяет документ до большого файла, гоняет приёмку из `sandbox.sh`,
проверяет смену кадра и долю CPU процесса и падает, если прокрутка или drag
занимают целое ядро.

Диагностика:

```
direnv exec . chrt -i 0 nice -n 19 ionice -c 3 cargo build --release
```

`target/sandbox-app.log` содержит информационный лог iced/wgpu, включая
выбранный адаптер.

## Замеры запуска

Референсная машина: NixOS, niri/Wayland, AMD Cezanne, Mesa 26.0.2, RADV и
llvmpipe, тёплый Mesa shader cache. Замер через `hyperfine` с фокусировкой окна:
без фокуса композитор не рисует окно на скрытом workspace и первый кадр
задерживается на секунды.

| Сценарий | Прогонов | p50 | p95 | max |
|---|---|---|---|---|
| Тёплый старт, стенные часы процесса | 50 | 103.5 мс | 148.1 мс | 167.3 мс |
| Тёплый старт, отметка первого кадра | 50 | 93.6 мс | 136.3 мс | 157.1 мс |
| Холодный кеш шейдеров, стенные часы | 5 | 121.0 мс | 155.0 мс | 155.0 мс |
| Холодный кеш шейдеров, первый кадр | 5 | — | — | 145.9 мс |

Холодный случай измерялся с `MESA_SHADER_CACHE_DISABLE=1`. Значимого штрафа
против тёплого кеша не видно: iced не перекомпилирует шейдеры из кеша Mesa на
этом пути.

Замеры выполнялись при загруженной машине (load average 7–9), поэтому отдельные
прогоны дают выбросы; в таблице приведён прогон из 50 запусков. Контрольный
прогон финального бинаря (20 запусков): стенные часы p50 126.6 мс / p95 144.7 мс,
первый кадр p50 116.8 мс / p95 134.2 мс. Внутренняя отметка `first-frame` и
стенные часы `hyperfine` различаются на время загрузки и завершения процесса.

Адаптер: iced выбирает `AMD Radeon Graphics (RADV RENOIR)`; llvmpipe не
используется. Если обнаружен только программный адаптер, приложение показывает
экран ошибки вместо тихого CPU-рендера.

## Ограничения

- Wayland не доставляет файлы при перетаскивании (ограничение winit); используйте
  Ctrl+O или аргумент. На X11/XWayland перетаскивание обрабатывается.
- Копирование возвращает целые строки исходного markdown, а не только видимый
  текст: частичное выделение внутри строки расширяется до всей строки.
- Подсветка пустых строк между блоками не рисуется, но сами строки попадают в
  копируемый диапазон.
