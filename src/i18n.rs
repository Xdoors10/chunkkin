//! Plugin message localization.
//!
//! English is the default language; Russian is selected via the `language`
//! field in `config.json`. All user-facing strings (chat feedback, help,
//! status labels, logs) are routed through [`tr`] / [`trf`].

use std::fmt::Display;
use std::sync::RwLock;

/// Supported plugin languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Ru,
}

impl Lang {
    /// Parses a config value; unknown values fall back to English.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "ru" | "rus" | "russian" | "russian_ru" => Self::Ru,
            _ => Self::En,
        }
    }

    /// Canonical config name for this language.
    #[must_use]
    pub const fn config_name(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ru => "ru",
        }
    }
}

static LANG: RwLock<Lang> = RwLock::new(Lang::En);

/// Switches the active plugin language.
pub fn set_lang(lang: Lang) {
    *LANG.write().unwrap_or_else(|e| e.into_inner()) = lang;
}

/// Returns the active plugin language.
#[must_use]
pub fn current_lang() -> Lang {
    *LANG.read().unwrap_or_else(|e| e.into_inner())
}

/// All localizable plugin messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    // Config
    ConfigParseError,
    ConfigSerializeError,
    ConfigWriteError,
    ConfigNotInitialized,
    ConfigLanguageUnknown,
    ConfigReloaded,
    ConfigReloadTaskNote,

    // Selection commands
    WorldUndetermined,
    WorldNotFound,
    StartWorldNotFound,
    WorldSelected,
    CenterSpecify,
    CenterSet,
    RadiusSpecify,
    RadiusPositive,
    RadiusSet,
    ErrorPrefix,
    StateSerializeError,
    StateWriteError,

    // Reports and labels
    SelectionHeader,
    LabelWorld,
    LabelCenter,
    LabelRadius,
    LabelRegion,
    LabelStatus,
    LabelStatusGen,
    LabelElapsed,
    LabelRemaining,

    // Units
    BlocksXY,
    BlocksX,
    ChunksXY,
    ChunksX,
    SpeedValue,

    // Task lifecycle
    TaskAlreadyRunning,
    TaskStarted,
    TaskPaused,
    TaskAlreadyPaused,
    NoTaskToPause,
    TaskResumed,
    TaskRunningAlready,
    NoTaskToResume,
    TaskCancelled,
    NoTaskToCancel,
    StatusTask,
    StatusProgressEnabled,
    StatusProgressDisabled,
    TaskRestored,
    WorldNotFoundTick,
    TaskCompleted,
    EtaComputing,

    // Task statuses
    StatusIdle,
    StatusRunning,
    StatusPaused,
    StatusCompleted,
    StatusCancelled,

    // Help
    HelpText,
}

/// Returns the translated message for `key` in the active language.
#[must_use]
pub fn tr(key: Msg) -> &'static str {
    match current_lang() {
        Lang::En => en(key),
        Lang::Ru => ru(key),
    }
}

/// Returns the translated message for `key` with `{0}`, `{1}`, ... placeholders
/// substituted by `args`.
pub fn trf(key: Msg, args: &[&dyn Display]) -> String {
    let mut text = tr(key).to_string();
    for (i, arg) in args.iter().enumerate() {
        text = text.replace(&format!("{{{i}}}"), &arg.to_string());
    }
    text
}

