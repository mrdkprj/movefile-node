use once_cell::sync::Lazy;
use std::os::windows::ffi::OsStrExt;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::{IFileOperation, IShellItem, SHCreateItemFromParsingName, FOF_ALLOWUNDO};
use windows::{
    core::{Error, HRESULT, PCWSTR},
    Win32::{
        Foundation::HANDLE,
        Storage::FileSystem::{DeleteFileW, GetFileAttributesW, MoveFileWithProgressW, LPPROGRESS_ROUTINE_CALLBACK_REASON, MOVEFILE_COPY_ALLOWED, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH},
    },
};

static UUID: AtomicU32 = AtomicU32::new(0);
static CANCELLABLES: Lazy<Mutex<HashMap<u32, u32>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static PROGRESS_STATUS: Lazy<Mutex<HashMap<u32, BulkProgressStatus>>> = Lazy::new(|| Mutex::new(HashMap::new()));
const PROGRESS_CANCEL: u32 = 1;
const FILE_NO_EXISTS: u32 = 4294967295;

struct ProgressData<'a> {
    cancel_id: Option<u32>,
    callback: Option<&'a dyn Fn(i64, i64)>,
}

struct BulkProgressStatus {
    total: i64,
    prev: i64,
    processed: i64,
}

struct BulkProgressData<'a> {
    id: u32,
    cancel_id: Option<u32>,
    callback: Option<&'a dyn Fn(i64, i64)>,
}

pub(crate) fn reserve_cancellable() -> u32 {
    let id = UUID.fetch_add(1, Ordering::Relaxed);

    let mut tokens = CANCELLABLES.lock().unwrap();
    tokens.insert(id, 0);

    id
}

pub(crate) fn trash(file: String) -> Result<(), String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let op: IFileOperation = CoCreateInstance(&windows::Win32::UI::Shell::FileOperation, None, CLSCTX_ALL).map_err(|e| e.message())?;
        op.SetOperationFlags(FOF_ALLOWUNDO).map_err(|e| e.message())?;
        let shell_item: IShellItem = SHCreateItemFromParsingName(to_file_path(file), None).map_err(|e| e.message())?;
        op.DeleteItem(&shell_item, None).map_err(|e| e.message())?;
        op.PerformOperations().map_err(|e| e.message())?;

        CoUninitialize();
    }

    Ok(())
}

pub(crate) fn mv(source_file: String, dest_file: String, callback: Option<&dyn Fn(i64, i64)>, cancel_id: Option<u32>) -> Result<(), String> {
    let callback = if let Some(callback) = callback {
        Some(callback)
    } else {
        None
    };

    let data = Box::into_raw(Box::new(ProgressData {
        cancel_id,
        callback,
    }));

    let source_file_fallback = source_file.clone();
    let dest_file_fallback = dest_file.clone();

    match unsafe {
        MoveFileWithProgressW(to_file_path(source_file), to_file_path(dest_file), Some(move_progress), Some(data as _), MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)
    } {
        Ok(_) => move_fallback(source_file_fallback, dest_file_fallback)?,
        Err(e) => handle_move_error(e, false)?,
    };

    Ok(())
}

