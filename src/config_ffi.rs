use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::response_ring::store_response;
use crate::plugin_config_api;

// Эти функции вызываются из anicore через VTable
// Они используют глобальные функции, которые будут предоставлены anicore при загрузке плагина

/// FFI функция для получения конфигурации плагина
/// Вызывается из anicore через VTable
/// Использует глобальную функцию из plugin_config_api
pub unsafe extern "C" fn get_plugin_config_ffi(plugin_name: *const c_char) -> *const c_char {
    if plugin_name.is_null() {
        return std::ptr::null();
    }

    // Используем функцию из plugin_config_api, которая будет предоставлена anicore
    plugin_config_api::get_config(plugin_name)
}

/// FFI функция для сохранения конфигурации плагина
/// Вызывается из anicore через VTable
/// Использует глобальную функцию из plugin_config_api
pub unsafe extern "C" fn save_plugin_config_ffi(plugin_name: *const c_char, config_json: *const c_char) -> *const c_char {
    if plugin_name.is_null() || config_json.is_null() {
        let error = CString::new("Ошибка: null указатели").unwrap();
        return store_response(error);
    }

    // Используем функцию из plugin_config_api, которая будет предоставлена anicore
    plugin_config_api::save_config(plugin_name, config_json)
}

