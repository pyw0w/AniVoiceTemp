use std::ffi::{CString, c_char};
use std::sync::Mutex;

// Потокобезопасное хранилище для временных C-строк (максимум 8 одновременных ответов)
pub struct ResponseRing {
    ring: [Option<CString>; 8],
    index: usize,
}

impl ResponseRing {
    pub fn store(&mut self, cstr: CString) -> *const c_char {
        let idx = self.index;
        self.ring[idx] = Some(cstr);
        self.index = (self.index + 1) % 8;
        // Безопасно получаем указатель, так как мы только что сохранили значение
        self.ring[idx].as_ref().map_or(std::ptr::null(), |s| s.as_ptr())
    }
}

// Используем Mutex для потокобезопасного доступа
static RESPONSE_RING: Mutex<ResponseRing> = Mutex::new(ResponseRing {
    ring: [None, None, None, None, None, None, None, None],
    index: 0,
});

/// Безопасно сохраняет C-строку в кольцевом буфере и возвращает указатель
pub fn store_response(cstr: CString) -> *const c_char {
    match RESPONSE_RING.lock() {
        Ok(mut ring) => ring.store(cstr),
        Err(_) => {
            // Если не удалось заблокировать мьютекс, возвращаем null
            // Это может произойти только если мьютекс отравлен (poisoned)
            std::ptr::null()
        }
    }
}

