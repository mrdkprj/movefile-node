use crate::{ClipboardData, FileAttribute, Operation, Volume};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    ffi::c_void,
    fs,
    os::windows::ffi::OsStrExt,
    path::Path,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};
use windows::{
    core::{Error, HRESULT, PCWSTR},
    Win32::{
        Foundation::{GlobalFree, HANDLE, HWND, MAX_PATH},
        Globalization::lstrlenW,
        Storage::FileSystem::{
            DeleteFileW, FindClose, FindExInfoBasic, FindExSearchNameMatch, FindFirstFileExW, FindFirstVolumeW, FindNextVolumeW, FindVolumeClose, GetFileAttributesW, GetVolumeInformationW,
            GetVolumePathNamesForVolumeNameW, MoveFileExW, MoveFileWithProgressW, FILE_ATTRIBUTE_DEVICE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_READONLY,
            FILE_ATTRIBUTE_SYSTEM, FIND_FIRST_EX_FLAGS, LPPROGRESS_ROUTINE_CALLBACK_REASON, MOVEFILE_COPY_ALLOWED, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, WIN32_FIND_DATAW,
        },
        System::{
            Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL},
            DataExchange::{CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW, SetClipboardData},
            Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
            Ole::{OleGetClipboard, DROPEFFECT_COPY, DROPEFFECT_MOVE, DROPEFFECT_NONE},
        },
        UI::Shell::{DragQueryFileW, FileOperation, IFileOperation, IShellItem, SHCreateItemFromParsingName, CFSTR_PREFERREDDROPEFFECT, DROPFILES, FOF_ALLOWUNDO, HDROP},
    },
};

static UUID: AtomicU32 = AtomicU32::new(0);
static CANCELLABLES: Lazy<Mutex<HashMap<u32, u32>>> = Lazy::new(|| Mutex::new(HashMap::new()));
const PROGRESS_CANCEL: u32 = 1;
const FILE_NO_EXISTS: u32 = 4294967295;
const CANCEL_ERROR_CODE: HRESULT = HRESULT::from_win32(1235);

pub(crate) fn list_volumes() -> Result<Vec<Volume>, String> {
    let mut volumes: Vec<Volume> = Vec::new();

    let mut volume_name = vec![0u16; MAX_PATH as usize];
    let handle = unsafe { FindFirstVolumeW(&mut volume_name).map_err(|e| e.message()) }?;

    loop {
        let mut drive_paths = vec![0u16; 261];
        let mut len = 0;
        unsafe { GetVolumePathNamesForVolumeNameW(PCWSTR::from_raw(volume_name.as_ptr()), Some(&mut drive_paths), &mut len).map_err(|e| e.message()) }?;

        let mount_point = decode_wide(&drive_paths);

        let mut volume_label_ptr = vec![0u16; 261];
        unsafe { GetVolumeInformationW(PCWSTR(volume_name.as_ptr()), Some(&mut volume_label_ptr), None, None, None, None).map_err(|e| e.message()) }?;

        let volume_label = decode_wide(&volume_label_ptr);

        volumes.push(Volume {
            mount_point,
            volume_label,
        });

        volume_name = vec![0u16; MAX_PATH as usize];
        let next = unsafe { FindNextVolumeW(handle, &mut volume_name) };
        if next.is_err() {
            break;
        }
    }

    unsafe { FindVolumeClose(handle).map_err(|e| e.message()) }?;

    Ok(volumes)
}

