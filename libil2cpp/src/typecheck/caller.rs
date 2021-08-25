use std::any::Any;
use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;

use crate::{Builtin, Il2CppObject, Il2CppType, MethodInfo, Type};

/// Trait implemented by types that can be used as a C# `this` arguments
///
/// # Note
/// You should most likely not be implementing this trait yourself, but rather
/// the [`Type`] trait
///
/// # Safety
/// The implementation must be correct
pub unsafe trait ThisArgument {
    /// Normalized type of `this`, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# `this` argument for the
    /// given [`MethodInfo`]
    fn matches(method: &MethodInfo) -> bool;

    /// Returns an untyped pointer which can be used as a libill2cpp `this`
    /// argument
    fn invokable(&mut self) -> *mut c_void;
}

/// Trait implemented by types that can be used as C# method arguments
///
/// # Note
/// You should most likely not be implementing this trait yourself, but rather
/// the [`Type`] trait
///
/// # Safety
/// The implementation must be correct
pub unsafe trait Argument {
    /// Normalized type of the argument, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# argument with the given
    /// [`Il2CppType`] to call a method
    fn matches(ty: &Il2CppType) -> bool;

    /// Returns an untyped pointer which can be used as a libil2cpp argument
    fn invokable(&mut self) -> *mut c_void;
}

/// Trait implemented by types that can be used as return types from C# methods
///
/// # Note
/// You should most likely not be implementing this trait yourself, but rather
/// the [`Type`] trait
///
/// # Safety
/// The implementation must be correct
pub unsafe trait Returned {
    /// Normalized type of the return type, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# return type of the given
    /// [`Il2CppType`]
    fn matches(ty: &Il2CppType) -> bool;

    /// Converts the [`Il2CppObject`] returned by
    /// [`runtime_invoke`](crate::raw::runtime_invoke) into self
    fn from_object(object: Option<&mut Il2CppObject>) -> Self;
}

/// Trait implemented by types that can be used as a collection of C# method
/// arguments
///
/// # Note
/// You should most likely not be implementing this trait yourself
///
/// # Safety
/// The implementation must be correct
pub unsafe trait Arguments<const N: usize> {
    /// Normalized type of the arguments, useful for caching
    type Type: Any;

    /// Checks whether the type can be used as a C# argument collection for the
    /// given [`MethodInfo`]
    fn matches(method: &MethodInfo) -> bool;

    /// Returns an array of untyped pointer which can be used to invoke C#
    /// methods
    fn invokable(&mut self) -> [*mut c_void; N];
}

unsafe impl<T> ThisArgument for Option<&mut T>
where
    T: Type,
{
    type Type = T;

    fn matches(method: &MethodInfo) -> bool {
        T::matches_this_argument(method)
    }

    fn invokable(&mut self) -> *mut c_void {
        unsafe { transmute((self as *mut Self).read()) }
    }
}

unsafe impl<T> ThisArgument for &mut T
where
    T: Type,
{
    type Type = T;

    fn matches(method: &MethodInfo) -> bool {
        T::matches_this_argument(method)
    }

    fn invokable(&mut self) -> *mut c_void {
        (*self as *mut T).cast()
    }
}

unsafe impl ThisArgument for () {
    type Type = ();

    fn matches(method: &MethodInfo) -> bool {
        method.is_static()
    }

    fn invokable(&mut self) -> *mut c_void {
        null_mut()
    }
}

// TODO: Remove this once rustfmt stops dropping generics on GATs
#[rustfmt::skip]
unsafe impl<T> Argument for Option<&mut T>
where
    T: for<'a> Type<Held<'a> = Option<&'a mut T>>,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches_reference_argument(ty)
    }

    fn invokable(&mut self) -> *mut c_void {
        unsafe { transmute((self as *mut Self).read()) }
    }
}

// TODO: Remove this once rustfmt stops dropping generics on GATs
#[rustfmt::skip]
unsafe impl<T> Argument for &mut T
where
    T: for<'a> Type<Held<'a> = Option<&'a mut T>>,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches_reference_argument(ty)
    }

    fn invokable(&mut self) -> *mut c_void {
        (*self as *mut T).cast()
    }
}

// TODO: Remove this once rustfmt stops dropping generics on GATs
#[rustfmt::skip]
unsafe impl<T> Returned for Option<&mut T>
where
    T: for<'a> Type<Held<'a> = Option<&'a mut T>>,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches_returned(ty)
    }

    fn from_object(object: Option<&mut Il2CppObject>) -> Self {
        unsafe { transmute(object) }
    }
}

// TODO: Remove this once rustfmt stops dropping generics on GATs
#[rustfmt::skip]
unsafe impl<T> Returned for Option<&T>
where
    T: for<'a> Type<Held<'a> = Option<&'a mut T>>,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches_returned(ty)
    }

    fn from_object(object: Option<&mut Il2CppObject>) -> Self {
        unsafe { transmute(object) }
    }
}

// TODO: Remove this once rustfmt stops dropping generics on GATs
#[rustfmt::skip]
unsafe impl<T> Returned for &mut T
where
    T: for<'a> Type<Held<'a> = Option<&'a mut T>>,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches_returned(ty)
    }

    fn from_object(object: Option<&mut Il2CppObject>) -> Self {
        unsafe { &mut *(object.unwrap() as *mut Il2CppObject).cast() }
    }
}

// TODO: Remove this once rustfmt stops dropping generics on GATs
#[rustfmt::skip]
unsafe impl<T> Returned for &T
where
    T: for<'a> Type<Held<'a> = Option<&'a mut T>>,
{
    type Type = T;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches_returned(ty)
    }

    fn from_object(object: Option<&mut Il2CppObject>) -> Self {
        unsafe { &*(object.unwrap() as *mut Il2CppObject).cast() }
    }
}

unsafe impl Returned for () {
    type Type = ();

    fn matches(ty: &Il2CppType) -> bool {
        ty.is_builtin(Builtin::Void)
    }

    fn from_object(_: Option<&mut Il2CppObject>) {}
}

unsafe impl Arguments<0> for () {
    type Type = ();

    fn matches(method: &MethodInfo) -> bool {
        method.parameters().is_empty()
    }

    fn invokable(&mut self) -> [*mut c_void; 0] {
        []
    }
}

unsafe impl<A> Arguments<1> for A
where
    A: Argument,
{
    type Type = (A::Type,);

    fn matches(method: &MethodInfo) -> bool {
        let params = method.parameters();
        params.len() == 1 && unsafe { A::matches(params.get_unchecked(0).ty()) }
    }

    fn invokable(&mut self) -> [*mut c_void; 1] {
        [Argument::invokable(self)]
    }
}
