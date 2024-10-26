use once_cell::sync::Lazy;
use std::os::windows::ffi::OsStrExt;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};
use windows::{
    core::{Result, HRESULT, PCWSTR},
    Win32::{
        Foundation::HANDLE,
        Storage::FileSystem::{MoveFileWithProgressW, LPPROGRESS_ROUTINE_CALLBACK_REASON, MOVEFILE_COPY_ALLOWED, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH},
    },
};

static UUID: AtomicU32 = AtomicU32::new(0);
static CANCEL_TOKEN: Lazy<Mutex<HashMap<u32, u32>>> = Lazy::new(|| Mutex::new(HashMap::new()));
const PROGRESS_CANCEL: u32 = 1;

struct ProgressData<'a> {
    id: u32,
    callback: Option<&'a mut dyn FnMut(i64, i64)>,
}

pub(crate) fn reserve() -> u32 {
    let id = UUID.fetch_add(1, Ordering::Relaxed);

    let mut tokens = CANCEL_TOKEN.lock().unwrap();
    tokens.insert(id, 0);

    id
}

pub(crate) fn mv(id: u32, source_file: String, dest_file: String) -> Result<()> {
    cancellable_move(id, source_file, dest_file, None)
}

pub(crate) fn mv_with_progress(id: u32, source_file: String, dest_file: String, handler: &mut dyn FnMut(i64, i64)) -> Result<()> {
    cancellable_move(id, source_file, dest_file, Some(handler))
}

fn cancellable_move(id: u32, source_file: String, dest_file: String, handler: Option<&mut dyn FnMut(i64, i64)>) -> Result<()> {
    let data = Box::into_raw(Box::new(ProgressData {
        id,
        callback: handler,
    }));

    match unsafe { MoveFileWithProgressW(to_file_path(source_file), to_file_path(dest_file), Some(progress), Some(data as _), MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING) } {
        Ok(_) => {}
        Err(e) => {
            if e.code() != HRESULT::from_win32(1235) {
                panic!("{}", e.message());
            }
        }
    }

    Ok(())
}

pub(crate) fn mv_sync(source_file: String, dest_file: String) -> Result<bool> {
    unsafe { MoveFileWithProgressW(to_file_path(source_file), to_file_path(dest_file), None, None, MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) }?;

    Ok(true)
}

fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

fn to_file_path(orig_file_path: String) -> PCWSTR {
    PCWSTR::from_raw(encode_wide(orig_file_path).as_ptr())
}

unsafe extern "system" fn progress(
    totalfilesize: i64,
    totalbytestransferred: i64,
    _streamsize: i64,
    _streambytestransferred: i64,
    _dwstreamnumber: u32,
    _dwcallbackreason: LPPROGRESS_ROUTINE_CALLBACK_REASON,
    _hsourcefile: HANDLE,
    _hdestinationfile: HANDLE,
    lpdata: *const core::ffi::c_void,
) -> u32 {
    let data_ptr = lpdata as *mut ProgressData;
    let data = unsafe { &mut *data_ptr };

    if let Some(callback) = data.callback.as_mut() {
        callback(totalfilesize, totalbytestransferred);
    }

    let id = data.id;

    if let Ok(mut tokens) = CANCEL_TOKEN.try_lock() {
        if let Some(token) = tokens.get(&id) {
            if *token != 0 {
                let token = tokens.remove(&id).unwrap();
                return token;
            } else {
                return *token;
            }
        }
    }
    0
}

pub(crate) fn cancel(id: u32) -> bool {
    if let Ok(mut tokens) = CANCEL_TOKEN.try_lock() {
        if let Some(token) = tokens.get_mut(&id) {
            *token = PROGRESS_CANCEL;
            return true;
        }
    }

    false
}
