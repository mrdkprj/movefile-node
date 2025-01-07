/*
   use std::ptr::{null, null_mut};

   let bus = gio_sys::g_bus_get_sync(gio_sys::G_BUS_TYPE_SESSION, null_mut(), null_mut());
   if bus.is_null() {
       return;
   }
   let uris = [uri, null()];
   let args = glib_sys::g_variant_new(
       b"(^ass)\0".as_ptr() as *const _,
       uris.as_ptr(),
       b"\0".as_ptr(),
   );
   let ret = gio_sys::g_dbus_connection_call_sync(
       bus,
       b"org.freedesktop.FileManager1\0".as_ptr() as *const _,
       b"/org/freedesktop/FileManager1\0".as_ptr() as *const _,
       b"org.freedesktop.FileManager1\0".as_ptr() as *const _,
       b"ShowItems\0".as_ptr() as *const _,
       args,
       null(),
       0,
       -1,
       null_mut(),
       null_mut(),
   );
   if !ret.is_null() {
       glib_sys::g_variant_unref(ret);
   }
   gobject_sys::g_object_unref(bus as *mut _);
*/
