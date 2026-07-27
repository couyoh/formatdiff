use core::marker;
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct PlWrapper {
    _data: (),
    _marker: marker::PhantomData<(*mut u8, marker::PhantomPinned)>,
}

#[repr(C)]
pub struct PlResult {
    pub path: *const c_char,
    pub type_name: *const c_char,
    pub value: *const c_char,
    pub offset: u64,
    pub size: u64,
}

unsafe extern "C" {
    pub fn pl_create() -> *mut PlWrapper;
    pub fn pl_destroy(rt: *mut PlWrapper);
    pub fn pl_add_include_path(rt: *mut PlWrapper, path: *const c_char);
    pub fn pl_execute_file(
        rt: *mut PlWrapper,
        pattern_path: *const c_char,
        data_path: *const c_char,
    ) -> c_int;
    pub fn pl_last_error(rt: *mut PlWrapper) -> *const c_char;
    pub fn pl_field_count(rt: *mut PlWrapper) -> usize;
    pub fn pl_field_at(rt: *mut PlWrapper, index: usize) -> *const PlResult;
}