fn en(key: Msg) -> &'static str {
    match key {
        Msg::ConfigParseError => "Failed to parse config.json: {0}. Using default settings.",
        Msg::ConfigSerializeError => "Failed to serialize config: {0}",
        Msg::ConfigWriteError => "Failed to write file {0}: {1}",
        Msg::ConfigNotInitialized => "Plugin configuration is not initialized.",
        Msg::ConfigLanguageUnknown => "Unknown language '{0}' in config.json. Using English.",
        Msg::ConfigReloaded => "Configuration reloaded successfully.",
        Msg::ConfigReloadTaskNote => "A generation task is currently running: only engine settings (batch size, intervals) were updated; selection changes will apply to the next task.",

        Msg::WorldUndetermined => "Could not determine the world: specify a world name or run the command as a player.",
        Msg::WorldNotFound => "World '{0}' not found!",
        Msg::StartWorldNotFound => "World '{0}' not found! Select an existing world: /chunkkin world <world>.",
        Msg::WorldSelected => "Selected world: §a{0}§r.",
        Msg::CenterSpecify => "Specify X Z coordinates or run the command as a player.",
        Msg::CenterSet => "Generation center: §e{0}§r, §e{1}§r.",
        Msg::RadiusSpecify => "Specify the radius in blocks, e.g.: /chunkkin radius 5000",
        Msg::RadiusPositive => "The radius must be a positive number.",
        Msg::RadiusSet => "Radius: §e{0}§r (region {1}).",
        Msg::ErrorPrefix => "Error: ",
        Msg::StateSerializeError => "Failed to serialize task state: {0}",
        Msg::StateWriteError => "Failed to write file {0}: {1}",

        Msg::SelectionHeader => "Current selection:",
        Msg::LabelWorld => "World",
        Msg::LabelCenter => "Center",
        Msg::LabelRadius => "Radius",
        Msg::LabelRegion => "Region",
        Msg::LabelStatus => "Task status",
        Msg::LabelStatusGen => "Generation status",
        Msg::LabelElapsed => "Elapsed",
        Msg::LabelRemaining => "Remaining",

        Msg::BlocksXY => "{0} x {1} blocks",
        Msg::BlocksX => "{0} blocks",
        Msg::ChunksXY => "{0} x {1} chunks",
        Msg::ChunksX => "{0} chunks",
        Msg::SpeedValue => "{0} ch/s",

        Msg::TaskAlreadyRunning => "A pre-generation task is already running! Use /chunkkin pause or /chunkkin cancel.",
        Msg::TaskStarted => "Started pre-generation: world §a{0}§r, center §e({1}, {2})§r, region §e{3} x {4} chunks§r, total §6{5} chunks§r.",
        Msg::TaskPaused => "Generation paused. Progress saved. Use /chunkkin continue to resume.",
        Msg::TaskAlreadyPaused => "The task is already paused.",
        Msg::NoTaskToPause => "No active task to pause.",
        Msg::TaskResumed => "Generation resumed.",
        Msg::TaskRunningAlready => "The task is already running.",
        Msg::NoTaskToResume => "No paused task to resume.",
        Msg::TaskCancelled => "Pre-generation task cancelled. Generated {0}/{1} chunks.",
        Msg::NoTaskToCancel => "No active task to cancel.",
        Msg::StatusTask => "Task status: {0}",
        Msg::StatusProgressEnabled => "§aChat progress enabled§r. Use /chunkkin progress to toggle.",
        Msg::StatusProgressDisabled => "§cChat progress disabled§r. Use /chunkkin progress to toggle.",
        Msg::TaskRestored => "Restored pre-generation task for world '{0}': progress {1}/{2} chunks ({3}%). Status: {4}",
        Msg::WorldNotFoundTick => "World '{0}' not found during generation tick! Pausing.",
        Msg::TaskCompleted => "§aGeneration of world §6{0} §acompleted successfully! §e{1}§a chunks processed.",
        Msg::EtaComputing => "computing...",

        Msg::StatusIdle => "Idle",
        Msg::StatusRunning => "Running",
        Msg::StatusPaused => "Paused",
        Msg::StatusCompleted => "Completed",
        Msg::StatusCancelled => "Cancelled",

        Msg::HelpText => "§6Chunkkin commands:§r\n\
            §a/chunkkin world [world]§7 - Select the target world (yours when omitted)§r\n\
            §a/chunkkin center [x z]§7 - Generation center (your position when omitted)§r\n\
            §a/chunkkin radius <radius> [radius2]§7 - Radius in blocks; two values make a rectangle§r\n\
            §a/chunkkin selection§7 - Show the current selection§r\n\
            §a/chunkkin start§7 - Start generation with the current selection§r\n\
            §a/chunkkin pause§7 - Pause with progress saved§r\n\
            §a/chunkkin continue§7 - Resume the task§r\n\
            §a/chunkkin cancel§7 - Cancel and reset§r\n\
            §a/chunkkin status§7 - Progress report§r\n\
            §a/chunkkin progress§7 - Toggle periodic chat progress on/off§r\n\
            §a/chunkkin reload§7 - Reload the configuration from disk§r",
    }
}

