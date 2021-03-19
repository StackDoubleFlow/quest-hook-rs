use std::fmt;
use std::{ffi::CStr, slice};

use crate::raw::{METHOD_ATTRIBUTE_ABSTRACT, METHOD_ATTRIBUTE_STATIC, METHOD_ATTRIBUTE_VIRTUAL};

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

    /// Whether the method is static
    pub fn is_static(&self) -> bool {
        self.raw().flags as u32 & METHOD_ATTRIBUTE_STATIC != 0
    }

    /// Whether the method is abstract
    pub fn is_abstract(&self) -> bool {
        self.raw().flags as u32 & METHOD_ATTRIBUTE_ABSTRACT != 0
    }

    /// Whether the method is virtual
    pub fn is_virtual(&self) -> bool {
        self.raw().flags as u32 & METHOD_ATTRIBUTE_VIRTUAL != 0
    }
}

unsafe impl WrapRaw for MethodInfo {
    type Raw = raw::MethodInfo;
}

impl fmt::Debug for MethodInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = self.parameters();
        let n = params.len() - 1;

        write!(f, "{:?}.{}(", self.class(), self.name().to_string_lossy())?;
        for p in &params[..n] {
            write!(f, "{}, ", p.name().to_string_lossy())?;
        }
        write!(f, "{})", params[n].name().to_string_lossy())
    }
}
