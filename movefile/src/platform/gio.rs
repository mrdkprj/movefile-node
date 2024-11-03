use gio::{
    ffi::{G_FILE_COPY_ALL_METADATA, G_FILE_COPY_OVERWRITE},
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

#[allow(unused_variables)]
pub fn mv_bulk(source_files: Vec<String>, dest_dir: String, callback: Option<&mut dyn FnMut(i64, i64)>, cancel_id: Option<u32>) -> Result<(), String> {
    Ok(())
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
