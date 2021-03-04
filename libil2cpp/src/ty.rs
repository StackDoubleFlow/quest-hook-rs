use super::{raw, WrapRaw};

/// An il2cpp type
#[repr(transparent)]
pub struct Il2CppType(raw::Il2CppType);

unsafe impl WrapRaw for Il2CppType {
    type Raw = raw::Il2CppType;
}
