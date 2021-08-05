use std::any::Any;

use crate::{
    Il2CppClass, Il2CppObject, Il2CppReflectionMethod, Il2CppReflectionType, Il2CppString,
    Il2CppType, MethodInfo,
};

/// Trait implemented by Rust types that are also C# types
///
/// # Safety
/// The Rust and C# types must be ABI-compatible and the trait implementation
/// must be correct
pub unsafe trait Type: Any {
    /// Semantics of the type, either [`Reference`] or [`Value`]
    type Semantics: semantics::Semantics;

    /// Namespace containingthe class the type represents
    const NAMESPACE: &'static str;
    /// Name of the class the type represents
    const CLASS_NAME: &'static str;

    /// [`Il2CppClass`] of the type
    fn class() -> &'static Il2CppClass {
        Il2CppClass::find(Self::NAMESPACE, Self::CLASS_NAME).unwrap()
    }

    /// Whether the type can be used as a `this` argument for the given
    /// [`MethodInfo`]
    fn matches_this_argument(method: &MethodInfo) -> bool;

    /// Whether the type can be used as a `this` parameter for the given
    /// [`MethodInfo`]
    fn matches_this_parameter(method: &MethodInfo) -> bool;

    /// Whether a reference to the type can be used as an argument of the given
    /// [`Il2CppType`]
    fn matches_reference_argument(ty: &Il2CppType) -> bool;
    /// Whether a value of the type can be used as an argument of the given
    /// [`Il2CppType`]
    fn matches_value_argument(ty: &Il2CppType) -> bool;

    /// Whether a reference to the type can be used as a parameter of the given
    /// [`Il2CppType`]
    fn matches_reference_parameter(ty: &Il2CppType) -> bool;
    /// Whether a value of the type can be used as a parameter of the given
    /// [`Il2CppType`]
    fn matches_value_parameter(ty: &Il2CppType) -> bool;

    /// Whether a reference to the type can be used as the value of the given
    /// [`Il2CppType`] returned from a C# method
    fn matches_reference_returned(ty: &Il2CppType) -> bool;
    /// Whether a value of the type can be used as the value of the given
    /// [`Il2CppType`] returned from a C# method
    fn matches_value_returned(ty: &Il2CppType) -> bool;

    /// Whether a reference to the type can be used as the return value of the
    /// given [`Il2CppType`] for a C# method
    fn matches_reference_return(ty: &Il2CppType) -> bool;
    /// Whether a value of the type can be used as the return value of the
    /// given [`Il2CppType`] for a C# method
    fn matches_value_return(ty: &Il2CppType) -> bool;
}

/// Marker type used to specify reference semantics
#[allow(missing_debug_implementations)]
pub struct Reference;
impl semantics::Semantics for Reference {}
impl semantics::ReferenceArgument for Reference {}
impl semantics::ReferenceParameter for Reference {}
impl semantics::ReferenceReturned for Reference {}
impl semantics::ReferenceReturn for Reference {}

/// Marker type used to specify value semantics
#[allow(missing_debug_implementations)]
pub struct Value;
impl semantics::Semantics for Value {}
impl semantics::ValueArgument for Value {}
impl semantics::ReferenceArgument for Value {}
impl semantics::ValueParameter for Value {}
impl semantics::ReferenceParameter for Value {}
impl semantics::ValueReturned for Value {}
impl semantics::ValueReturn for Value {}

pub(crate) mod semantics {
    pub trait Semantics {}

    pub trait ReferenceArgument: Semantics {}
    pub trait ValueArgument: Semantics {}

    pub trait ReferenceParameter: Semantics {}
    pub trait ValueParameter: Semantics {}

    pub trait ReferenceReturned: Semantics {}
    pub trait ValueReturned: Semantics {}

    pub trait ReferenceReturn: Semantics {}
    pub trait ValueReturn: Semantics {}
}

