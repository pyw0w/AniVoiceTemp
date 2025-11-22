use crate::constants::{PLUGIN_NAME, PLUGIN_VERSION};
use crate::handlers::{voicetemp_on_message, voicetemp_on_interaction, voicetemp_on_voice_state_update};
use crate::commands::voicetemp_get_commands;
use crate::config_ffi::{get_plugin_config_ffi, save_plugin_config_ffi};
use aniapi::{BotPluginVTable, PluginVTablePtr};
use aniapi::logger::PluginLogger;
use std::os::raw::c_char;

// C-совместимые функции для имени и версии
unsafe extern "C" fn voicetemp_name() -> *const c_char {
    c"VoiceTemp Plugin".as_ptr()
}

unsafe extern "C" fn voicetemp_version() -> *const c_char {
    c"1.0.0".as_ptr()
}

// VTable
static VOICETEMP_PLUGIN_VTABLE: BotPluginVTable = BotPluginVTable {
    name: voicetemp_name,
    version: voicetemp_version,
    on_message: voicetemp_on_message,
    get_commands: voicetemp_get_commands,
    on_interaction: voicetemp_on_interaction,
    on_voice_state_update: Some(voicetemp_on_voice_state_update),
    get_plugin_config: Some(get_plugin_config_ffi),
    save_plugin_config: Some(save_plugin_config_ffi),
};

#[no_mangle]
pub extern "C" fn _get_vtable() -> PluginVTablePtr {
    PluginLogger::info(&format!("Плагин {} v{} загружается", PLUGIN_NAME, PLUGIN_VERSION));
    PluginVTablePtr::new(&VOICETEMP_PLUGIN_VTABLE)
}

