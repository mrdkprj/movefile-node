use neon::{
    object::Object,
    prelude::{Context, FunctionContext, ModuleContext},
    result::{JsResult, NeonResult},
    types::{JsBoolean, JsFunction, JsNumber, JsString},
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

pub fn mv(cx: FunctionContext) -> JsResult<JsNumber> {
    if cx.len() > 2 {
        listen_mv(cx)
    } else {
        spawn_mv(cx)
    }
}

fn spawn_mv(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let source_file = cx.argument::<JsString>(0)?.value(&mut cx);
    let dest_file = cx.argument::<JsString>(1)?.value(&mut cx);

    let id = movefile::reserve();

    std::thread::spawn(move || {
        movefile::mv(id, source_file, dest_file);
    });

    Ok(cx.number(id))
}

static CALLBACKS: Lazy<Mutex<HashMap<u32, neon::handle::Root<JsFunction>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn listen_mv(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let source_file = cx.argument::<JsString>(0)?.value(&mut cx);
    let dest_file = cx.argument::<JsString>(1)?.value(&mut cx);
    let callback = cx.argument::<JsFunction>(2)?.root(&mut cx);

    let channel = cx.channel();

    let id = movefile::reserve();

    let mut callbacks = CALLBACKS.lock().unwrap();
    callbacks.insert(id, callback);

    std::thread::spawn(move || {
        movefile::mv_with_progress(id, source_file, dest_file, &mut |a, b| {
            channel.send(move |mut cx| {
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
                    if let Some(callback) = callbacks.get(&id) {
                        callback.clone(&mut cx).into_inner(&mut cx).call(&mut cx, this, args).unwrap();
                        if a == b {
                            let _ = callbacks.remove(&id);
                        }
                    }
                }

                Ok(())
            });
        });
    });

    Ok(cx.number(id))
}

pub fn mv_sync(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let source_file = cx.argument::<JsString>(0)?.value(&mut cx);
    let dest_file = cx.argument::<JsString>(1)?.value(&mut cx);

    let result = movefile::mv_sync(source_file, dest_file);

    Ok(cx.boolean(result))
}

pub fn cancel(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let id = cx.argument::<JsNumber>(0)?.value(&mut cx) as i32;

    if id < 0 {
        panic!("Invalid Id");
    }

    let id = id as u32;
    let result = movefile::cancel(id);

    if let Ok(mut callbacks) = CALLBACKS.try_lock() {
        if callbacks.get(&id).is_some() {
            let _ = callbacks.remove(&id);
        }
    }

    Ok(cx.boolean(result))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("mv", mv)?;
    cx.export_function("mvSync", mv_sync)?;
    cx.export_function("cancel", cancel)?;

    Ok(())
}