pub(crate) fn get_file_attribute(file_path: &str, _retry_count: u8) -> Result<FileAttribute, String> {
    let wide = encode_wide(prefixed(file_path));
    let path = PCWSTR::from_raw(wide.as_ptr());

    let mut data: WIN32_FIND_DATAW = unsafe { std::mem::zeroed() };
    let handle = unsafe { FindFirstFileExW(path, FindExInfoBasic, &mut data as *mut _ as _, FindExSearchNameMatch, None, FIND_FIRST_EX_FLAGS(0)).map_err(|e| e.message()) }?;
    let attributes = data.dwFileAttributes;
    unsafe { FindClose(handle).map_err(|e| e.message()) }?;

    Ok(FileAttribute {
        directory: attributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0,
        read_only: attributes & FILE_ATTRIBUTE_READONLY.0 != 0,
        hidden: attributes & FILE_ATTRIBUTE_HIDDEN.0 != 0,
        system: attributes & FILE_ATTRIBUTE_SYSTEM.0 != 0,
        device: attributes & FILE_ATTRIBUTE_DEVICE.0 != 0,
        ctime: to_msecs(data.ftCreationTime.dwLowDateTime, data.ftCreationTime.dwHighDateTime),
        mtime: to_msecs(data.ftLastWriteTime.dwLowDateTime, data.ftLastWriteTime.dwHighDateTime),
        atime: to_msecs(data.ftLastAccessTime.dwLowDateTime, data.ftLastAccessTime.dwHighDateTime),
        size: (data.nFileSizeLow as u64) | ((data.nFileSizeHigh as u64) << 32),
    })
}

fn to_msecs(low: u32, high: u32) -> f64 {
    let windows_epoch = 11644473600000.0; // FILETIME epoch (1601-01-01) to Unix epoch (1970-01-01) in milliseconds
    let ticks = ((high as u64) << 32) | low as u64;
    let milliseconds = ticks as f64 / 10_000.0; // FILETIME is in 100-nanosecond intervals

    milliseconds - windows_epoch
}

const CF_HDROP: u32 = 15;
pub(crate) fn read_urls_from_clipboard(window_handle: isize) -> Result<ClipboardData, String> {
    let mut data = ClipboardData {
        operation: Operation::None,
        urls: Vec::new(),
    };

    if unsafe { IsClipboardFormatAvailable(CF_HDROP).is_ok() } {
        let mut urls = Vec::new();
        let operation = get_preferred_drop_effect();

        unsafe { OpenClipboard(HWND(window_handle as _)).map_err(|e| e.message()) }?;

        if let Ok(handle) = unsafe { GetClipboardData(CF_HDROP) } {
            let hdrop = HDROP(handle.0);
            let count = unsafe { DragQueryFileW(hdrop, 0xFFFFFFFF, None) };
            for i in 0..count {
                // Get the length of the file path
                let len = unsafe { DragQueryFileW(hdrop, i, None) } as usize;

                // Create a buffer to hold the file path
                let mut buffer = vec![0u16; len + 1];

                // Retrieve the file path
                unsafe { DragQueryFileW(hdrop, i, Some(&mut buffer)) };

                urls.push(decode_wide(&buffer));
            }
        }

        unsafe { CloseClipboard().map_err(|e| e.message()) }?;

        data.operation = operation;
        data.urls = urls;
    }

    Ok(data)
}

