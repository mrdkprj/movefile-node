mod platform;
use platform::platform_impl;

pub fn reserve() -> u32 {
    platform_impl::reserve()
}

pub fn mv(id: u32, source_file: String, dest_file: String) {
    platform_impl::mv(id, source_file, dest_file).unwrap()
}

pub fn mv_with_progress(id: u32, source_file: String, dest_file: String, handler: &mut dyn FnMut(i64, i64)) {
    platform_impl::mv_with_progress(id, source_file, dest_file, handler).unwrap()
}

pub fn mv_sync(source_file: String, dest_file: String) -> bool {
    platform_impl::mv_sync(source_file, dest_file).unwrap()
}

pub fn cancel(id: u32) -> bool {
    platform_impl::cancel(id)
}
