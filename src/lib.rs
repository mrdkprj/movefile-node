use neon::{
    object::Object,
    prelude::{Context, FunctionContext, ModuleContext},
    result::{JsResult, NeonResult},
    types::{JsArray, JsBoolean, JsFunction, JsNumber, JsObject, JsPromise, JsString, JsUndefined},
};
use nonstd::Operation;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};

static UUID: AtomicU32 = AtomicU32::new(0);
static CALLBACKS: Lazy<Mutex<HashMap<u32, neon::handle::Root<JsFunction>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn reserve_cancellable(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(nonstd::fs::reserve_cancellable()))
}

pub fn mv(cx: FunctionContext) -> JsResult<JsPromise> {
    if cx.len() > 2 {
        listen_mv(cx, false)
    } else {
        spawn_mv(cx, false)
    }
}

pub fn mv_all(cx: FunctionContext) -> JsResult<JsPromise> {
    if cx.len() > 2 {
        listen_mv(cx, true)
    } else {
        spawn_mv(cx, true)
    }
}

fn extract_optional_id(cx: &mut FunctionContext, index: usize) -> Option<u32> {
    let opt_id = cx.argument_opt(index);

    if let Some(id_value) = opt_id {
        if id_value.is_a::<JsNumber, _>(cx) {
            let id = id_value.downcast::<JsNumber, _>(cx).unwrap().value(cx) as u32;
            Some(id)
        } else {
            None
        }
    } else {
        None
    }
}

fn spawn_mv(mut cx: FunctionContext, bulk: bool) -> JsResult<JsPromise> {
    let source_files: Vec<String> = if bulk {
        cx.argument::<JsArray>(0)?.to_vec(&mut cx)?.iter().map(|a| a.downcast::<JsString, _>(&mut cx).unwrap().value(&mut cx)).collect()
    } else {
        vec![cx.argument::<JsString>(0)?.value(&mut cx)]
    };
    let dest_file = cx.argument::<JsString>(1)?.value(&mut cx);
    let id = extract_optional_id(&mut cx, 2);

    let (deferred, promise) = cx.promise();
    let channel = cx.channel();

    if bulk {
        async_std::task::spawn(async move {
            let result = nonstd::fs::mv_all(source_files, dest_file, None, id);
            deferred.settle_with(&channel, |mut cx| match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(e),
            });
        });
    } else {
        async_std::task::spawn(async move {
            let result = nonstd::fs::mv(source_files.first().unwrap(), dest_file, None, id);
            deferred.settle_with(&channel, |mut cx| match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(e),
            });
        });
    }

    Ok(promise)
}

fn listen_mv(mut cx: FunctionContext, bulk: bool) -> JsResult<JsPromise> {
    let source_files: Vec<String> = if bulk {
        cx.argument::<neon::types::JsArray>(0)?.to_vec(&mut cx)?.iter().map(|a| a.downcast::<JsString, _>(&mut cx).unwrap().value(&mut cx)).collect()
    } else {
        vec![cx.argument::<JsString>(0)?.value(&mut cx)]
    };
    let dest_file = cx.argument::<JsString>(1)?.value(&mut cx);
    let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);
    let id = extract_optional_id(&mut cx, 3);

    let (deferred, promise) = cx.promise();
    let channel = cx.channel();

    let callback_id = UUID.fetch_add(1, Ordering::Relaxed);
    let mut callbacks = CALLBACKS.lock().unwrap();
    callbacks.insert(callback_id, callback);

    if bulk {
        async_std::task::spawn(async move {
            let result = nonstd::fs::mv_all(
                source_files,
                dest_file,
                Some(&mut |a, b| {
                    channel.clone().send(move |mut cx| {
                        let obj = cx.empty_object();
                        let total = if cfg!(windows) {
                            cx.number(a as f64)
                        } else {
                            cx.number(b as f64)
                        };
                        let transferred = if cfg!(windows) {
                            cx.number(b as f64)
                        } else {
                            cx.number(a as f64)
                        };
                        obj.set(&mut cx, "totalFileSize", total).unwrap();
                        obj.set(&mut cx, "transferred", transferred).unwrap();
                        let this = cx.undefined();
                        let args = vec![obj.upcast()];
                        if let Ok(mut callbacks) = CALLBACKS.try_lock() {
                            if let Some(callback) = callbacks.get(&callback_id) {
                                callback.clone(&mut cx).into_inner(&mut cx).call(&mut cx, this, args).unwrap();
                                if a == b {
                                    let _ = callbacks.remove(&callback_id);
                                }
                            }
                        }

                        Ok(())
                    });
                }),
                id,
            );

            deferred.settle_with(&channel, |mut cx| match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(e),
            });
        });
    } else {
        async_std::task::spawn(async move {
            let result = nonstd::fs::mv(
                source_files.first().unwrap(),
                dest_file,
                Some(&mut |a, b| {
                    channel.clone().send(move |mut cx| {
                        let obj = cx.empty_object();
                        let total = if cfg!(windows) {
                            cx.number(a as f64)
                        } else {
                            cx.number(b as f64)
                        };
                        let transferred = if cfg!(windows) {
                            cx.number(b as f64)
                        } else {
                            cx.number(a as f64)
                        };
                        obj.set(&mut cx, "totalFileSize", total).unwrap();
                        obj.set(&mut cx, "transferred", transferred).unwrap();
                        let this = cx.undefined();
                        let args = vec![obj.upcast()];
                        if let Ok(mut callbacks) = CALLBACKS.try_lock() {
                            if let Some(callback) = callbacks.get(&callback_id) {
                                callback.clone(&mut cx).into_inner(&mut cx).call(&mut cx, this, args).unwrap();
                                if a == b {
                                    let _ = callbacks.remove(&callback_id);
                                }
                            }
                        }

                        Ok(())
                    });
                }),
                id,
            );

            deferred.settle_with(&channel, |mut cx| match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(e),
            });
        });
    }

    Ok(promise)
}

