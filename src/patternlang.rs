use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use std::sync::Arc;

use crate::ffi;

#[derive(Debug, Clone)]
pub struct PlField {
    pub path: String,
    pub type_name: String,
    pub value: String,
    pub offset: u64,
    pub size: u64,
}

pub struct PlResult {
    pub fields: Vec<PlField>,
    pub error: Option<String>,
}

pub struct PatternLang {
    raw: *mut ffi::PlWrapper,
    handle: Arc<Handle>,
}

struct Handle(*mut ffi::PlWrapper);

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}
unsafe impl Send for PatternLang {}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { ffi::pl_destroy(self.0) };
    }
}

#[derive(Clone)]
pub struct Aborter {
    handle: Arc<Handle>,
}

impl Aborter {
    pub fn abort(&self) {
        unsafe { ffi::pl_abort(self.handle.0) };
    }
}

impl Default for PatternLang {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternLang {
    pub fn new() -> Self {
        let raw = unsafe { ffi::pl_create() };
        assert!(!raw.is_null(), "failed to create FFI PatternLanguage");
        Self {
            raw,
            handle: Arc::new(Handle(raw)),
        }
    }

    pub fn aborter(&self) -> Aborter {
        Aborter {
            handle: self.handle.clone(),
        }
    }

    pub fn set_max_eval_depth(&mut self, depth: u64) {
        unsafe { ffi::pl_set_max_eval_depth(self.raw, depth) };
    }

    pub fn add_include_path(&mut self, path: &Path) {
        unsafe { ffi::pl_add_include_path(self.raw, path_to_cstring(path).as_ptr()) };
    }

    pub fn run(&mut self, pattern_path: &Path, data_path: &Path) -> Result<PlResult, String> {
        let pattern_c = path_to_cstring(pattern_path);
        let data_c = path_to_cstring(data_path);
        let result = unsafe { ffi::pl_execute_file(self.raw, pattern_c.as_ptr(), data_c.as_ptr()) };
        let fields = self.collect_fields();

        if result == 0 {
            Ok(PlResult {
                fields,
                error: None,
            })
        } else {
            let message =
                unsafe { to_owned_string(ffi::pl_last_error(self.raw)) }.unwrap_or_default();

            if result == -1 {
                Err(message)
            } else {
                Ok(PlResult {
                    fields,
                    error: Some(message),
                })
            }
        }
    }

    fn collect_fields(&mut self) -> Vec<PlField> {
        let count = unsafe { ffi::pl_field_count(self.raw) };
        let mut fields = Vec::with_capacity(count);

        for index in 0..count {
            let field = unsafe { ffi::pl_field_at(self.raw, index).as_ref() }
                .expect("pl_field_at returns NULL.");
            fields.push(PlField {
                path: unsafe { to_owned_string(field.path) }.unwrap_or_default(),
                type_name: unsafe { to_owned_string(field.type_name) }.unwrap_or_default(),
                value: unsafe { to_owned_string(field.value) }.unwrap_or_default(),
                offset: field.offset,
                size: field.size,
            });
        }

        fields
    }
}

fn path_to_cstring(path: &Path) -> CString {
    CString::new(path.to_string_lossy().as_bytes()).expect("path must not contain NULL")
}

unsafe fn to_owned_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}
