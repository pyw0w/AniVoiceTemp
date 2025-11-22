use serde::{Deserialize, Serialize};

/// Конфигурация плагина VoiceTemp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceTempConfig {
    /// ID канала-триггера (голосовой канал, при входе в который создается временный канал)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_channel_id: Option<u64>,
    /// ID категории, в которой будут создаваться временные каналы
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<u64>,
    /// Шаблон имени канала (по умолчанию "{user} временный")
    #[serde(default = "default_channel_name_template")]
    pub channel_name_template: String,
    /// Автоматически удалять пустые каналы
    #[serde(default = "default_auto_delete_empty")]
    pub auto_delete_empty: bool,
}

fn default_channel_name_template() -> String {
    "{user} временный".to_string()
}

fn default_auto_delete_empty() -> bool {
    true
}

impl Default for VoiceTempConfig {
    fn default() -> Self {
        Self {
            trigger_channel_id: None,
            category_id: None,
            channel_name_template: default_channel_name_template(),
            auto_delete_empty: default_auto_delete_empty(),
        }
    }
}

impl VoiceTempConfig {
    /// Загружает конфигурацию из JSON строки
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Сохраняет конфигурацию в JSON строку
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