fn ru(key: Msg) -> &'static str {
    match key {
        Msg::ConfigParseError => "Ошибка парсинга config.json: {0}. Используются настройки по умолчанию.",
        Msg::ConfigSerializeError => "Ошибка сериализации конфига: {0}",
        Msg::ConfigWriteError => "Ошибка записи файла {0}: {1}",
        Msg::ConfigNotInitialized => "Конфигурация плагина не инициализирована.",
        Msg::ConfigLanguageUnknown => "Неизвестный язык '{0}' в config.json. Используется английский.",
        Msg::ConfigReloaded => "Конфигурация успешно перезагружена.",
        Msg::ConfigReloadTaskNote => "Сейчас выполняется задача генерации: обновлены только настройки движка (размер батча, интервалы); изменения выбора применятся к следующей задаче.",

        Msg::WorldUndetermined => "Не удалось определить мир: укажите имя мира или выполните команду от лица игрока.",
        Msg::WorldNotFound => "Мир '{0}' не найден!",
        Msg::StartWorldNotFound => "Мир '{0}' не найден! Выберите существующий мир: /chunkkin world <мир>.",
        Msg::WorldSelected => "Выбран мир: §a{0}§r.",
        Msg::CenterSpecify => "Укажите координаты X Z или выполните команду от лица игрока.",
        Msg::CenterSet => "Центр генерации: §e{0}§r, §e{1}§r.",
        Msg::RadiusSpecify => "Укажите радиус в блоках, например: /chunkkin radius 5000",
        Msg::RadiusPositive => "Радиус должен быть положительным числом.",
        Msg::RadiusSet => "Радиус: §e{0}§r (область {1}).",
        Msg::ErrorPrefix => "Ошибка: ",
        Msg::StateSerializeError => "Не удалось сериализовать состояние задачи: {0}",
        Msg::StateWriteError => "Не удалось записать файл {0}: {1}",

        Msg::SelectionHeader => "Текущий выбор (selection):",
        Msg::LabelWorld => "Мир",
        Msg::LabelCenter => "Центр",
        Msg::LabelRadius => "Радиус",
        Msg::LabelRegion => "Область",
        Msg::LabelStatus => "Статус задачи",
        Msg::LabelStatusGen => "Статус генерации",
        Msg::LabelElapsed => "Прошло",
        Msg::LabelRemaining => "Осталось",

        Msg::BlocksXY => "{0} x {1} блоков",
        Msg::BlocksX => "{0} блоков",
        Msg::ChunksXY => "{0} x {1} чанков",
        Msg::ChunksX => "{0} чанков",
        Msg::SpeedValue => "{0} чанк/с",

        Msg::TaskAlreadyRunning => "Задача предварительной генерации уже выполняется! Используйте /chunkkin pause или /chunkkin cancel.",
        Msg::TaskStarted => "Запущена предварительная генерация: мир §a{0}§r, центр §e({1}, {2})§r, область §e{3} x {4} чанков§r, всего §6{5} чанков§r.",
        Msg::TaskPaused => "Генерация приостановлена. Прогресс сохранен. Для продолжения используйте /chunkkin continue.",
        Msg::TaskAlreadyPaused => "Задача уже приостановлена.",
        Msg::NoTaskToPause => "Нет активной задачи для приостановки.",
        Msg::TaskResumed => "Генерация возобновлена.",
        Msg::TaskRunningAlready => "Задача уже выполняется.",
        Msg::NoTaskToResume => "Нет приостановленной задачи для возобновления.",
        Msg::TaskCancelled => "Задача предварительной генерации отменена. Сгенерировано {0}/{1} чанков.",
        Msg::NoTaskToCancel => "Нет активной задачи для отмены.",
        Msg::StatusTask => "Статус задачи: {0}",
        Msg::StatusProgressEnabled => "§aПрогресс в чате включен§r. Используйте /chunkkin progress для переключения.",
        Msg::StatusProgressDisabled => "§cПрогресс в чате отключен§r. Используйте /chunkkin progress для переключения.",
        Msg::TaskRestored => "Восстановлена задача предварительной генерации мира '{0}': прогресс {1}/{2} чанков ({3}%). Статус: {4}",
        Msg::WorldNotFoundTick => "Мир '{0}' не найден во время тика генерации! Приостановка.",
        Msg::TaskCompleted => "§aГенерация мира §6{0} §aуспешно завершена! Обработано §e{1}§a чанков.",
        Msg::EtaComputing => "вычисление...",

        Msg::StatusIdle => "Ожидание",
        Msg::StatusRunning => "Выполняется",
        Msg::StatusPaused => "Приостановлено",
        Msg::StatusCompleted => "Завершено",
        Msg::StatusCancelled => "Отменено",

        Msg::HelpText => "§6Команды предварительной генерации:§r\n\
            §a/chunkkin world [мир]§7 - Выбор мира (без аргумента - ваш мир)§r\n\
            §a/chunkkin center [x z]§7 - Центр генерации (без аргумента - ваша позиция)§r\n\
            §a/chunkkin radius <радиус> [радиус2]§7 - Радиус в блоках; два значения - прямоугольник§r\n\
            §a/chunkkin selection§7 - Показать текущий выбор§r\n\
            §a/chunkkin start§7 - Запуск генерации по текущему выбору§r\n\
            §a/chunkkin pause§7 - Приостановка с сохранением§r\n\
            §a/chunkkin continue§7 - Возобновление§r\n\
            §a/chunkkin cancel§7 - Отмена и сброс§r\n\
            §a/chunkkin status§7 - Отчёт о прогрессе§r\n\
            §a/chunkkin progress§7 - Включение/выключение прогресса в чате§r\n\
            §a/chunkkin reload§7 - Перезагрузка конфигурации с диска§r",
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Exposed to other test modules that need to hold the shared language
    /// lock before switching the global language.
    pub(crate) fn lock_lang_for_tests() -> std::sync::MutexGuard<'static, ()> {
        LANG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn test_lang_parse() {
        assert_eq!(Lang::parse("en"), Lang::En);
        assert_eq!(Lang::parse("EN"), Lang::En);
        assert_eq!(Lang::parse("ru"), Lang::Ru);
        assert_eq!(Lang::parse(" RU "), Lang::Ru);
        assert_eq!(Lang::parse("russian"), Lang::Ru);
        assert_eq!(Lang::parse("klingon"), Lang::En);
    }

    /// Lock shared by tests that switch the global language, so they cannot
    /// race each other when the test harness runs them in parallel threads.
    static LANG_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_translation_switch() {
        let _guard = LANG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        set_lang(Lang::En);
        assert_eq!(tr(Msg::LabelWorld), "World");
        assert_eq!(tr(Msg::StatusRunning), "Running");

        set_lang(Lang::Ru);
        assert_eq!(tr(Msg::LabelWorld), "Мир");
        assert_eq!(tr(Msg::StatusRunning), "Выполняется");

        // Restore default for other tests
        set_lang(Lang::En);
    }

    #[test]
    fn test_trf_placeholders() {
        let _guard = LANG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        set_lang(Lang::En);
        let msg = trf(Msg::WorldNotFound, &[&"overworld"]);
        assert_eq!(msg, "World 'overworld' not found!");

        set_lang(Lang::Ru);
        let msg = trf(Msg::WorldNotFound, &[&"overworld"]);
        assert_eq!(msg, "Мир 'overworld' не найден!");

        set_lang(Lang::En);
    }

    #[test]
    fn test_all_messages_non_empty_both_languages() {
        let _guard = LANG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        for key in [
            Msg::ConfigParseError,
            Msg::ConfigSerializeError,
            Msg::ConfigWriteError,
            Msg::ConfigNotInitialized,
            Msg::ConfigLanguageUnknown,
            Msg::ConfigReloaded,
            Msg::ConfigReloadTaskNote,
            Msg::WorldUndetermined,
            Msg::WorldNotFound,
            Msg::StartWorldNotFound,
            Msg::WorldSelected,
            Msg::CenterSpecify,
            Msg::CenterSet,
            Msg::RadiusSpecify,
            Msg::RadiusPositive,
            Msg::RadiusSet,
            Msg::ErrorPrefix,
            Msg::StateSerializeError,
            Msg::StateWriteError,
            Msg::SelectionHeader,
            Msg::LabelWorld,
            Msg::LabelCenter,
            Msg::LabelRadius,
            Msg::LabelRegion,
            Msg::LabelStatus,
            Msg::LabelStatusGen,
            Msg::LabelElapsed,
            Msg::LabelRemaining,
            Msg::BlocksXY,
            Msg::BlocksX,
            Msg::ChunksXY,
            Msg::ChunksX,
            Msg::SpeedValue,
            Msg::TaskAlreadyRunning,
            Msg::TaskStarted,
            Msg::TaskPaused,
            Msg::TaskAlreadyPaused,
            Msg::NoTaskToPause,
            Msg::TaskResumed,
            Msg::TaskRunningAlready,
            Msg::NoTaskToResume,
            Msg::TaskCancelled,
            Msg::NoTaskToCancel,
            Msg::StatusTask,
            Msg::StatusProgressEnabled,
            Msg::StatusProgressDisabled,
            Msg::TaskRestored,
            Msg::WorldNotFoundTick,
            Msg::TaskCompleted,
            Msg::EtaComputing,
            Msg::StatusIdle,
            Msg::StatusRunning,
            Msg::StatusPaused,
            Msg::StatusCompleted,
            Msg::StatusCancelled,
            Msg::HelpText,
        ] {
            set_lang(Lang::En);
            assert!(!tr(key).is_empty(), "empty English text for {key:?}");
            set_lang(Lang::Ru);
            assert!(!tr(key).is_empty(), "empty Russian text for {key:?}");
        }
        set_lang(Lang::En);
    }
}
