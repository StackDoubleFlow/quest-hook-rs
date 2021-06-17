use std::any::Any;

use crate::{Builtin, Il2CppType, MethodInfo, ParameterInfo, Type};

/// Trait implemented by types that can be used as C# method parameters
///
/// # Safety
/// Interfaces depending on this trait assume that all of its methods are
/// correct in an il2cpp context
pub unsafe trait Parameter {
    /// Normalized type of the parameter, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# parameter with the given
    /// [`Il2CppType`] in a method definition
    fn matches(ty: &Il2CppType) -> bool;
}

/// Trait implemented by types that can be used as a C# instance parameters
///
/// # Safety
/// Interfaces depending on this trait assume that all of its methods are
/// correct in an il2cpp context
pub unsafe trait This {
    /// Normalized type of `this`, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# instance parameter for the
    /// given [`MethodInfo`]
    fn matches(method: &MethodInfo) -> bool;
}

/// Trait implemented by types that can be used as a collection of C# method
/// parameters
///
/// # Safety
/// Interfaces depending on this trait assume that all of its methods are
/// correct in an il2cpp context
pub unsafe trait Parameters<const N: usize> {
    /// Normalized type of the parameters, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# parameter collection with
    /// the given [`ParameterInfo`]s in a method definition
    fn matches(params: &[ParameterInfo]) -> bool;
}

/// Trait implemented by types that can be used as callee return types for C#
/// methods
///
/// # Safety
/// Interfaces depending on this trait assume that all of its methods are
/// correct in an il2cpp context
pub unsafe trait Return {
    /// Normalized type of the return type, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# return type with the given
    /// [`Il2CppType`] in a method definition
    fn matches(ty: &Il2CppType) -> bool;
}

// When we are the callee, we can't know if the parameters will be null, so we
// can't impl Parameter for &T or &mut T
unsafe impl<T> Parameter for Option<&T>
where
    T: Type + Any,
{
    type Type = T;

    default fn matches(ty: &Il2CppType) -> bool {
        T::class().is_assignable_from(ty.class())
    }
}
unsafe impl<T> Parameter for Option<&mut T>
where
    T: Type + Any,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        <Option<&T> as Parameter>::matches(ty)
    }
}

unsafe impl<T> This for &T
where
    T: Type + Any,
{
    type Type = T;

    fn matches(method: &MethodInfo) -> bool {
        T::class().is_assignable_from(method.class())
    }
}
unsafe impl<T> This for &mut T
where
    T: Type + Any,
{
    type Type = T;

    fn matches(method: &MethodInfo) -> bool {
        <&T as This>::matches(method)
    }
}

unsafe impl This for () {
    type Type = ();

    fn matches(method: &MethodInfo) -> bool {
        method.is_static()
    }
}

unsafe impl Parameters<0> for () {
    type Type = ();

    fn matches(args: &[ParameterInfo]) -> bool {
        args.is_empty()
    }
}

unsafe impl<P> Parameters<1> for P
where
    P: Parameter,
{
    type Type = (P::Type,);

    fn matches(params: &[ParameterInfo]) -> bool {
        params.len() == 1 && P::matches(params[0].ty())
    }
}

unsafe impl<T> Return for &T
where
    T: Type + Any,
{
    type Type = T;

    default fn matches(ty: &Il2CppType) -> bool {
        ty.class().is_assignable_from(T::class())
    }
}
unsafe impl<T> Return for &mut T
where
    T: Type + Any,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        <&T as Return>::matches(ty)
    }
}
unsafe impl<T> Return for Option<&T>
where
    T: Type + Any,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        <&T as Return>::matches(ty)
    }
}
unsafe impl<T> Return for Option<&mut T>
where
    T: Type + Any,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        <&T as Return>::matches(ty)
    }
}

unsafe impl Return for () {
    type Type = ();

    fn matches(ty: &Il2CppType) -> bool {
        ty.is_builtin(Builtin::Void)
    }
}

macro_rules! impl_return_value {
    ($type:ty, $($builtin:ident),+) => {
        unsafe impl Return for $type {
            type Type = $type;

            fn matches(ty: &Il2CppType) -> bool {
                $(ty.is_builtin(Builtin::$builtin))||+
            }
        }
        unsafe impl Return for &$type {
            fn matches(_: &Il2CppType) -> bool {
                false
            }
        }
    }
}

impl_return_value!(u8, Byte);
impl_return_value!(i8, SByte);
impl_return_value!(u16, UShort, Char);
impl_return_value!(i16, Short);
impl_return_value!(u32, UInt);
impl_return_value!(i32, Int);
impl_return_value!(u64, ULong);
impl_return_value!(i64, Long);
impl_return_value!(f32, Single);
impl_return_value!(f64, Double);
impl_return_value!(bool, Bool);