pub(crate) fn write_urls_to_clipboard(window_handle: isize, paths: &[String], operation: Operation) -> Result<(), String> {
    unsafe {
        let mut file_list = paths.join("\0");
        // Append null to the last file
        file_list.push('\0');
        // Append null to the last
        file_list.push('\0');

        let mut total_size = std::mem::size_of::<u32>();
        for path in paths {
            let path_wide: Vec<u16> = encode_wide(path);
            total_size += path_wide.len() * 2;
        }
        total_size += std::mem::size_of::<DROPFILES>();
        // Double null terminator
        total_size += 2;

        // Calculate the size needed for the DROPFILES structure and file list
        let dropfiles_size = std::mem::size_of::<DROPFILES>();
        let file_list_size = file_list.len() * std::mem::size_of::<u16>();

        let hglobal = GlobalAlloc(GMEM_MOVEABLE, total_size).map_err(|e| e.message())?;

        // Lock the memory to write to it
        let ptr = GlobalLock(hglobal) as *mut u8;
        if ptr.is_null() {
            GlobalFree(hglobal).map_err(|e| e.message())?;
            return Err("Failed to lock memory".to_string());
        }

        let dropfiles = DROPFILES {
            pFiles: dropfiles_size as u32,
            pt: Default::default(),
            fNC: false.into(),
            fWide: true.into(),
        };
        std::ptr::copy_nonoverlapping(&dropfiles as *const _ as *const u8, ptr, dropfiles_size);

        // Write the file list as wide characters (UTF-16)
        let wide_file_list: Vec<u16> = file_list.encode_utf16().collect();
        std::ptr::copy_nonoverlapping(wide_file_list.as_ptr() as *const u8, ptr.add(dropfiles_size), file_list_size);

        let _ = GlobalUnlock(hglobal);

        OpenClipboard(HWND(window_handle as _)).map_err(|e| e.message())?;
        EmptyClipboard().map_err(|e| e.message())?;

        if SetClipboardData(CF_HDROP, HANDLE(hglobal.0)).is_err() {
            CloseClipboard().map_err(|e| e.message())?;
            GlobalFree(hglobal).map_err(|e| e.message())?;
            return Err("Failed to write clipboard".to_string());
        }

        if GlobalFree(hglobal).is_err() {
            CloseClipboard().map_err(|e| e.message())?;
            return Err("Failed to free memory".to_string());
        }

        let operation_value = match operation {
            Operation::Copy => DROPEFFECT_COPY.0,
            Operation::Move => DROPEFFECT_MOVE.0,
            Operation::None => DROPEFFECT_NONE.0,
        };

        let hglobal_operation = GlobalAlloc(GMEM_MOVEABLE, std::mem::size_of::<u32>()).map_err(|e| e.message())?;

        let ptr_operation = GlobalLock(hglobal_operation) as *mut u32;
        if ptr_operation.is_null() {
            GlobalFree(hglobal_operation).map_err(|e| e.message())?;
            return Err("Failed to lock memory".to_string());
        }

        *ptr_operation = operation_value;

        let _ = GlobalUnlock(hglobal_operation);

        let custom_format = RegisterClipboardFormatW(CFSTR_PREFERREDDROPEFFECT);

        if SetClipboardData(custom_format, HANDLE(hglobal_operation.0)).is_err() {
            CloseClipboard().map_err(|e| e.message())?;
            GlobalFree(hglobal_operation).map_err(|e| e.message())?;
            return Err("Failed to write clipboard2".to_string());
        }

        if GlobalFree(hglobal).is_err() {
            CloseClipboard().map_err(|e| e.message())?;
            return Err("Failed to free memory".to_string());
        }

        CloseClipboard().map_err(|e| e.message())?;

        Ok(())
    }
}

fn get_preferred_drop_effect() -> Operation {
    if let Ok(data_object) = unsafe { OleGetClipboard() } {
        let cf_format = unsafe { RegisterClipboardFormatW(CFSTR_PREFERREDDROPEFFECT) };
        if cf_format == 0 {
            return Operation::None;
        }

        let format_etc = FORMATETC {
            cfFormat: cf_format as u16,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0 as u32,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        };

        if let Ok(medium) = unsafe { data_object.GetData(&format_etc) } {
            let h_global = unsafe { medium.u.hGlobal };

            if !h_global.is_invalid() {
                let ptr = unsafe { GlobalLock(h_global) };
                if !ptr.is_null() {
                    let drop_effect = unsafe { *(ptr as *const u32) };
                    let _ = unsafe { GlobalUnlock(h_global) };
                    if (drop_effect & DROPEFFECT_COPY.0) != 0 {
                        return Operation::Copy;
                    }

                    if (drop_effect & DROPEFFECT_MOVE.0) != 0 {
                        return Operation::Move;
                    }

                    return Operation::None;
                }
            }
        }
    }

    Operation::None
}

pub(crate) fn decode_wide(wide: &[u16]) -> String {
    let len = unsafe { lstrlenW(PCWSTR::from_raw(wide.as_ptr())) } as usize;
    let w_str_slice = unsafe { std::slice::from_raw_parts(wide.as_ptr(), len) };
    String::from_utf16_lossy(w_str_slice)
}

struct ProgressData<'a> {
    cancel_id: Option<u32>,
    callback: Option<&'a mut dyn FnMut(i64, i64)>,
    total: i64,
    prev: i64,
    processed: i64,
}

pub(crate) fn reserve_cancellable() -> u32 {
    let id = UUID.fetch_add(1, Ordering::Relaxed);

    let mut tokens = CANCELLABLES.lock().unwrap();
    tokens.insert(id, 0);

    id
}

pub(crate) fn mv(source_file: String, dest_file: String, callback: Option<&mut dyn FnMut(i64, i64)>, cancel_id: Option<u32>) -> Result<(), String> {
    let result = inner_mv(source_file, dest_file, callback, cancel_id);
    clean_up(cancel_id);
    result
}

