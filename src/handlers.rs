use aniapi::{InteractionEvent, VoiceStateUpdateEvent};
use crate::config::VoiceTempConfig;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serenity::all::Http;
use std::error::Error;

/// Обработчик взаимодействий (slash команд)
pub fn handle_interaction(
    event: InteractionEvent,
    user_channel_map: Arc<Mutex<HashMap<u64, u64>>>,
) -> Result<Option<Value>, Box<dyn Error>> {
    aniapi::logger::PluginLogger::debug(&format!("Обработка взаимодействия: команда '{}'", event.command_name));

    let response_content = match event.command_name.as_str() {
        "voicetemp-setup" => handle_setup(&event.interaction_data),
        "voicetemp-create" => handle_create(&event.interaction_data),
        "voicetemp-delete" => handle_delete(&event.interaction_data, Arc::clone(&user_channel_map)),
        "voicetemp-info" => handle_info(Arc::clone(&user_channel_map)),
        _ => {
            aniapi::logger::PluginLogger::warn(&format!("Неизвестная команда: {}", event.command_name));
            return Ok(None);
        }
    };

    if let Some(content) = response_content {
        let response_json = serde_json::json!({
            "content": content,
            "ephemeral": false
        });
        Ok(Some(response_json))
    } else {
        Ok(None)
    }
}

