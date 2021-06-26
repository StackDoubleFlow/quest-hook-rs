use std::any::Any;
use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;

use crate::{raw, Builtin, Il2CppObject, Il2CppType, MethodInfo, ParameterInfo, Type, WrapRaw};

/// Trait implemented by types that can be used as C# method arguments
///
/// # Safety
/// Interfaces depending on this trait assume that all of its methods are
/// correct in an il2cpp context
pub unsafe trait Argument {
    /// Normalized type of the argument, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# argument with the given
    /// [`Il2CppType`] to call a method
    fn matches(ty: &Il2CppType) -> bool;
    /// Returns an untyped pointer which can be used as an argument to invoke C#
    /// methods
    fn invokable(&self) -> *mut c_void;
}

/// Trait implemented by types that can be used as a C# instance arguments
///
/// # Safety
/// Interfaces depending on this trait assume that all of its methods are
/// correct in an il2cpp context
pub unsafe trait This {
    /// Normalized type of the `this`, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# instance argument for the
    /// given [`MethodInfo`]
    fn matches(method: &MethodInfo) -> bool;
    /// Returns an untyped pointer which can be used as an instance argument
    fn invokable(&self) -> *mut c_void;
}

/// Trait implemented by types that can be used as a collection of C# method
/// arguments
///
/// # Safety
/// Interfaces depending on this trait assume that all of its methods are
/// correct in an il2cpp context
pub unsafe trait Arguments<const N: usize> {
    /// Normalized type of the arguments, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# argument collection with the
    /// given [`ParameterInfo`]s to call a method
    fn matches(args: &[ParameterInfo]) -> bool;
    /// Returns an array of untyped pointer which can be used to invoke C#
    /// methods
    fn invokable(&self) -> [*mut c_void; N];
}

/// Trait implemented by types that can be used as caller return types for C#
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
    /// Converts the [`Il2CppObject`] returned by
    /// [`runtime_invoke`](crate::raw::runtime_invoke) into self
    fn from_object(object: Option<&mut Il2CppObject>) -> Self;
}

// When we are the caller, we can't know if the arguments will be mutated, so we
// can't impl Argument for &T or Option<&T>
unsafe impl<T> Argument for &mut T
where
    T: Type + Any,
{
    type Type = T;

    default fn matches(ty: &Il2CppType) -> bool {
        ty.class().is_assignable_from(T::class())
    }

    fn invokable(&self) -> *mut c_void {
        *self as *const T as *mut c_void
    }
}
unsafe impl<T> Argument for Option<&mut T>
where
    T: Type + Any,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        <&mut T as Argument>::matches(ty)
    }

    fn invokable(&self) -> *mut c_void {
        let this = unsafe { *(self as *const Option<&mut T> as *const Option<&T>) };
        unsafe { transmute::<Option<&T>, *mut c_void>(this) }
    }
}

unsafe impl<T> This for &mut T
where
    T: Type + Any,
{
    type Type = T;

    fn matches(method: &MethodInfo) -> bool {
        method.class().is_assignable_from(T::class())
    }

    fn invokable(&self) -> *mut c_void {
        *self as *const T as *mut c_void
    }
}

unsafe impl This for () {
    type Type = ();

    fn matches(method: &MethodInfo) -> bool {
        method.is_static()
    }

    fn invokable(&self) -> *mut c_void {
        null_mut()
    }
}

unsafe impl Arguments<0> for () {
    type Type = ();

    fn matches(args: &[ParameterInfo]) -> bool {
        args.is_empty()
    }

    fn invokable(&self) -> [*mut c_void; 0] {
        []
    }
}

unsafe impl<A> Arguments<1> for A
where
    A: Argument,
{
    type Type = (A::Type,);

    fn matches(args: &[ParameterInfo]) -> bool {
        args.len() == 1 && A::matches(args[0].ty())
    }

    fn invokable(&self) -> [*mut c_void; 1] {
        [Argument::invokable(self)]
    }
}

unsafe impl<T> Return for Option<&T>
where
    T: Type + Any,
{
    type Type = T;

    default fn matches(ty: &Il2CppType) -> bool {
        T::class().is_assignable_from(ty.class())
    }

    default fn from_object(object: Option<&mut Il2CppObject>) -> Self {
        unsafe { transmute(object) }
    }
}
unsafe impl<T> Return for Option<&mut T>
where
    T: Type + Any,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        <Option<&T> as Return>::matches(ty)
    }

    fn from_object(object: Option<&mut Il2CppObject>) -> Self {
        unsafe { transmute(<Option<&T> as Return>::from_object(object)) }
    }
}

unsafe impl Return for () {
    type Type = ();

    fn matches(ty: &Il2CppType) -> bool {
        ty.is_builtin(Builtin::Void)
    }

    fn from_object(_: Option<&mut Il2CppObject>) -> Self {}
}

macro_rules! impl_return_value {
    ($type:ty, $($builtin:ident),+) => {
        unsafe impl Return for $type {
            type Type = $type;

            fn matches(ty: &Il2CppType) -> bool {
                $(ty.is_builtin(Builtin::$builtin))||+
            }

            fn from_object(object: Option<&mut Il2CppObject>) -> Self {
                unsafe { *(raw::object_unbox(object.unwrap().raw_mut()) as *mut Self) }
            }
        }

        unsafe impl Return for Option<&$type> {
            fn matches(_: &Il2CppType) -> bool {
                false
            }

            fn from_object(_: Option<&mut Il2CppObject>) -> Self {
                panic!("value types can't be returned by reference")
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
