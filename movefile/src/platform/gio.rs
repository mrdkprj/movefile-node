use gio::{
    ffi::{G_FILE_COPY_ALL_METADATA, G_FILE_COPY_OVERWRITE},
    glib::translate::ToGlibPtr,
    prelude::{CancellableExt, FileExt},
    Cancellable,
};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};

static UUID: AtomicU32 = AtomicU32::new(0);
static CANCELABLES: Lazy<Mutex<HashMap<u32, Cancellable>>> = Lazy::new(|| Mutex::new(HashMap::new()));

struct BulkProgressData<'a> {
    callback: Option<&'a mut dyn FnMut(i64, i64)>,
    total: i64,
    processed: i64,
    in_process: bool,
}

pub(crate) fn reserve_cancellable() -> u32 {
    let id = UUID.fetch_add(1, Ordering::Relaxed);

    let mut tokens = CANCELABLES.lock().unwrap();
    let token = Cancellable::new();
    tokens.insert(id, token);

    id
}

pub(crate) fn mv(source_file: String, dest_file: String, callback: Option<&mut dyn FnMut(i64, i64)>, cancellable: Option<u32>) -> Result<(), String> {
    cancellable_move(source_file, dest_file, callback, cancellable)
}

fn cancellable_move(source_file: String, dest_file: String, callback: Option<&mut dyn FnMut(i64, i64)>, id: Option<u32>) -> Result<(), String> {
    let source = gio::File::for_parse_name(&source_file);
    let dest = gio::File::for_parse_name(&dest_file);

    let cancellable_token = if let Some(id) = id {
        {
            let tokens = CANCELABLES.lock().unwrap();
            tokens.get(&id).unwrap().clone()
        }
    } else {
        Cancellable::new()
    };

    match source.copy(&dest, gio::FileCopyFlags::from_bits(G_FILE_COPY_OVERWRITE | G_FILE_COPY_ALL_METADATA).unwrap(), Some(&cancellable_token), callback) {
        Ok(_) => {
            source.delete(Cancellable::NONE).map_err(|e| e.message().to_string())?;
            if let Ok(mut tokens) = CANCELABLES.try_lock() {
                if let Some(id) = id {
                    if tokens.get(&id).is_some() {
                        tokens.remove(&id);
                    }
                }
            }
        }
        Err(e) => {
            if dest.query_exists(Cancellable::NONE) {
                dest.delete(Cancellable::NONE).map_err(|e| e.message().to_string())?;
            }
            if !e.matches(gio::IOErrorEnum::Cancelled) {
                return Err(e.message().to_string());
            }
        }
    }

    Ok(())
}

pub fn mv_bulk(source_files: Vec<String>, dest_dir: String, callback: Option<&mut dyn FnMut(i64, i64)>, cancel_id: Option<u32>) -> Result<(), String> {
    let sources: Vec<gio::File> = source_files.iter().map(|f| gio::File::for_parse_name(&f)).collect();
    let dest_dir_path = std::path::Path::new(&dest_dir);
    if dest_dir_path.is_file() {
        return Err("Destination is file".to_string());
    }

    let mut total: i64 = 0;
    let mut dest_files: Vec<gio::File> = Vec::new();

    for source_file in source_files {
        let metadata = std::fs::metadata(&source_file).unwrap();
        total += metadata.len() as i64;
        let path = std::path::Path::new(&source_file);
        let name = path.file_name().unwrap();
        let dest_file = dest_dir_path.join(name);
        dest_files.push(gio::File::for_parse_name(dest_file.to_string_lossy().as_ref()));
    }

    let cancellable_token = if let Some(id) = cancel_id {
        {
            let tokens = CANCELABLES.lock().unwrap();
            tokens.get(&id).unwrap().clone()
        }
    } else {
        Cancellable::new()
    };

    let data = Box::into_raw(Box::new(BulkProgressData {
        callback,
        total,
        processed: 0,
        in_process: true,
    }));

    let flags = gio::FileCopyFlags::from_bits(G_FILE_COPY_OVERWRITE | G_FILE_COPY_ALL_METADATA).unwrap().bits();

    for (i, source) in sources.iter().enumerate() {
        let dest = dest_files.get(i).unwrap();
        let mut error = std::ptr::null_mut();

        let is_ok = unsafe { gio::ffi::g_file_copy(source.to_glib_none().0, dest.to_glib_none().0, flags, cancellable_token.to_glib_none().0, Some(progress_callback), data as _, &mut error) };
        debug_assert_eq!(is_ok == gio::glib::ffi::GFALSE, !error.is_null());

        let result: Result<(), gio::glib::Error> = if error.is_null() {
            Ok(())
        } else {
            Err(unsafe { gio::glib::translate::from_glib_full(error) })
        };

        match result {
            Ok(_) => source.delete(Cancellable::NONE).map_err(|e| e.message().to_string())?,
            Err(e) => {
                if dest.query_exists(Cancellable::NONE) {
                    dest.delete(Cancellable::NONE).map_err(|e| e.message().to_string())?;
                }

                if !e.matches(gio::IOErrorEnum::Cancelled) {
                    return Err(e.message().to_string());
                }

                if e.matches(gio::IOErrorEnum::Cancelled) {
                    break;
                }
            }
        }
    }

    if let Ok(mut tokens) = CANCELABLES.try_lock() {
        if let Some(id) = cancel_id {
            if tokens.get(&id).is_some() {
                tokens.remove(&id);
            }
        }
    }

    Ok(())
}

