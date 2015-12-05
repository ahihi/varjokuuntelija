pub mod options;
pub mod shaders;

use std::ffi::CString;

pub fn str_ptr(s: &str) -> *const i8 {
    CString::new(s).unwrap().as_ptr()
}
