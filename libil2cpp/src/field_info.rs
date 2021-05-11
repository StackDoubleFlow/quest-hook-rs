use std::borrow::Cow;
use std::ffi::CStr;

use super::{Argument, Il2CppClass, Il2CppObject, Il2CppType, Return, WrapRaw};
use crate::raw;

/// Information about a C# field
#[repr(transparent)]
pub struct FieldInfo(raw::FieldInfo);

impl FieldInfo {
    pub fn store<A>(&self, instance: &mut Il2CppObject, val: A)
    where
        A: Argument,
    {
        assert!(A::matches(self.ty()));

        unsafe {
            self.store_unchecked(instance, val);
        }
    }

    /// Store a value into a field without type checking
    ///
    /// # Safety
    /// To be safe, the provided types have to match the field signature
    pub unsafe fn store_unchecked(&self, instance: &mut Il2CppObject, val: impl Argument) {
        raw::field_set_value(instance.raw_mut(), self.raw(), val.invokable());
    }

    pub fn load<R>(&self, instance: &mut Il2CppObject) -> R
    where
        R: Return,
    {
        assert!(R::matches(self.ty()));

        unsafe { self.load_unchecked(instance) }
    }

    /// Store a value into a field without type checking
    ///
    /// # Safety
    /// To be safe, the provided types have to match the field signature
    pub unsafe fn load_unchecked<R>(&self, instance: &mut Il2CppObject) -> R
    where
        R: Return,
    {
        // let value
        // raw::field_set_value(instance.raw_mut(), self.raw(), val.invokable());
        unimplemented!()
    }

    /// Name of the field
    pub fn name(&self) -> Cow<'_, str> {
        let name = self.raw().name;
        assert!(!name.is_null());
        unsafe { CStr::from_ptr(name) }.to_string_lossy()
    }

    /// Class the field is from
    pub fn parent(&self) -> &Il2CppClass {
        unsafe { Il2CppClass::wrap_ptr(self.raw().parent) }.unwrap()
    }

    /// Type of the field
    pub fn ty(&self) -> &Il2CppType {
        unsafe { Il2CppType::wrap_ptr(self.raw().type_) }.unwrap()
    }
}

unsafe impl WrapRaw for FieldInfo {
    type Raw = raw::FieldInfo;
}