pub fn mv_bulk(source_files: Vec<String>, dest_dir: String, callback: Option<&dyn Fn(i64, i64)>, cancel_id: Option<u32>) -> Result<(), String> {
    let dest_dir_path = std::path::Path::new(&dest_dir);
    if dest_dir_path.is_file() {
        return Err("Destination is file".to_string());
    }

    let mut total: i64 = 0;
    let mut dest_files: Vec<String> = Vec::new();
    let owned_source_files = source_files.clone();

    for source_file in source_files {
        let metadata = std::fs::metadata(&source_file).unwrap();
        total += metadata.len() as i64;
        let path = std::path::Path::new(&source_file);
        let name = path.file_name().unwrap();
        let dest_file = dest_dir_path.join(name);
        dest_files.push(dest_file.to_string_lossy().to_string());
    }

    let id = UUID.fetch_add(1, Ordering::Relaxed);
    let data = Box::into_raw(Box::new(BulkProgressData {
        id,
        cancel_id,
        callback,
    }));

    {
        let status = BulkProgressStatus {
            total,
            prev: 0,
            processed: 0,
        };

        if let Ok(mut progress_status) = PROGRESS_STATUS.try_lock() {
            progress_status.insert(id, status);
        }
    }

    for (i, source_file) in owned_source_files.iter().enumerate() {
        let dest_file = dest_files.get(i).unwrap();
        let source_file_fallback = source_file.clone();
        let dest_file_fallback = dest_file.clone();

        let done = match unsafe {
            MoveFileWithProgressW(
                to_file_path_str(source_file),
                to_file_path_str(dest_file),
                Some(move_files_progress),
                Some(data as _),
                MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } {
            Ok(_) => move_fallback(source_file_fallback, dest_file_fallback)?,
            Err(e) => handle_move_error(e, true)?,
        };

        if !done {
            break;
        }
    }

    if let Ok(mut progress_status) = PROGRESS_STATUS.try_lock() {
        let _ = progress_status.remove(&id);
    }

    Ok(())
}

fn move_fallback(source_file: String, dest_file: String) -> Result<bool, String> {
    let source_file_exists = unsafe { GetFileAttributesW(to_file_path_str(&source_file)) } != FILE_NO_EXISTS;
    let dest_file_exists = unsafe { GetFileAttributesW(to_file_path_str(&dest_file)) } != FILE_NO_EXISTS;

    if source_file_exists && dest_file_exists {
        unsafe { DeleteFileW(to_file_path_str(&source_file)) }.map_err(|e| e.message())?;
    }

    if source_file_exists && !dest_file_exists {
        return Err("Failed to move file.".to_string());
    }

    Ok(true)
}

fn handle_move_error(e: Error, treat_cancel_as_error: bool) -> Result<bool, String> {
    if e.code() != HRESULT::from_win32(1235) {
        return Err(e.message());
    }

    if treat_cancel_as_error && e.code() != HRESULT::from_win32(1235) {
        return Ok(false);
    }

    Ok(true)
}

fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

fn to_file_path_str(orig_file_path: &str) -> PCWSTR {
    PCWSTR::from_raw(encode_wide(orig_file_path).as_ptr())
}

fn to_file_path(orig_file_path: String) -> PCWSTR {
    PCWSTR::from_raw(encode_wide(orig_file_path).as_ptr())
}

unsafe extern "system" fn move_progress(
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

    if let Some(callback) = &data.callback {
        callback(totalfilesize, totalbytestransferred);
    }

    if let Some(cancel_id) = data.cancel_id {
        if let Ok(mut cancellables) = CANCELLABLES.try_lock() {
            if let Some(cancellable) = cancellables.get(&cancel_id) {
                if *cancellable != 0 {
                    let cancellable = cancellables.remove(&cancel_id).unwrap();
                    return cancellable;
                } else {
                    return *cancellable;
                }
            }
        }
    }

    0
}

unsafe extern "system" fn move_files_progress(
    _totalfilesize: i64,
    totalbytestransferred: i64,
    _streamsize: i64,
    _streambytestransferred: i64,
    _dwstreamnumber: u32,
    _dwcallbackreason: LPPROGRESS_ROUTINE_CALLBACK_REASON,
    _hsourcefile: HANDLE,
    _hdestinationfile: HANDLE,
    lpdata: *const core::ffi::c_void,
) -> u32 {
    let data_ptr = lpdata as *mut BulkProgressData;
    let data = unsafe { &mut *data_ptr };

    if let Ok(mut progress_status) = PROGRESS_STATUS.try_lock() {
        if let Some(progress) = progress_status.get_mut(&data.id) {
            if totalbytestransferred - progress.prev > 0 {
                progress.processed += totalbytestransferred - progress.prev;
            }
            progress.prev = totalbytestransferred;

            if let Some(callback) = data.callback {
                callback(progress.total, progress.processed);
            }
        }
    }

    if let Some(cancel_id) = data.cancel_id {
        if let Ok(mut cancellables) = CANCELLABLES.try_lock() {
            if let Some(cancellable) = cancellables.get(&cancel_id) {
                if *cancellable != 0 {
                    let cancellable = cancellables.remove(&cancel_id).unwrap();
                    return cancellable;
                } else {
                    return *cancellable;
                }
            }
        }
    }

    0
}

pub(crate) fn cancel(id: u32) -> bool {
    if let Ok(mut tokens) = CANCELLABLES.try_lock() {
        if let Some(token) = tokens.get_mut(&id) {
            *token = PROGRESS_CANCEL;
            return true;
        }
    }

    false
}
