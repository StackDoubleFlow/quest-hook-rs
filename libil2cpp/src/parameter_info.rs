use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt;

use crate::{raw, Il2CppType, WrapRaw};

/// Information about a C# parameter
#[repr(transparent)]
pub struct ParameterInfo(raw::ParameterInfo);

impl ParameterInfo {
    /// Name of the parameter
    pub fn name(&self) -> Cow<'_, str> {
        let name = self.raw().name;
        assert!(!name.is_null());
        unsafe { CStr::from_ptr(name) }.to_string_lossy()
    }

    /// Position of the parameter
    pub fn position(&self) -> usize {
        self.raw().position as _
    }

    /// Type of the parameter
    pub fn ty(&self) -> &Il2CppType {
        unsafe { Il2CppType::wrap_ptr(self.raw().parameter_type) }.unwrap()
    }
}

impl fmt::Display for ParameterInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.ty(), self.name())
    }
}

unsafe impl WrapRaw for ParameterInfo {
    type Raw = raw::ParameterInfo;
}