/// Обработчик событий голосовых каналов
pub fn handle_voice_state_update(
    event: VoiceStateUpdateEvent,
    user_channel_map: Arc<Mutex<HashMap<u64, u64>>>,
    _http_client: Arc<Http>,
) {
    let config = load_config().unwrap_or_default();

    // Извлекаем информацию о пользователе и канале
    let channel_id = event.new_state.get("channel_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok());

    let guild_id = event.new_state.get("guild_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok());

    if let (Some(user_id), Some(channel_id_val), Some(_guild_id_val)) = (Some(event.user_id), channel_id, guild_id) {
        // Проверяем, зашел ли пользователь в канал-триггер
        if let Some(trigger_id) = config.trigger_channel_id {
            if channel_id_val == trigger_id {
                // Пользователь зашел в канал-триггер - создаем временный канал
                aniapi::logger::PluginLogger::info(&format!("Пользователь {} зашел в канал-триггер {}, создаем временный канал", user_id, trigger_id));
                
                // Получаем имя пользователя из JSON (если доступно)
                let user_name = event.new_state.get("member")
                    .and_then(|m| m.get("user"))
                    .and_then(|u| u.get("username"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("Пользователь");

                // Формируем имя канала
                let channel_name = config.channel_name_template
                    .replace("{user}", user_name);

                // TODO: Создать канал через HTTP клиент
                // Пока используем заглушку
                aniapi::logger::PluginLogger::warn("Создание канала временно отключено - требуется реализация через HTTP клиент");
                let new_channel_id = 0;

                if new_channel_id != 0 {
                    aniapi::logger::PluginLogger::info(&format!("Создан временный канал '{}' (ID: {}) для пользователя {}", channel_name, new_channel_id, user_id));
                    
                    // Сохраняем маппинг пользователь -> канал
                    let mut map = user_channel_map.lock().unwrap();
                    map.insert(user_id, new_channel_id);
                } else {
                    aniapi::logger::PluginLogger::error(&format!("Не удалось создать временный канал '{}' для пользователя {}", channel_name, user_id));
                }
            }
        }

        // Проверяем, покинул ли пользователь временный канал
        let old_channel_id = event.old_state.as_ref()
            .and_then(|s| s.get("channel_id"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok());

        if let Some(old_ch_id) = old_channel_id {
            let mut map = user_channel_map.lock().unwrap();
            if let Some(&temp_channel_id) = map.get(&user_id) {
                if old_ch_id == temp_channel_id {
                    // Пользователь покинул свой временный канал
                    aniapi::logger::PluginLogger::info(&format!("Пользователь {} покинул временный канал {}", user_id, temp_channel_id));
                    
                    if config.auto_delete_empty {
                        // TODO: Удалить канал через HTTP клиент
                        aniapi::logger::PluginLogger::warn("Удаление канала временно отключено - требуется реализация через HTTP клиент");
                        let deleted = false;
                        if deleted {
                            aniapi::logger::PluginLogger::info(&format!("Удален временный канал {} (пустой)", temp_channel_id));
                        } else {
                            aniapi::logger::PluginLogger::warn(&format!("Не удалось удалить временный канал {}", temp_channel_id));
                        }
                        
                        map.remove(&user_id);
                    }
                }
            }
        }
    }
}

/// Обработка команды настройки канала-триггера
fn handle_setup(interaction: &Value) -> Option<String> {
    aniapi::logger::PluginLogger::info("Обработка команды voicetemp-setup");
    
    // Извлекаем опции из взаимодействия
    let options = interaction.get("options")?.as_array()?;
    
    let mut trigger_channel_id: Option<u64> = None;
    let mut category_id: Option<u64> = None;

    for opt in options {
        let name = opt.get("name")?.as_str()?;
        let value = opt.get("value");
        
        match name {
            "trigger_channel" => {
                if let Some(channel_id) = value.and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()) {
                    trigger_channel_id = Some(channel_id);
                }
            }
            "category" => {
                if let Some(cat_id) = value.and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()) {
                    category_id = Some(cat_id);
                }
            }
            _ => {}
        }
    }

    if let Some(trigger_id) = trigger_channel_id {
        let mut response = format!(
            "✅ Канал-триггер настроен!\n\
            **Канал-триггер:** <#{}>\n",
            trigger_id
        );
        
        if let Some(cat_id) = category_id {
            response.push_str(&format!("**Категория:** <#{}>\n", cat_id));
        } else {
            response.push_str("**Категория:** Не указана (каналы будут создаваться в той же категории)\n");
        }
        
        response.push_str("\n💡 Теперь при входе пользователя в канал-триггер будет автоматически создаваться временный голосовой канал.");
        
        // Сохраняем настройки в конфигурацию
        let mut config = load_config().unwrap_or_default();
        config.trigger_channel_id = Some(trigger_id);
        config.category_id = category_id;
        
        // TODO: Сохранить конфигурацию через PluginContext
        // Пока просто логируем
        aniapi::logger::PluginLogger::info("Конфигурация обновлена (требуется сохранение через PluginContext)");
        
        Some(response)
    } else {
        Some("❌ Ошибка: не указан канал-триггер".to_string())
    }
}

/// Обработка команды создания временного канала
fn handle_create(interaction: &Value) -> Option<String> {
    aniapi::logger::PluginLogger::info("Обработка команды voicetemp-create");
    
    let options = interaction.get("options")?.as_array()?;
    
    let mut channel_name: Option<String> = None;
    let mut user_limit: Option<u64> = None;

    for opt in options {
        let name = opt.get("name")?.as_str()?;
        let value = opt.get("value");
        
        match name {
            "name" => {
                if let Some(name_str) = value.and_then(|v| v.as_str()) {
                    channel_name = Some(name_str.to_string());
                }
            }
            "limit" => {
                if let Some(limit) = value.and_then(|v| v.as_u64()) {
                    user_limit = Some(limit);
                }
            }
            _ => {}
        }
    }

    let name = channel_name.unwrap_or_else(|| "Временный канал".to_string());
    let limit_text = if let Some(limit) = user_limit {
        if limit == 0 {
            "без лимита".to_string()
        } else {
            format!("лимит: {}", limit)
        }
    } else {
        "без лимита".to_string()
    };

    Some(format!(
        "✅ Запрос на создание временного канала получен!\n\
        **Название:** {}\n\
        **Лимит:** {}\n\n\
        ⚠️ Примечание: Для полноценной работы требуется интеграция с событиями Discord (voice_state_update).",
        name, limit_text
    ))
}

/// Обработка команды удаления временного канала
fn handle_delete(_interaction: &Value, _user_channel_map: Arc<Mutex<HashMap<u64, u64>>>) -> Option<String> {
    aniapi::logger::PluginLogger::info("Обработка команды voicetemp-delete");
    
    // TODO: Реализовать удаление канала через Discord API
    
    Some("✅ Запрос на удаление временного канала получен!\n⚠️ Примечание: Для полноценной работы требуется интеграция с событиями Discord.".to_string())
}

/// Обработка команды просмотра информации
fn handle_info(user_channel_map: Arc<Mutex<HashMap<u64, u64>>>) -> Option<String> {
    aniapi::logger::PluginLogger::info("Обработка команды voicetemp-info");
    
    let config = load_config().unwrap_or_default();
    
    let mut response = format!(
        "📋 **Информация о плагине VoiceTemp**\n\
        **Плагин:** {}\n\
        **Версия:** {}\n\n",
        crate::constants::PLUGIN_NAME, crate::constants::PLUGIN_VERSION
    );
    
    if let Some(trigger_id) = config.trigger_channel_id {
        response.push_str(&format!("**Канал-триггер:** <#{}>\n", trigger_id));
    } else {
        response.push_str("**Канал-триггер:** Не настроен\n");
    }
    
    if let Some(cat_id) = config.category_id {
        response.push_str(&format!("**Категория:** <#{}>\n", cat_id));
    } else {
        response.push_str("**Категория:** Не указана\n");
    }
    
    response.push_str(&format!("**Шаблон имени:** {}\n", config.channel_name_template));
    response.push_str(&format!("**Автоудаление пустых:** {}\n", if config.auto_delete_empty { "Да" } else { "Нет" }));
    
    let map = user_channel_map.lock().unwrap();
    response.push_str(&format!("**Активных временных каналов:** {}", map.len()));
    
    Some(response)
}

/// Загружает конфигурацию плагина
/// TODO: Использовать PluginContext для загрузки конфигурации
fn load_config() -> Result<VoiceTempConfig, String> {
    // Пока возвращаем дефолтные значения
    // В реальности конфигурация должна быть загружена через PluginContext
    Ok(VoiceTempConfig::default())
}
