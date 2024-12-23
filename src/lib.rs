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
                arr.set(&mut cx, i as u32, obj)?;
            }
            Ok(arr)
        }
        Err(e) => cx.throw_error(e),
    }
}

pub fn get_file_attribute(mut cx: FunctionContext) -> JsResult<JsObject> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    match nonstd::fs::get_file_attribute(&file_path) {
        Ok(att) => {
            let obj = cx.empty_object();
            let a = cx.boolean(att.directory);
            obj.set(&mut cx, "directory", a)?;
            let a = cx.boolean(att.read_only);
            obj.set(&mut cx, "readOnly", a)?;
            let a = cx.boolean(att.hidden);
            obj.set(&mut cx, "hidden", a)?;
            let a = cx.boolean(att.system);
            obj.set(&mut cx, "system", a)?;
            let a = cx.boolean(att.device);
            obj.set(&mut cx, "device", a)?;
            let a = cx.number(att.ctime);
            obj.set(&mut cx, "ctime", a)?;
            let a = cx.number(att.mtime);
            obj.set(&mut cx, "mtime", a)?;
            let a = cx.number(att.atime);
            obj.set(&mut cx, "atime", a)?;
            let a = cx.number(att.size as f64);
            obj.set(&mut cx, "size", a)?;
            Ok(obj)
        }
        Err(e) => cx.throw_error(e),
    }
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

pub fn read_urls_from_clipboard(mut cx: FunctionContext) -> JsResult<JsObject> {
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

pub fn write_urls_to_clipboard(mut cx: FunctionContext) -> JsResult<JsUndefined> {
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

pub fn mv_bulk(cx: FunctionContext) -> JsResult<JsPromise> {
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
            let result = nonstd::fs::mv_bulk(source_files, dest_file, None, id);
            deferred.settle_with(&channel, |mut cx| match result {
                Ok(_) => Ok(cx.undefined()),
                Err(e) => cx.throw_error(e),
            });
        });
    } else {
        async_std::task::spawn(async move {
            let result = nonstd::fs::mv(source_files.first().unwrap().to_string(), dest_file, None, id);
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
            let result = nonstd::fs::mv_bulk(
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
                source_files.first().unwrap().to_string(),
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

pub fn trash(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let source_file = cx.argument::<JsString>(0)?.value(&mut cx);
    let _ = nonstd::fs::trash(source_file);
    Ok(cx.undefined())
}

pub fn open_file_property(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let window_handle = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let source_file = cx.argument::<JsString>(1)?.value(&mut cx);
    let _ = nonstd::fs::open_file_property(window_handle as isize, source_file);
    Ok(cx.undefined())
}

pub fn open_path(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let window_handle = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let source_file = cx.argument::<JsString>(1)?.value(&mut cx);
    let _ = nonstd::fs::open_path(window_handle as isize, source_file);
    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("mv", mv)?;
    cx.export_function("mv_sync", mv_sync)?;
    cx.export_function("cancel", cancel)?;
    cx.export_function("reserve_cancellable", reserve_cancellable)?;
    cx.export_function("trash", trash)?;
    cx.export_function("mv_bulk", mv_bulk)?;
    cx.export_function("list_volumes", list_volumes)?;
    cx.export_function("get_file_attribute", get_file_attribute)?;
    cx.export_function("read_urls_from_clipboard", read_urls_from_clipboard)?;
    cx.export_function("write_urls_to_clipboard", write_urls_to_clipboard)?;
    cx.export_function("read_text", read_text)?;
    cx.export_function("write_text", write_text)?;
    cx.export_function("open_file_property", open_file_property)?;
    cx.export_function("open_path", open_path)?;

    Ok(())
}
