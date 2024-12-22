mod platform;
use platform::platform_impl;

pub fn reserve_cancellable() -> u32 {
    platform_impl::fs::reserve_cancellable()
}

pub fn mv(source_file: String, dest_file: String, callback: Option<&mut dyn FnMut(i64, i64)>, cancellable: Option<u32>) -> Result<(), String> {
    platform_impl::fs::mv(source_file, dest_file, callback, cancellable)
}

pub fn mv_bulk(source_files: Vec<String>, dest_dir: String, callback: Option<&mut dyn FnMut(i64, i64)>, cancellable: Option<u32>) -> Result<(), String> {
    platform_impl::fs::mv_bulk(source_files, dest_dir, callback, cancellable)
}

pub fn cancel(id: u32) -> bool {
    platform_impl::fs::cancel(id)
}

pub fn trash(file: String) -> Result<(), String> {
    platform_impl::fs::trash(file)
}

#[derive(Debug, Clone)]
pub struct Volume {
    pub mount_point: String,
    pub volume_label: String,
}

pub fn list_volumes() -> Result<Vec<Volume>, String> {
    platform_impl::fs::list_volumes()
}

#[derive(Debug, Clone)]
pub struct FileAttribute {
    pub directory: bool,
    pub read_only: bool,
    pub hidden: bool,
    pub system: bool,
    pub device: bool,
    pub ctime: f64,
    pub mtime: f64,
    pub atime: f64,
    pub size: u64,
}

pub fn get_file_attribute(file_path: &str) -> Result<FileAttribute, String> {
    platform_impl::fs::get_file_attribute(file_path)
}

#[derive(Debug, Clone)]
pub enum Operation {
    None,
    Copy,
    Move,
}

#[derive(Debug, Clone)]
pub struct ClipboardData {
    pub operation: Operation,
    pub urls: Vec<String>,
}

pub fn is_text_availabel() -> bool {
    platform_impl::clipboard::is_text_availabel()
}

pub fn is_uris_available() -> bool {
    platform_impl::clipboard::is_uris_available()
}

pub fn read_text(window_handle: isize) -> Result<String, String> {
    platform_impl::clipboard::read_text(window_handle)
}

pub fn write_text(window_handle: isize, text: String) -> Result<(), String> {
    platform_impl::clipboard::write_text(window_handle, text)
}

pub fn read_urls_from_clipboard(window_handle: isize) -> Result<ClipboardData, String> {
    platform_impl::clipboard::read_uris(window_handle)
}

pub fn write_urls_to_clipboard(window_handle: isize, file_paths: &[String], operation: Operation) -> Result<(), String> {
    platform_impl::clipboard::write_uris(window_handle, file_paths, operation)
}

pub fn open_file_property(window_handle: isize, file_path: String) -> Result<(), String> {
    platform_impl::fs::open_file_property(window_handle, file_path)
}
