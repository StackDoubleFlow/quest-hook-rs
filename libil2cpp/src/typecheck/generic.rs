use crate::{raw, Il2CppReflectionType, Type, WrapRaw};

/// Trait implemented for Rust types which can represent a list of C# generic
/// arguments
pub trait Generics {
    /// Returns an array of `System.RuntimeType`s matching the generic arguments
    fn type_array() -> &'static mut raw::Il2CppArray;
}

impl<T: Type> Generics for T {
    fn type_array() -> &'static mut raw::Il2CppArray {
        let arr = unsafe { raw::array_new(Il2CppReflectionType::class().raw(), 1) }.unwrap();
        unsafe {
            (((arr as *mut _ as isize) + (raw::kIl2CppSizeOfArray as isize))
                as *mut &Il2CppReflectionType)
                .write_unaligned(T::class().ty().reflection_object());
        }
        arr
    }
}
