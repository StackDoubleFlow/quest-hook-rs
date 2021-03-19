use std::any::Any;
use std::ptr;

use crate::{Il2CppObject, Il2CppType, MethodInfo, Type, WrapRaw};

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
pub unsafe trait ThisParameter {
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
    /// the given [`Il2CppType`] in a method definition
    fn matches(params: &[&Il2CppType]) -> bool;
}

// When we are the callee, we can't know if the parameters will be null, so we
// can't impl Parameter for &T or &mut T
unsafe impl<T> Parameter for Option<&T>
where
    T: Type + Any,
{
    type Type = T;

    default fn matches(ty: &Il2CppType) -> bool {
        let self_ty = T::class().raw().this_arg;
        unsafe { self_ty.data.klassIndex == ty.raw().data.klassIndex }
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

// Il2CppObject can be used as a parameter wildcard for all reference types (as
// they all inherit from it). This is necessary cause we can't get the class
// hierarchy from an Il2CppType, but using a type that is more generic is fine
// as a callee. We use specialization to make this work even with the generic
// impl for Option<&T> where T: Type, and repetition isn't needed since the impl
// for Option<&mut T> defers to it.
unsafe impl Parameter for Option<&Il2CppObject> {
    fn matches(ty: &Il2CppType) -> bool {
        ty.is_reference_type()
    }
}

unsafe impl<T> ThisParameter for &T
where
    T: Type + Any,
{
    type Type = T;

    fn matches(method: &MethodInfo) -> bool {
        // When we are the callee it's fine to cast the received type to a less specific
        // one
        let class = T::class();
        method.class().hierarchy().any(|mc| ptr::eq(mc, class))
    }
}
unsafe impl<T> ThisParameter for &mut T
where
    T: Type + Any,
{
    type Type = T;

    fn matches(method: &MethodInfo) -> bool {
        <&T as ThisParameter>::matches(method)
    }
}

unsafe impl ThisParameter for () {
    type Type = ();

    fn matches(method: &MethodInfo) -> bool {
        method.is_static()
    }
}

unsafe impl Parameters<0> for () {
    type Type = ();

    fn matches(args: &[&Il2CppType]) -> bool {
        args.is_empty()
    }
}

unsafe impl<P> Parameters<1> for P
where
    P: Parameter,
{
    type Type = (P::Type,);

    fn matches(params: &[&Il2CppType]) -> bool {
        params.len() == 1 && P::matches(params[0])
    }
}
