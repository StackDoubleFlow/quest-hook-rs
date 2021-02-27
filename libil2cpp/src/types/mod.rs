mod raw;

// TODO: Safe wrappers around certain types
pub use raw::*;

use crate::functions;
use std::ffi::CString;

impl Il2CppClass {
    pub fn find_method(
        &self,
        method_name: &str,
        method_args_count: u32,
    ) -> Option<&'static MethodInfo> {
        let method_name = CString::new(method_name).unwrap();

        functions::class_get_method_from_name(self, method_name.as_ptr(), method_args_count)
    }
}