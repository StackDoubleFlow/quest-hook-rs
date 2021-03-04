use std::{ffi::CStr, slice};

use super::{raw, Il2CppClass, Il2CppType, ParameterInfo, WrapRaw};

/// Information about a C# method
#[repr(transparent)]
pub struct MethodInfo(raw::MethodInfo);

impl MethodInfo {
    /// Name of the method
    pub fn name(&self) -> &CStr {
        let name = self.raw().name;
        assert!(!name.is_null());
        unsafe { CStr::from_ptr(name) }
    }

    /// Class the method is from
    pub fn class(&self) -> &Il2CppClass {
        unsafe { Il2CppClass::wrap_ptr(self.raw().klass) }.unwrap()
    }

    /// Return type of the method
    pub fn return_type(&self) -> Option<&Il2CppType> {
        unsafe { Il2CppType::wrap_ptr(self.raw().return_type) }
    }

    /// Parameters the method takes
    pub fn parameters(&self) -> &[&ParameterInfo] {
        let parameters = self.raw().parameters;
        if !parameters.is_null() {
            unsafe { slice::from_raw_parts(parameters as _, self.parameters_count()) }
        } else {
            &[]
        }
    }

    /// Number of parameters the method takes
    pub fn parameters_count(&self) -> usize {
        self.raw().parameters_count as _
    }
}

unsafe impl WrapRaw for MethodInfo {
    type Raw = raw::MethodInfo;
}
