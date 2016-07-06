use std::sync::atomic::{AtomicPtr, Ordering};

use qvariant::*;
use types::*;
use qurl::*;

extern "C" {
    fn dos_qapplication_create();
    fn dos_qapplication_exec();
    fn dos_qapplication_quit();
    fn dos_qapplication_delete();

    fn dos_qqmlapplicationengine_create() -> DosQmlApplicationEngine;
    fn dos_qqmlapplicationengine_load(vptr: DosQmlApplicationEngine, filename: c_str);
    fn dos_qqmlapplicationengine_load_url(vptr: DosQmlApplicationEngine, url: DosQUrl);
    fn dos_qqmlapplicationengine_load_data(vptr: DosQmlApplicationEngine, data: c_str);
    // fn dos_qqmlapplicationengine_add_import_path(vptr: *mut DosQQmlApplicationEngine, const char *path);
    fn dos_qqmlapplicationengine_context(vptr: DosQmlApplicationEngine) -> DosQQmlContext;
    fn dos_qqmlapplicationengine_delete(vptr: DosQmlApplicationEngine);

    fn dos_qqmlcontext_setcontextproperty(vptr: DosQQmlContext, name: c_str, value: DosQVariant);

}

/// Provides an entry point for building QML applications from Rust
pub struct QmlEngine {
    ptr: AtomicPtr<WQmlApplicationEngine>,
    stored: Vec<QVariant>,
}

impl QmlEngine {
    /// Creates a QML context of a non-headless application
    pub fn new() -> Self {
        unsafe {
            dos_qapplication_create();
            QmlEngine {
                ptr: AtomicPtr::new(dos_qqmlapplicationengine_create()),
                stored: Vec::new(),
            }
        }
    }

    /// Loads a file as a qml file
    pub fn load_file(&self, path: &str) {
        let path_raw = ::std::env::current_dir().unwrap().join(path);
        let path = if cfg!(windows) {
            format!("file:///{}", path_raw.display())
        } else {
            format!("file://{}", path_raw.display())
        };
        unsafe {
            dos_qqmlapplicationengine_load_url(self.ptr.load(Ordering::Relaxed),
                                               construct_qurl(&path))
        }
    }

    /// Loads a string as a qml file
    pub fn load_data(&self, data: &str) {
        unsafe {
            dos_qqmlapplicationengine_load_data(self.ptr.load(Ordering::Relaxed), stoptr(data))
        }
    }

    /// Launches the application
    pub fn exec(&self) {
        unsafe {
            dos_qapplication_exec();
        }
    }
    /// Closes the application
    pub fn quit(&self) {
        unsafe {
            dos_qapplication_quit();
        }
    }

    /// Sets a property for this QML context
    ///
    /// This variant stores qvariant, so it is removed, only when this QmlEngine is removed.
    pub fn set_and_store_property<T: Into<QVariant>>(&mut self, name: &str, value: T) {
        let val = value.into();
        unsafe {
            let context = dos_qqmlapplicationengine_context(self.ptr.load(Ordering::Relaxed));
            dos_qqmlcontext_setcontextproperty(context, stoptr(name), get_private_variant(&val));
        }
        self.stored.push(val);
    }

    /// Sets a property for this QML context
    pub fn set_property(&self, name: &str, value: &QVariant) {
        unsafe {
            let context = dos_qqmlapplicationengine_context(self.ptr.load(Ordering::Relaxed));
            dos_qqmlcontext_setcontextproperty(context, stoptr(name), get_private_variant(value));
        }
    }
}

use utils::*;

impl Default for QmlEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for QmlEngine {
    fn drop(&mut self) {
        unsafe {
            dos_qapplication_quit();
            dos_qqmlapplicationengine_delete(self.ptr.load(Ordering::Relaxed));
            dos_qapplication_delete();
        }
    }
}