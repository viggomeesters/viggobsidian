//! Single-worker buffers. The host copies bytes between calls; no callbacks/imports.
use std::sync::Mutex;
static INPUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static OUTPUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());

#[unsafe(no_mangle)]
pub extern "C" fn request_buffer(length: u32) -> u32 {
    if length as usize > crate::MAX_REQUEST_BYTES {
        return 0;
    }
    let mut input = INPUT.lock().unwrap();
    input.resize(length as usize, 0);
    input.as_mut_ptr() as u32
}
#[unsafe(no_mangle)]
pub extern "C" fn validate() -> u32 {
    let result = crate::respond(&INPUT.lock().unwrap());
    let mut output = OUTPUT.lock().unwrap();
    *output = serde_json::to_vec(&result).unwrap();
    output.len() as u32
}
#[unsafe(no_mangle)]
pub extern "C" fn response_buffer() -> u32 {
    OUTPUT.lock().unwrap().as_ptr() as u32
}