pub fn mv_sync(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let source_file = cx.argument::<JsString>(0)?.value(&mut cx);
    let dest_file = cx.argument::<JsString>(1)?.value(&mut cx);

    let _ = nonstd::fs::mv(source_file, dest_file, None, None);

    Ok(cx.undefined())
}

pub fn cancel(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let id = cx.argument::<JsNumber>(0)?.value(&mut cx) as i32;

    if id < 0 {
        panic!("Invalid Id");
    }

    let id = id as u32;
    let result = nonstd::fs::cancel(id);

    if let Ok(mut callbacks) = CALLBACKS.try_lock() {
        if callbacks.get(&id).is_some() {
            let _ = callbacks.remove(&id);
        }
    }

    Ok(cx.boolean(result))
}

pub fn list_volumes(mut cx: FunctionContext) -> JsResult<JsArray> {
    match nonstd::fs::list_volumes() {
        Ok(volumes) => {
            let arr = cx.empty_array();
            for (i, volume) in volumes.iter().enumerate() {
                let obj = cx.empty_object();
                let a = cx.string(&volume.mount_point);
                obj.set(&mut cx, "mountPoint", a)?;
                let a = cx.string(&volume.volume_label);
                obj.set(&mut cx, "volumeLabel", a)?;
                let a = cx.number(volume.available_units as f64);
                obj.set(&mut cx, "availableUnits", a)?;
                let a = cx.number(volume.total_units as f64);
                obj.set(&mut cx, "totalUnits", a)?;
                arr.set(&mut cx, i as u32, obj)?;
            }
            Ok(arr)
        }
        Err(e) => cx.throw_error(e),
    }
}

pub fn get_file_attribute(mut cx: FunctionContext) -> JsResult<JsObject> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    match nonstd::fs::stat(&file_path) {
        Ok(att) => {
            let attrs = cx.empty_object();

            let a = cx.boolean(att.is_device);
            attrs.set(&mut cx, "isDevice", a)?;
            let a = cx.boolean(att.is_directory);
            attrs.set(&mut cx, "isDirectory", a)?;
            let a = cx.boolean(att.is_file);
            attrs.set(&mut cx, "isFile", a)?;
            let a = cx.boolean(att.is_hidden);
            attrs.set(&mut cx, "isHidden", a)?;
            let a = cx.boolean(att.is_read_only);
            attrs.set(&mut cx, "isReadOnly", a)?;
            let a = cx.boolean(att.is_symbolic_link);
            attrs.set(&mut cx, "isSymbolicLink", a)?;
            let a = cx.boolean(att.is_system);
            attrs.set(&mut cx, "isSystem", a)?;
            let a = cx.number(att.atime_ms);
            attrs.set(&mut cx, "atimeMs", a)?;
            let a = cx.number(att.ctime_ms);
            attrs.set(&mut cx, "ctimeMs", a)?;
            let a = cx.number(att.mtime_ms);
            attrs.set(&mut cx, "mtimeMs", a)?;
            let a = cx.number(att.birthtime_ms);
            attrs.set(&mut cx, "birthtimeMs", a)?;
            let a = cx.number(att.size as f64);
            attrs.set(&mut cx, "size", a)?;

            Ok(attrs)
        }
        Err(e) => cx.throw_error(e),
    }
}

