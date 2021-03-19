use std::ffi::CStr;

use super::{raw, Il2CppType, WrapRaw};

/// Information about a C# parameter
#[repr(transparent)]
pub struct ParameterInfo(raw::ParameterInfo);

impl ParameterInfo {
    /// Name of the parameter
    pub fn name(&self) -> &CStr {
        let name = self.raw().name;
        assert!(!name.is_null());
        unsafe { CStr::from_ptr(name) }
    }

    /// Position of the parameter
    pub fn position(&self) -> usize {
        self.raw().position as _
    }

    /// Type of the parameter
    pub fn parameter_type(&self) -> &Il2CppType {
        unsafe { Il2CppType::wrap_ptr(self.raw().parameter_type) }.unwrap()
    }
}

unsafe impl WrapRaw for ParameterInfo {
    type Raw = raw::ParameterInfo;
}
