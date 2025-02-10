use neon::{
    object::Object,
    prelude::{Context, FunctionContext, ModuleContext},
    result::{JsResult, NeonResult},
    types::{JsArray, JsBoolean, JsNumber, JsObject, JsString, JsUndefined},
};
use nonstd::{Operation, ThumbButton};

pub fn mv(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let source_file = cx.argument::<JsString>(0)?.value(&mut cx);
    let dest_file = cx.argument::<JsString>(1)?.value(&mut cx);
    #[cfg(target_os = "windows")]
    nonstd::fs::mv(source_file, dest_file).unwrap();
    #[cfg(target_os = "linux")]
    nonstd::fs::mv(source_file, dest_file, None).unwrap();
    Ok(cx.undefined())
}

pub fn mv_all(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let source_files: Vec<String> = cx.argument::<JsArray>(0)?.to_vec(&mut cx)?.iter().map(|a| a.downcast::<JsString, _>(&mut cx).unwrap().value(&mut cx)).collect();
    let dest_file = cx.argument::<JsString>(1)?.value(&mut cx);
    #[cfg(target_os = "windows")]
    nonstd::fs::mv_all(&source_files, dest_file).unwrap();
    #[cfg(target_os = "linux")]
    nonstd::fs::mv_all(&source_files, dest_file, None).unwrap();
    Ok(cx.undefined())
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
    let _ = nonstd::fs::trash(file_path);
    Ok(cx.undefined())
}

pub fn open_file_property(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let _ = nonstd::shell::open_file_property(file_path);
    Ok(cx.undefined())
}

pub fn open_path(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let _ = nonstd::shell::open_path(file_path);
    Ok(cx.undefined())
}

pub fn open_path_with(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let app_path = cx.argument::<JsString>(1)?.value(&mut cx);
    let _ = nonstd::shell::open_path_with(file_path, app_path);
    Ok(cx.undefined())
}

pub fn show_open_with_dialog(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let _ = nonstd::shell::show_open_with_dialog(file_path);
    Ok(cx.undefined())
}

pub fn get_open_with(mut cx: FunctionContext) -> JsResult<JsArray> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let result = nonstd::shell::get_open_with(file_path);
    let arr = cx.empty_array();
    let obj = cx.empty_object();
    let a = cx.string(result[0].path.clone());
    obj.set(&mut cx, "path", a)?;
    arr.set(&mut cx, 0, obj)?;
    Ok(arr)
}

pub fn show_item_in_folder(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let _ = nonstd::shell::show_item_in_folder(file_path);
    Ok(cx.undefined())
}

pub fn get_mime_type(mut cx: FunctionContext) -> JsResult<JsString> {
    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let content_type = nonstd::fs::get_mime_type(file_path);
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

    nonstd::drag_drop::start_drag(files, Operation::Copy).unwrap();

    Ok(cx.undefined())
}

fn register(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let window_handle = cx.argument::<JsNumber>(0)?.value(&mut cx);
    nonstd::shell::set_thumbar_buttons(
        window_handle as _,
        &[
            ThumbButton {
                id: String::from("1"),
                tool_tip: Some("Ok".to_string()),
                icon: std::path::PathBuf::from(r#"play.png"#),
            },
            ThumbButton {
                id: String::from("2"),
                tool_tip: Some("Ok".to_string()),
                icon: std::path::PathBuf::from(r#"play.png"#),
            },
            ThumbButton {
                id: String::from("3"),
                tool_tip: Some("Ok".to_string()),
                icon: std::path::PathBuf::from(r#"play.png"#),
            },
        ],
        |id| {
            println!("{:?}", id);
        },
    )
    .unwrap();

    Ok(cx.undefined())
}

fn copy(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let from = cx.argument::<JsString>(0)?.value(&mut cx);
    let to = cx.argument::<JsString>(1)?.value(&mut cx);
    #[cfg(target_os = "windows")]
    nonstd::fs::copy(from, to).unwrap();
    #[cfg(target_os = "linux")]
    nonstd::fs::copy(from, to, None).unwrap();
    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("mv", mv)?;
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
    cx.export_function("get_open_with", get_open_with)?;
    cx.export_function("show_open_with_dialog", show_open_with_dialog)?;
    cx.export_function("register", register)?;
    cx.export_function("copy", copy)?;

    Ok(())
}