pub fn is_text_available(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    Ok(cx.boolean(nonstd::clipboard::is_text_available()))
}

pub fn read_text(mut cx: FunctionContext) -> JsResult<JsString> {
    let window_handle = cx.argument::<JsNumber>(0)?.value(&mut cx);
    match nonstd::clipboard::read_text(window_handle as isize) {
        Ok(text) => Ok(cx.string(text)),
        Err(e) => cx.throw_error(e),
    }
}

pub fn write_text(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let window_handle = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let text = cx.argument::<JsString>(1)?.value(&mut cx);
    let _ = nonstd::clipboard::write_text(window_handle as isize, text);
    Ok(cx.undefined())
}

pub fn is_uris_available(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    Ok(cx.boolean(nonstd::clipboard::is_uris_available()))
}

pub fn read_uris(mut cx: FunctionContext) -> JsResult<JsObject> {
    let window_handle = cx.argument::<JsNumber>(0)?.value(&mut cx);
    match nonstd::clipboard::read_uris(window_handle as isize) {
        Ok(data) => {
            let obj = cx.empty_object();
            let a = match data.operation {
                Operation::Copy => cx.string(String::from("Copy")),
                Operation::Move => cx.string(String::from("Move")),
                Operation::None => cx.string(String::from("None")),
            };
            obj.set(&mut cx, "operation", a)?;
            let arr = cx.empty_array();
            for (i, url) in data.urls.iter().enumerate() {
                let a = cx.string(url);
                arr.set(&mut cx, i as u32, a)?;
            }
            obj.set(&mut cx, "urls", arr)?;
            Ok(obj)
        }
        Err(e) => cx.throw_error(e),
    }
}

pub fn write_uris(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let window_handle = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let files: Vec<String> = cx.argument::<JsArray>(1)?.to_vec(&mut cx)?.iter().map(|a| a.downcast::<JsString, _>(&mut cx).unwrap().value(&mut cx)).collect();
    let operation_str = cx.argument::<JsString>(2)?.value(&mut cx);
    let operation = match operation_str.as_str() {
        "Copy" => Operation::Copy,
        "Move" => Operation::Move,
        _ => Operation::None,
    };

    nonstd::clipboard::write_uris(window_handle as isize, files.as_slice(), operation).unwrap();

    Ok(cx.undefined())
}

pub fn trash(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let _ = nonstd::shell::trash(file_path);
    Ok(cx.undefined())
}

pub fn open_file_property(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let window_handle = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let file_path = cx.argument::<JsString>(1)?.value(&mut cx);
    let _ = nonstd::shell::open_file_property(window_handle as isize, file_path);
    Ok(cx.undefined())
}

pub fn open_path(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let window_handle = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let file_path = cx.argument::<JsString>(1)?.value(&mut cx);
    let _ = nonstd::shell::open_path(window_handle as isize, file_path);
    Ok(cx.undefined())
}

pub fn open_path_with(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let window_handle = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let file_path = cx.argument::<JsString>(1)?.value(&mut cx);
    let _ = nonstd::shell::open_path_with(window_handle as isize, file_path);
    Ok(cx.undefined())
}

pub fn show_item_in_folder(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let _ = nonstd::shell::show_item_in_folder(file_path);
    Ok(cx.undefined())
}

pub fn get_mime_type(mut cx: FunctionContext) -> JsResult<JsString> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let content_type = nonstd::fs::get_mime_type(file_path).unwrap_or_default();
    Ok(cx.string(content_type))
}

