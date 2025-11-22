use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Mutex;

// Глобальные функции для работы с HTTP клиентом
// Эти функции устанавливаются через set_http_functions

type CreateChannelFn = unsafe extern "C" fn(u64, *const c_char, u64) -> u64;
type DeleteChannelFn = unsafe extern "C" fn(u64) -> bool;

static HTTP_FUNCTIONS: Mutex<Option<(CreateChannelFn, DeleteChannelFn)>> = Mutex::new(None);

/// Устанавливает функции для работы с HTTP клиентом
/// Вызывается из anicore при загрузке плагина
#[no_mangle]
pub extern "C" fn set_http_functions(create_fn: CreateChannelFn, delete_fn: DeleteChannelFn) {
    let mut funcs = HTTP_FUNCTIONS.lock().unwrap();
    *funcs = Some((create_fn, delete_fn));
}

/// Создает голосовой канал через установленные функции
pub unsafe fn create_voice_channel(guild_id: u64, name: *const c_char, category_id: u64) -> u64 {
    let funcs = HTTP_FUNCTIONS.lock().unwrap();
    if let Some((create_fn, _)) = *funcs {
        create_fn(guild_id, name, category_id)
    } else {
        0
    }
}

/// Удаляет канал через установленные функции
pub unsafe fn delete_channel(channel_id: u64) -> bool {
    let funcs = HTTP_FUNCTIONS.lock().unwrap();
    if let Some((_, delete_fn)) = *funcs {
        delete_fn(channel_id)
    } else {
        false
    }
}