unsafe extern "C" fn progress_callback(current_num_bytes: i64, total_num_bytes: i64, userdata: gio::glib::ffi::gpointer) {
    let item_data_ptr = userdata as *mut BulkProgressData;
    let data = unsafe { &mut *item_data_ptr };

    if total_num_bytes == current_num_bytes {
        data.in_process = !data.in_process;
    }

    if data.in_process {
        let current = data.processed + current_num_bytes;

        if total_num_bytes == current_num_bytes {
            data.processed = data.processed + total_num_bytes;
        }

        if let Some(callback) = data.callback.as_mut() {
            callback(current, data.total);
        }
    }
}

pub(crate) fn cancel(id: u32) -> bool {
    if let Ok(mut tokens) = CANCELABLES.try_lock() {
        if let Some(token) = tokens.get(&id) {
            token.cancel();
            tokens.remove(&id);
            return true;
        }
    }

    false
}

pub(crate) fn trash(file: String) -> Result<(), String> {
    let file = gio::File::for_parse_name(&file);
    file.trash(Cancellable::NONE).map_err(|e| e.message().to_string())
}

/*
  fn copy(
        &self,
        destination: &impl IsA<File>,
        flags: FileCopyFlags,
        cancellable: Option<&impl IsA<Cancellable>>,
        progress_callback: Option<&mut dyn (FnMut(i64, i64))>,
    ) -> Result<(), glib::Error> {
        let progress_callback_data: Option<&mut dyn (FnMut(i64, i64))> = progress_callback;
        unsafe extern "C" fn progress_callback_func(
            current_num_bytes: i64,
            total_num_bytes: i64,
            data: glib::ffi::gpointer,
        ) {
            let callback = data as *mut Option<&mut dyn (FnMut(i64, i64))>;
            if let Some(ref mut callback) = *callback {
                callback(current_num_bytes, total_num_bytes)
            } else {
                panic!("cannot get closure...")
            }
        }
        let progress_callback = if progress_callback_data.is_some() {
            Some(progress_callback_func as _)
        } else {
            None
        };
        let super_callback0: &Option<&mut dyn (FnMut(i64, i64))> = &progress_callback_data;
        unsafe {
            let mut error = std::ptr::null_mut();
            let is_ok = ffi::g_file_copy(
                self.as_ref().to_glib_none().0,
                destination.as_ref().to_glib_none().0,
                flags.into_glib(),
                cancellable.map(|p| p.as_ref()).to_glib_none().0,
                progress_callback,
                super_callback0 as *const _ as *mut _,
                &mut error,
            );
            debug_assert_eq!(is_ok == glib::ffi::GFALSE, !error.is_null());
            if error.is_null() {
                Ok(())
            } else {
                Err(from_glib_full(error))
            }
        }
    }
*/
