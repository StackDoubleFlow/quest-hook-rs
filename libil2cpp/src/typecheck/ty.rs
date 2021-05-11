use crate::{Il2CppClass, Il2CppObject, Il2CppString};

/// Trait implemented by Rust types that are also C# types
///
/// # Safety
/// The Rust and C# types must be ABI-compatible
pub unsafe trait Type {
    /// Namespace containingthe class the type represents
    const NAMESPACE: &'static str;
    /// Name of the class the type represents
    const CLASS_NAME: &'static str;

    /// [`Il2CppClass`] of the type
    fn class() -> &'static Il2CppClass {
        Il2CppClass::find(Self::NAMESPACE, Self::CLASS_NAME).unwrap()
    }
}

macro_rules! impl_type {
    ($type:ty, $namespace:literal, $class:literal) => {
        unsafe impl Type for $type {
            const NAMESPACE: &'static str = $namespace;
            const CLASS_NAME: &'static str = $class;
        }
    };
}

impl_type!(u8, "System", "Byte");
impl_type!(i8, "System", "SByte");
impl_type!(u16, "System", "UInt16");
impl_type!(i16, "System", "Int16");
impl_type!(u32, "System", "UInt32");
impl_type!(i32, "System", "Int32");
impl_type!(u64, "System", "UInt64");
impl_type!(i64, "System", "Int64");
impl_type!(usize, "System", "UIntPtr");
impl_type!(isize, "System", "IntPtr");
impl_type!(f32, "System", "Single");
impl_type!(f64, "System", "Double");
impl_type!(bool, "System", "Boolean");
impl_type!(Il2CppObject, "System", "Object");
impl_type!(Il2CppString, "System", "String");
