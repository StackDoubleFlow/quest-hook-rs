use std::borrow::Cow;
use std::ffi::CStr;
use std::mem::MaybeUninit;

use crate::{raw, Argument, Il2CppClass, Il2CppObject, Il2CppType, Return, WrapRaw};

/// Information about a C# field
#[repr(transparent)]
pub struct FieldInfo(raw::FieldInfo);

impl FieldInfo {
    /// Store a typechecked value into a field
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
    /// To be safe, the provided type has to match the field signature
    pub unsafe fn store_unchecked<A>(&self, instance: &mut Il2CppObject, val: A)
    where
        A: Argument,
    {
        raw::field_set_value(instance.raw_mut(), self.raw(), val.invokable());
    }

    /// Load a typechecked value from a field
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
    /// To be safe, the provided type has to match the field signature
    pub unsafe fn load_unchecked<R>(&self, instance: &mut Il2CppObject) -> R
    where
        R: Return,
    {
        let val = MaybeUninit::uninit();
        raw::field_get_value(instance.raw_mut(), self.raw(), &val as *const _ as *const _);
        val.assume_init()
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
