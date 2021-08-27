use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt;
use std::mem::MaybeUninit;

use crate::{raw, Argument, Il2CppClass, Il2CppObject, Il2CppType, Type, WrapRaw};

/// Information about a C# field
#[repr(transparent)]
pub struct FieldInfo(raw::FieldInfo);

unsafe impl Send for FieldInfo {}
unsafe impl Sync for FieldInfo {}

impl FieldInfo {
    /// Store a typechecked value into a field
    pub fn store<A>(&self, instance: &mut Il2CppObject, val: A)
    where
        A: Argument,
    {
        assert!(A::matches(self.ty()));
        unsafe { self.store_unchecked(instance, val) };
    }

    /// Store a value into a field without type checking
    ///
    /// # Safety
    /// To be safe, the provided type has to match the field signature
    pub unsafe fn store_unchecked<A>(&self, instance: &mut Il2CppObject, mut val: A)
    where
        A: Argument,
    {
        raw::field_set_value(instance.raw_mut(), self.raw(), val.invokable());
    }

    /// Load a typechecked value from a field
    pub fn load<'a, T>(&self, instance: &'a mut Il2CppObject) -> T::Held<'a>
    where
        T: Type,
    {
        assert!(T::class().is_assignable_from(self.ty().class()));
        unsafe { self.load_unchecked::<T>(instance) }
    }

    /// Load a value from a field without type checking
    ///
    /// # Safety
    /// To be safe, the provided type has to match the field signature
    pub unsafe fn load_unchecked<'a, T>(&self, instance: &'a mut Il2CppObject) -> T::Held<'a>
    where
        T: Type,
    {
        let mut val: MaybeUninit<T::Held<'a>> = MaybeUninit::uninit();
        raw::field_get_value(instance.raw_mut(), self.raw(), val.as_mut_ptr().cast());
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

impl fmt::Debug for FieldInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldInfo")
            .field("name", &self.name())
            .field("type", self.ty())
            .finish()
    }
}
