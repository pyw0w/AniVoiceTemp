use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

// Глобальные функции, которые будут предоставлены anicore при загрузке плагина
// Эти функции устанавливаются через set_config_functions

type GetConfigFn = unsafe extern "C" fn(*const c_char) -> *const c_char;
type SaveConfigFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *const c_char;

static CONFIG_FUNCTIONS: Mutex<Option<(GetConfigFn, SaveConfigFn)>> = Mutex::new(None);

/// Устанавливает функции для работы с конфигурацией
/// Вызывается из anicore при загрузке плагина
#[no_mangle]
pub extern "C" fn set_config_functions(get_fn: GetConfigFn, save_fn: SaveConfigFn) {
    let mut funcs = CONFIG_FUNCTIONS.lock().unwrap();
    *funcs = Some((get_fn, save_fn));
}

/// Получает конфигурацию через установленные функции
pub unsafe fn get_config(plugin_name: *const c_char) -> *const c_char {
    let funcs = CONFIG_FUNCTIONS.lock().unwrap();
    if let Some((get_fn, _)) = *funcs {
        get_fn(plugin_name)
    } else {
        std::ptr::null()
    }
}

/// Сохраняет конфигурацию через установленные функции
pub unsafe fn save_config(plugin_name: *const c_char, config_json: *const c_char) -> *const c_char {
    let funcs = CONFIG_FUNCTIONS.lock().unwrap();
    if let Some((_, save_fn)) = *funcs {
        save_fn(plugin_name, config_json)
    } else {
        let error = CString::new("Функции конфигурации не инициализированы").unwrap();
        crate::response_ring::store_response(error)
    }
}