fn inner_mv(source_file: String, dest_file: String, callback: Option<&mut dyn FnMut(i64, i64)>, cancel_id: Option<u32>) -> Result<(), String> {
    let source_wide = encode_wide(prefixed(&source_file));
    let dest_wide = encode_wide(prefixed(&dest_file));
    let source_file_fallback = source_file.clone();
    let dest_file_fallback = dest_file.clone();

    if let Some(callback) = callback {
        let data = Box::into_raw(Box::new(ProgressData {
            cancel_id,
            callback: Some(callback),
            total: 0,
            prev: 0,
            processed: 0,
        }));

        match unsafe {
            MoveFileWithProgressW(
                PCWSTR::from_raw(source_wide.as_ptr()),
                PCWSTR::from_raw(dest_wide.as_ptr()),
                Some(move_progress),
                Some(data as _),
                MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } {
            Ok(_) => move_fallback(source_file_fallback, dest_file_fallback)?,
            Err(e) => handel_error(e, source_file_fallback, false)?,
        };
    } else {
        match unsafe { MoveFileExW(PCWSTR::from_raw(source_wide.as_ptr()), PCWSTR::from_raw(dest_wide.as_ptr()), MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) } {
            Ok(_) => move_fallback(source_file_fallback, dest_file_fallback)?,
            Err(e) => handel_error(e, source_file_fallback, false)?,
        };
    };

    Ok(())
}

pub(crate) fn mv_bulk(source_files: Vec<String>, dest_dir: String, callback: Option<&mut dyn FnMut(i64, i64)>, cancel_id: Option<u32>) -> Result<(), String> {
    let result = inner_mv_bulk(source_files, dest_dir, callback, cancel_id);
    clean_up(cancel_id);
    result
}

fn inner_mv_bulk(source_files: Vec<String>, dest_dir: String, callback: Option<&mut dyn FnMut(i64, i64)>, cancel_id: Option<u32>) -> Result<(), String> {
    let dest_dir_path = Path::new(&dest_dir);
    if dest_dir_path.is_file() {
        return Err("Destination is file".to_string());
    }

    if let Some(callback) = callback {
        let mut total: i64 = 0;
        let mut dest_files: Vec<String> = Vec::new();
        let owned_source_files = source_files.clone();

        for source_file in source_files {
            let metadata = fs::metadata(&source_file).unwrap();
            total += metadata.len() as i64;
            let path = Path::new(&source_file);
            let name = path.file_name().unwrap();
            let dest_file = dest_dir_path.join(name);
            dest_files.push(dest_file.to_string_lossy().to_string());
        }

        let data = Box::into_raw(Box::new(ProgressData {
            cancel_id,
            callback: Some(callback),
            total,
            prev: 0,
            processed: 0,
        }));

        for (i, source_file) in owned_source_files.iter().enumerate() {
            let dest_file = dest_files.get(i).unwrap();
            let source_file_fallback = source_file.clone();
            let dest_file_fallback = dest_file.clone();

            let done = match unsafe {
                let source_wide = encode_wide(prefixed(&source_file));
                let dest_wide = encode_wide(prefixed(&dest_file));
                MoveFileWithProgressW(
                    PCWSTR::from_raw(source_wide.as_ptr()),
                    PCWSTR::from_raw(dest_wide.as_ptr()),
                    Some(move_files_progress),
                    Some(data as _),
                    MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            } {
                Ok(_) => move_fallback(source_file_fallback, dest_file_fallback)?,
                Err(e) => handel_error(e, source_file_fallback, true)?,
            };

            if !done {
                break;
            }
        }
    } else {
        let mut dest_files: Vec<String> = Vec::new();
        let owned_source_files = source_files.clone();

        for source_file in source_files {
            let path = Path::new(&source_file);
            let name = path.file_name().unwrap();
            let dest_file = dest_dir_path.join(name);
            dest_files.push(dest_file.to_string_lossy().to_string());
        }

        for (i, source_file) in owned_source_files.iter().enumerate() {
            let dest_file = dest_files.get(i).unwrap();
            let source_file_fallback = source_file.clone();
            let dest_file_fallback = dest_file.clone();

            let done = match unsafe {
                let source_wide = encode_wide(prefixed(&source_file));
                let dest_wide = encode_wide(prefixed(&dest_file));
                MoveFileExW(PCWSTR::from_raw(source_wide.as_ptr()), PCWSTR::from_raw(dest_wide.as_ptr()), MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)
            } {
                Ok(_) => move_fallback(source_file_fallback, dest_file_fallback)?,
                Err(e) => handel_error(e, source_file_fallback, true)?,
            };

            if !done {
                break;
            }
        }
    }

    Ok(())
}

fn move_fallback(source_file: String, dest_file: String) -> Result<bool, String> {
    let source_wide = encode_wide(&source_file);
    let dest_wide = encode_wide(&dest_file);

    let source_file_exists = unsafe { GetFileAttributesW(PCWSTR::from_raw(source_wide.as_ptr())) } != FILE_NO_EXISTS;
    let dest_file_exists = unsafe { GetFileAttributesW(PCWSTR::from_raw(dest_wide.as_ptr())) } != FILE_NO_EXISTS;
    if source_file_exists && dest_file_exists {
        unsafe { DeleteFileW(PCWSTR::from_raw(source_wide.as_ptr())) }.map_err(|e| e.message())?;
    }

    if source_file_exists && !dest_file_exists {
        return Err("Failed to move file.".to_string());
    }

    Ok(true)
}

fn handel_error(e: Error, file: String, treat_cancel_as_error: bool) -> Result<bool, String> {
    if e.code() != CANCEL_ERROR_CODE {
        return Err(format!("File: {}, Message: {}", file, e.message()));
    }

    if treat_cancel_as_error && e.code() == CANCEL_ERROR_CODE {
        return Ok(false);
    }

    Ok(true)
}

fn clean_up(cancel_id: Option<u32>) {
    if let Ok(mut tokens) = CANCELLABLES.try_lock() {
        if let Some(id) = cancel_id {
            if tokens.get(&id).is_some() {
                tokens.remove(&id);
            }
        }
    }
}

fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

fn prefixed(path: &str) -> String {
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

unsafe extern "system" fn move_progress(
    totalfilesize: i64,
    totalbytestransferred: i64,
    _streamsize: i64,
    _streambytestransferred: i64,
    _dwstreamnumber: u32,
    _dwcallbackreason: LPPROGRESS_ROUTINE_CALLBACK_REASON,
    _hsourcefile: HANDLE,
    _hdestinationfile: HANDLE,
    lpdata: *const c_void,
) -> u32 {
    let data_ptr = lpdata as *mut ProgressData;
    let data = unsafe { &mut *data_ptr };

    if let Some(callback) = data.callback.as_mut() {
        callback(totalfilesize, totalbytestransferred);
    }

    if let Some(cancel_id) = data.cancel_id {
        if let Ok(cancellables) = CANCELLABLES.try_lock() {
            if let Some(cancellable) = cancellables.get(&cancel_id) {
                return *cancellable;
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
    lpdata: *const c_void,
) -> u32 {
    let data_ptr = lpdata as *mut ProgressData;
    let data = unsafe { &mut *data_ptr };

    if totalbytestransferred - data.prev > 0 {
        data.processed += totalbytestransferred - data.prev;
    }
    data.prev = totalbytestransferred;

    if let Some(callback) = data.callback.as_mut() {
        callback(data.total, data.processed);
    }

    if let Some(cancel_id) = data.cancel_id {
        if let Ok(cancellables) = CANCELLABLES.try_lock() {
            if let Some(cancellable) = cancellables.get(&cancel_id) {
                return *cancellable;
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

pub(crate) fn trash(file: String) -> Result<(), String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let op: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL).map_err(|e| e.message())?;
        op.SetOperationFlags(FOF_ALLOWUNDO).map_err(|e| e.message())?;
        let file_wide = encode_wide(prefixed(&file));
        let shell_item: IShellItem = SHCreateItemFromParsingName(PCWSTR::from_raw(file_wide.as_ptr()), None).map_err(|e| e.message())?;
        op.DeleteItem(&shell_item, None).map_err(|e| e.message())?;
        op.PerformOperations().map_err(|e| e.message())?;

        CoUninitialize();
    }

    Ok(())
}
