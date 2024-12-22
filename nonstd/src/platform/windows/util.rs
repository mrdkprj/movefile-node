use std::os::windows::ffi::OsStrExt;
use windows::{
    core::PCWSTR,
    Win32::{Foundation::MAX_PATH, Globalization::lstrlenW},
};

pub(crate) fn decode_wide(wide: &[u16]) -> String {
    let len = unsafe { lstrlenW(PCWSTR::from_raw(wide.as_ptr())) } as usize;
    let w_str_slice = unsafe { std::slice::from_raw_parts(wide.as_ptr(), len) };
    String::from_utf16_lossy(w_str_slice)
}

pub(crate) fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

pub(crate) fn prefixed(path: &str) -> String {
    if path.len() >= MAX_PATH as usize {
        if path.starts_with("\\\\") {
            format!("\\\\?\\UNC\\{}", &path[2..])
        } else {
            println!("yest long");
            format!("\\\\?\\{}", path)
        }
    } else {
        path.to_string()
    }
}
