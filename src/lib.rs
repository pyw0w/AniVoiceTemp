// Главный модуль плагина VoiceTemp
// Новый API на основе trait

mod constants;
mod handlers;
mod commands;
mod config;

use aniapi::{Plugin, PluginContext};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct VoiceTempPlugin {
    name: String,
    version: String,
    // Хранилище маппинга пользователь -> канал для отслеживания временных каналов
    user_channel_map: Arc<Mutex<HashMap<u64, u64>>>,
}

impl VoiceTempPlugin {
    pub fn new() -> Self {
        Self {
            name: constants::PLUGIN_NAME.to_string(),
            version: constants::PLUGIN_VERSION.to_string(),
            user_channel_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Plugin for VoiceTempPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn initialize(&mut self, ctx: PluginContext) -> Result<(), Box<dyn Error>> {
        aniapi::logger::PluginLogger::info(&format!("Плагин {} v{} инициализируется", self.name, self.version));
        
        // Регистрируем обработчик взаимодействий (slash команд)
        let user_channel_map = Arc::clone(&self.user_channel_map);
        ctx.register_interaction_handler(Box::new(move |event| {
            handlers::handle_interaction(event, Arc::clone(&user_channel_map))
        }));
        
        // Регистрируем обработчик событий голосовых каналов
        let user_channel_map = Arc::clone(&self.user_channel_map);
        // Получаем HTTP клиент для использования в обработчике
        let http_client = ctx.http_client();
        ctx.register_voice_state_handler(Box::new(move |event| {
            handlers::handle_voice_state_update(event, Arc::clone(&user_channel_map), Arc::clone(&http_client));
            Ok(None)
        }));

        // Регистрируем команды
        let plugin_commands = commands::get_commands();
        ctx.register_commands(plugin_commands)?;
        
        aniapi::logger::PluginLogger::info(&format!("Плагин {} v{} успешно инициализирован", self.name, self.version));
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        aniapi::logger::PluginLogger::info(&format!("Плагин {} v{} завершает работу", self.name, self.version));
        // Очищаем маппинг
        let mut map = self.user_channel_map.lock().unwrap();
        map.clear();
        Ok(())
    }
}

// Экспортируем функцию для инициализации плагина
#[no_mangle]
pub extern "C" fn init_plugin() -> *mut std::ffi::c_void {
    let plugin: Box<dyn aniapi::Plugin> = Box::new(VoiceTempPlugin::new());
    // Оборачиваем Box<dyn Plugin> в еще один Box, чтобы получить *mut Box<dyn Plugin> (обычный указатель)
    // Box::into_raw(Box<dyn Plugin>) возвращает *mut (dyn Plugin) - это fat pointer, который нельзя передать через *mut c_void
    // Поэтому используем Box::into_raw(Box::new(Box<dyn Plugin>)), который возвращает *mut Box<dyn Plugin>
    Box::into_raw(Box::new(plugin)) as *mut std::ffi::c_void
}
