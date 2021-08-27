use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt;

use crate::{raw, Il2CppType, WrapRaw};

/// Information about a C# parameter
#[repr(transparent)]
pub struct ParameterInfo(raw::ParameterInfo);

unsafe impl Send for ParameterInfo {}
unsafe impl Sync for ParameterInfo {}

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

impl fmt::Debug for ParameterInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParameterInfo")
            .field("name", &self.name())
            .field("position", &self.position())
            .field("ty", &self.ty())
            .finish()
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
