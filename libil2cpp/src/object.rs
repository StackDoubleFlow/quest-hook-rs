use super::{raw, Il2CppClass, WrapRaw};

/// An il2cpp object
#[repr(transparent)]
pub struct Il2CppObject(raw::Il2CppObject);

impl Il2CppObject {
    /// [`Il2CppClass`] of the object
    pub fn class(&self) -> &Il2CppClass {
        unsafe { Il2CppClass::wrap_ptr(self.raw().__bindgen_anon_1.klass) }.unwrap()
    }
}

unsafe impl WrapRaw for Il2CppObject {
    type Raw = raw::Il2CppObject;
}