pub fn readdir(mut cx: FunctionContext) -> JsResult<JsArray> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let recursive = cx.argument::<JsBoolean>(1)?.value(&mut cx);
    let with_mime_type = cx.argument::<JsBoolean>(2)?.value(&mut cx);
    match nonstd::fs::readdir(file_path, recursive, with_mime_type) {
        Ok(dirents) => {
            let entries = cx.empty_array();
            for (i, dirent) in dirents.iter().enumerate() {
                let obj = cx.empty_object();
                let a = cx.string(dirent.name.clone());
                obj.set(&mut cx, "name", a)?;
                let a = cx.string(dirent.parent_path.clone());
                obj.set(&mut cx, "parentPath", a)?;
                let a = cx.string(dirent.full_path.clone());
                obj.set(&mut cx, "fullPath", a)?;
                let a = cx.string(dirent.mime_type.clone());
                obj.set(&mut cx, "mimeType", a)?;
                let attrs = cx.empty_object();
                let a = cx.boolean(dirent.attributes.is_device);
                attrs.set(&mut cx, "isDevice", a)?;
                let a = cx.boolean(dirent.attributes.is_directory);
                attrs.set(&mut cx, "isDirectory", a)?;
                let a = cx.boolean(dirent.attributes.is_file);
                attrs.set(&mut cx, "isFile", a)?;
                let a = cx.boolean(dirent.attributes.is_hidden);
                attrs.set(&mut cx, "isHidden", a)?;
                let a = cx.boolean(dirent.attributes.is_read_only);
                attrs.set(&mut cx, "isReadOnly", a)?;
                let a = cx.boolean(dirent.attributes.is_symbolic_link);
                attrs.set(&mut cx, "isSymbolicLink", a)?;
                let a = cx.boolean(dirent.attributes.is_system);
                attrs.set(&mut cx, "isSystem", a)?;
                let a = cx.number(dirent.attributes.atime_ms);
                attrs.set(&mut cx, "atimeMs", a)?;
                let a = cx.number(dirent.attributes.ctime_ms);
                attrs.set(&mut cx, "ctimeMs", a)?;
                let a = cx.number(dirent.attributes.mtime_ms);
                attrs.set(&mut cx, "mtimeMs", a)?;
                let a = cx.number(dirent.attributes.birthtime_ms);
                attrs.set(&mut cx, "birthtimeMs", a)?;
                let a = cx.number(dirent.attributes.size as f64);
                attrs.set(&mut cx, "size", a)?;
                obj.set(&mut cx, "attributes", attrs)?;

                entries.set(&mut cx, i as u32, obj)?;
            }
            Ok(entries)
        }
        Err(e) => cx.throw_error(e),
    }
}

fn start_drag(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let files: Vec<String> = cx.argument::<JsArray>(0)?.to_vec(&mut cx)?.iter().map(|a| a.downcast::<JsString, _>(&mut cx).unwrap().value(&mut cx)).collect();
    #[cfg(target_os = "linux")]
    {
        let window_handle = cx.argument::<JsNumber>(1)?.value(&mut cx);
        nonstd::drag_drop::start_drag(window_handle as _, files, Operation::Copy).unwrap();
    }
    #[cfg(target_os = "windows")]
    {
        nonstd::drag_drop::start_drag(files, Operation::Copy).unwrap();
    }
    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("mv", mv)?;
    cx.export_function("mv_sync", mv_sync)?;
    cx.export_function("cancel", cancel)?;
    cx.export_function("reserve_cancellable", reserve_cancellable)?;
    cx.export_function("trash", trash)?;
    cx.export_function("mv_all", mv_all)?;
    cx.export_function("list_volumes", list_volumes)?;
    cx.export_function("get_file_attribute", get_file_attribute)?;
    cx.export_function("read_uris", read_uris)?;
    cx.export_function("write_uris", write_uris)?;
    cx.export_function("read_text", read_text)?;
    cx.export_function("write_text", write_text)?;
    cx.export_function("open_file_property", open_file_property)?;
    cx.export_function("open_path", open_path)?;
    cx.export_function("open_path_with", open_path_with)?;
    cx.export_function("is_uris_available", is_uris_available)?;
    cx.export_function("is_text_available", is_text_available)?;
    cx.export_function("show_item_in_folder", show_item_in_folder)?;
    cx.export_function("readdir", readdir)?;
    cx.export_function("get_mime_type", get_mime_type)?;
    cx.export_function("start_drag", start_drag)?;

    Ok(())
}
