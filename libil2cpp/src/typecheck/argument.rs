use std::any::Any;
use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::{self, null_mut};

use crate::{Il2CppType, MethodInfo, Type, WrapRaw};

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
pub unsafe trait ThisArgument {
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
    /// given [`Il2CppType`] to call a method
    fn matches(args: &[&Il2CppType]) -> bool;
    /// Returns an array of untyped pointer which can be used to invoke C#
    /// methods
    fn invokable(&self) -> [*mut c_void; N];
}

// When we are the caller, we can't know if the arguments will be mutated, so we
// can't impl Argument for &T or Option<&T>
unsafe impl<T> Argument for &mut T
where
    T: Type + Any,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        // When we are the caller it's fine to pass a type that is more specific than
        // the one expected
        T::class().hierarchy().any(|c| {
            let self_ty = c.raw().this_arg;
            unsafe { self_ty.data.klassIndex == ty.raw().data.klassIndex }
        })
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

unsafe impl<T> ThisArgument for &mut T
where
    T: Type + Any,
{
    type Type = T;

    fn matches(method: &MethodInfo) -> bool {
        let method_class = method.class();
        T::class().hierarchy().any(|c| ptr::eq(c, method_class))
    }

    fn invokable(&self) -> *mut c_void {
        *self as *const T as *mut c_void
    }
}

unsafe impl ThisArgument for () {
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

    fn matches(args: &[&Il2CppType]) -> bool {
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

    fn matches(args: &[&Il2CppType]) -> bool {
        args.len() == 1 && A::matches(args[0])
    }

    fn invokable(&self) -> [*mut c_void; 1] {
        [Argument::invokable(self)]
    }
}