/// Implements the [`Type`] trait for a C# reference type
///
/// # Safety
/// The Rust and C# types must be ABI-compatible and the C# type must be a
/// reference type
#[macro_export]
macro_rules! unsafe_impl_reference_type {
    ($type:ty, $namespace:literal, $class:literal) => {
        unsafe impl $crate::Type for $type {
            type Semantics = $crate::Reference;

            const NAMESPACE: &'static str = $namespace;
            const CLASS_NAME: &'static str = $class;

            fn matches_this_argument(method: &MethodInfo) -> bool {
                method
                    .class()
                    .is_assignable_from(<Self as $crate::Type>::class())
            }

            fn matches_this_parameter(method: &MethodInfo) -> bool {
                <Self as $crate::Type>::class().is_assignable_from(method.class())
            }

            fn matches_reference_argument(ty: &Il2CppType) -> bool {
                ty.class()
                    .is_assignable_from(<Self as $crate::Type>::class())
            }
            fn matches_value_argument(_: &Il2CppType) -> bool {
                false
            }

            fn matches_reference_parameter(ty: &Il2CppType) -> bool {
                <Self as $crate::Type>::class().is_assignable_from(ty.class())
            }
            fn matches_value_parameter(_: &Il2CppType) -> bool {
                false
            }

            fn matches_reference_returned(ty: &Il2CppType) -> bool {
                <Self as $crate::Type>::class().is_assignable_from(ty.class())
            }
            fn matches_value_returned(_: &Il2CppType) -> bool {
                false
            }

            fn matches_reference_return(ty: &Il2CppType) -> bool {
                ty.class()
                    .is_assignable_from(<Self as $crate::Type>::class())
            }
            fn matches_value_return(_: &Il2CppType) -> bool {
                false
            }
        }
    };
}

/// Implements the [`Type`] trait for a C# value type
///
/// # Safety
/// The Rust and C# types must be ABI-compatible and the C# type must be a value
/// type
#[macro_export]
macro_rules! unsafe_impl_value_type {
    ($type:ty, $namespace:literal, $class:literal) => {
        unsafe impl $crate::Type for $type {
            type Semantics = $crate::Value;

            const NAMESPACE: &'static str = $namespace;
            const CLASS_NAME: &'static str = $class;

            fn matches_this_argument(method: &MethodInfo) -> bool {
                method
                    .class()
                    .is_assignable_from(<Self as $crate::Type>::class())
            }

            fn matches_this_parameter(method: &MethodInfo) -> bool {
                <Self as $crate::Type>::class().is_assignable_from(method.class())
            }

            fn matches_value_argument(ty: &Il2CppType) -> bool {
                !ty.is_ref()
                    && ty
                        .class()
                        .is_assignable_from(<Self as $crate::Type>::class())
            }
            fn matches_reference_argument(ty: &Il2CppType) -> bool {
                ty.is_ref()
                    && ty
                        .class()
                        .is_assignable_from(<Self as $crate::Type>::class())
            }

            fn matches_value_parameter(ty: &Il2CppType) -> bool {
                !ty.is_ref() && <Self as $crate::Type>::class().is_assignable_from(ty.class())
            }
            fn matches_reference_parameter(ty: &Il2CppType) -> bool {
                ty.is_ref() && <Self as $crate::Type>::class().is_assignable_from(ty.class())
            }

            fn matches_value_returned(ty: &Il2CppType) -> bool {
                <Self as $crate::Type>::class().is_assignable_from(ty.class())
            }
            fn matches_reference_returned(_: &Il2CppType) -> bool {
                false
            }

            fn matches_value_return(ty: &Il2CppType) -> bool {
                ty.class()
                    .is_assignable_from(<Self as $crate::Type>::class())
            }
            fn matches_reference_return(_: &Il2CppType) -> bool {
                false
            }
        }
    };
}

unsafe_impl_value_type!(u8, "System", "Byte");
unsafe_impl_value_type!(i8, "System", "SByte");
unsafe_impl_value_type!(u16, "System", "UInt16");
unsafe_impl_value_type!(i16, "System", "Int16");
unsafe_impl_value_type!(u32, "System", "UInt32");
unsafe_impl_value_type!(i32, "System", "Int32");
unsafe_impl_value_type!(u64, "System", "UInt64");
unsafe_impl_value_type!(i64, "System", "Int64");
unsafe_impl_value_type!(usize, "System", "UIntPtr");
unsafe_impl_value_type!(isize, "System", "IntPtr");
unsafe_impl_value_type!(f32, "System", "Single");
unsafe_impl_value_type!(f64, "System", "Double");
unsafe_impl_value_type!(bool, "System", "Boolean");

unsafe_impl_reference_type!(Il2CppObject, "System", "Object");
unsafe_impl_reference_type!(Il2CppString, "System", "String");
unsafe_impl_reference_type!(Il2CppReflectionType, "System", "RuntimeType");
unsafe_impl_reference_type!(Il2CppReflectionMethod, "System.Reflection", "MonoMethod");
