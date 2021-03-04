use super::{raw, WrapRaw};

/// An il2cpp object
#[repr(transparent)]
pub struct Il2CppObject(raw::Il2CppObject);

unsafe impl WrapRaw for Il2CppObject {
    type Raw = raw::Il2CppObject;
}
