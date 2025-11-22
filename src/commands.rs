use aniapi::{CommandBuilder, Commands, CommandOptionBuilder, CommandOptionType};

/// Функция для получения команд плагина
pub fn get_commands() -> Commands {
    aniapi::logger::PluginLogger::debug("Получение списка команд плагина VoiceTemp");
    
    // Команда для настройки канала-триггера
    let setup_command = CommandBuilder::new("voicetemp-setup", "Настроить канал-триггер для временных голосовых каналов")
        .add_option(
            CommandOptionBuilder::new(CommandOptionType::Channel, "trigger_channel", "Голосовой канал, при входе в который создается временный канал")
                .required(true)
                .build()
        )
        .add_option(
            CommandOptionBuilder::new(CommandOptionType::Channel, "category", "Категория, в которой будут создаваться временные каналы (опционально)")
                .required(false)
                .build()
        )
        .build();

    // Команда для создания временного канала вручную
    let create_command = CommandBuilder::new("voicetemp-create", "Создать временный голосовой канал")
        .add_option(
            CommandOptionBuilder::new(CommandOptionType::String, "name", "Название канала (по умолчанию: имя пользователя)")
                .required(false)
                .build()
        )
        .add_option(
            CommandOptionBuilder::new(CommandOptionType::Integer, "limit", "Лимит пользователей (0 = без лимита)")
                .required(false)
                .build()
        )
        .build();

    // Команда для удаления временного канала
    let delete_command = CommandBuilder::new("voicetemp-delete", "Удалить текущий временный канал")
        .build();

    // Команда для просмотра настроек
    let info_command = CommandBuilder::new("voicetemp-info", "Показать информацию о настройках временных каналов")
        .build();

    // Создаем контейнер команд и добавляем все команды
    Commands::new()
        .add(setup_command)
        .add(create_command)
        .add(delete_command)
        .add(info_command)
}
