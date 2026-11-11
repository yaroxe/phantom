# Phantom Launcher v1.0

Автономный лаунчер на eframe/egui: один бинарник, ~8 прямых зависимостей, без Horizon, без Microsoft-входа.

## Структура

```
phantom-launcher/
├─ Cargo.toml        отдельный пакет (пустой [workspace] отвязывает его от родительского репозитория)
└─ src/
   ├─ main.rs        окно, запуск eframe
   ├─ theme.rs       палитра Graphite + Cyber Yellow
   ├─ fx.rs          переливающийся неон, градиенты, свечение, HUD-уголки, «реактор»
   ├─ app.rs         боковая рейка, hero, нижний «док» с ИГРАТЬ, 4 страницы, потоки + канал сообщений
   ├─ config.rs      %APPDATA%\phantom\launcher.json
   ├─ java.rs        поиск javaw.exe, определение версии Java
   ├─ launch.rs      запуск Phantom / Java-версии / .jar, офлайн-UUID
   └─ modrinth.rs    поиск и скачивание через api.modrinth.com/v2
```

## Подключение

Как отдельный проект (рекомендуется): положите папку рядом с движком и соберите.

```
cd phantom-launcher
cargo run --release
```

Если нужно держать в общем workspace движка: переместите в `crates/phantom-launcher-native`,
добавьте путь в `members` корневого `Cargo.toml` и удалите строку `[workspace]` из
`phantom-launcher/Cargo.toml`. Профиль `[profile.release]` в этом случае тоже нужно перенести в корень.

Готовый файл: `target/release/phantom-launcher.exe`.

## Где что ищется

- **Phantom Engine**: `launcher.json → phantom_client`, затем `%APPDATA%\phantom\phantom-client.exe`,
  `bin\`, `versions\<новейшая>\`, рядом с лаунчером, `target\release\`. Аргументы: `--resources <путь> --username <ник>`.
  Ресурсы: `%APPDATA%\phantom\resources`, иначе `<папка клиента>\resources`.
- **Java**: JAVA_HOME, `Program Files\{Java, Eclipse Adoptium, Microsoft, Zulu, BellSoft, Amazon Corretto, Semeru}`,
  `.minecraft\runtime`, PATH. Выбирается самая младшая Java, подходящая под `javaVersion` версии.
- **Моды**: `<папка игры>\mods`, шейдеры: `<папка игры>\shaderpacks`.

## Ограничения v1.0

- Java: только ванильные версии 1.13+, уже установленные (библиотеки и assets должны лежать в `.minecraft`).
  Forge/Fabric-профили (`inheritsFrom`) и версии до 1.13 не поддерживаются. Скачивания ванили нет.
- Зависимости модов Modrinth не подтягиваются, скачивается только выбранный мод.
- Иконок проектов нет (плитка с первой буквой), чтобы не тащить image-loaders.
