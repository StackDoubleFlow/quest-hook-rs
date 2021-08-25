use std::fmt;

use crate::{Builtin, Il2CppType, MethodInfo, Type};

/// Trait implemented by types that can be used as C# `this` method parameters
///
/// # Note
/// You should most likely not be implementing this trait yourself, but rather
/// the [`Type`] trait
///
/// # Safety
/// The implementation must be correct
pub unsafe trait ThisParameter {
    /// Type of the actual `this` parameter
    type Actual;

    /// Checks whether the type can be used as a C# instance parameter for the
    /// given [`MethodInfo`]
    fn matches(method: &MethodInfo) -> bool;

    /// Converts from the actual type to the desired one
    fn from_actual(actual: Self::Actual) -> Self;
    /// Converts from the desired type into the actual one
    fn into_actual(self) -> Self::Actual;
}

/// Trait implemented by types that can be used as C# method parameters
///
/// # Note
/// You should most likely not be implementing this trait yourself, but rather
/// the [`Type`] trait
///
/// # Safety
/// The implementation must be correct
pub unsafe trait Parameter {
    /// Type of the actual parameter
    type Actual;

    /// Checks whether the type can be used as a C# parameter with the given
    /// [`Il2CppType`]
    fn matches(ty: &Il2CppType) -> bool;

    /// Converts from the actual type to the desired one
    fn from_actual(actual: Self::Actual) -> Self;
    /// Converts from the desired type into the actual one
    fn into_actual(self) -> Self::Actual;
}

/// Trait implemented by types that can be used as return types for C#
/// methods
///
/// # Note
/// You should most likely not be implementing this trait yourself, but rather
/// the [`Type`] trait
///
/// # Safety
/// The implementation must be correct
pub unsafe trait Return {
    /// Type of the actual return value
    type Actual;

    /// Checks whether the type can be used as a C# return type of the given
    /// [`Il2CppType`]
    fn matches(ty: &Il2CppType) -> bool;

    /// Converts from the desired type into the actual one
    fn into_actual(self) -> Self::Actual;
    /// Converts from the actual type to the desired one
    fn from_actual(actual: Self::Actual) -> Self;
}

/// Trait implemented by types that can be used as a collection of C# method
/// parameters
///
/// # Note
/// You should most likely not be implementing this trait yourself
///
/// # Safety
/// The implementation must be correct
pub unsafe trait Parameters {
    /// Parameter count
    const COUNT: usize;

    /// Checks whether the type can be used as a C# parameter collection for the
    /// given [`MethodInfo`]
    fn matches(method: &MethodInfo) -> bool;
}

unsafe impl<T> ThisParameter for Option<&mut T>
where
    T: Type,
{
    type Actual = Self;

    fn matches(method: &MethodInfo) -> bool {
        T::matches_this_parameter(method)
    }

    fn from_actual(actual: Self::Actual) -> Self {
        actual
    }
    fn into_actual(self) -> Self::Actual {
        self
    }
}

unsafe impl<T> ThisParameter for &mut T
where
    T: Type,
{
    type Actual = Option<Self>;

    fn matches(method: &MethodInfo) -> bool {
        T::matches_this_parameter(method)
    }

    fn from_actual(actual: Self::Actual) -> Self {
        actual.unwrap()
    }
    fn into_actual(self) -> Self::Actual {
        Some(self)
    }
}

unsafe impl ThisParameter for () {
    type Actual = !;

    fn matches(method: &MethodInfo) -> bool {
        method.is_static()
    }

    fn from_actual(_: !) {
        unreachable!();
    }
    fn into_actual(self) -> ! {
        unreachable!()
    }
}

// TODO: Remove this once rustfmt stops dropping generics on GATs
#[rustfmt::skip]
unsafe impl<T> Parameter for Option<&mut T>
where
    T: for<'a> Type<Held<'a> = Option<&'a mut T>>,
{
    type Actual = Self;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches_reference_parameter(ty)
    }

    fn from_actual(actual: Self::Actual) -> Self {
        actual
    }
    fn into_actual(self) -> Self::Actual {
        self
    }
}

// TODO: Remove this once rustfmt stops dropping generics on GATs
#[rustfmt::skip]
unsafe impl<T> Parameter for &mut T
where
    T: for<'a> Type<Held<'a> = Option<&'a mut T>>,
{
    type Actual = Option<Self>;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches_reference_parameter(ty)
    }

    fn from_actual(actual: Self::Actual) -> Self {
        actual.unwrap()
    }
    fn into_actual(self) -> Self::Actual {
        Some(self)
    }
}

// TODO: Remove this once rustfmt stops dropping generics on GATs
#[rustfmt::skip]
unsafe impl<T> Return for Option<&mut T>
where
    T: for<'a> Type<Held<'a> = Option<&'a mut T>>,
{
    type Actual = Self;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches_return(ty)
    }

    fn into_actual(self) -> Self::Actual {
        self
    }
    fn from_actual(actual: Self::Actual) -> Self {
        actual
    }
}

// TODO: Remove this once rustfmt stops dropping generics on GATs
#[rustfmt::skip]
unsafe impl<T> Return for &mut T
where
    T: for<'a> Type<Held<'a> = Option<&'a mut T>>,
{
    type Actual = Option<Self>;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches_return(ty)
    }

    fn into_actual(self) -> Self::Actual {
        Some(self)
    }
    fn from_actual(actual: Self::Actual) -> Self {
        actual.unwrap()
    }
}

unsafe impl Return for () {
    type Actual = ();

    fn matches(ty: &Il2CppType) -> bool {
        ty.is_builtin(Builtin::Void)
    }

    fn into_actual(self) {}
    fn from_actual((): ()) {}
}

unsafe impl<T, E> Return for Result<T, E>
where
    T: Return,
    E: fmt::Debug,
{
    type Actual = T::Actual;

    fn matches(ty: &Il2CppType) -> bool {
        T::matches(ty)
    }

    fn into_actual(self) -> Self::Actual {
        self.unwrap().into_actual()
    }
    fn from_actual(actual: Self::Actual) -> Self {
        Ok(T::from_actual(actual))
    }
}

unsafe impl Parameters for () {
    const COUNT: usize = 0;

    fn matches(method: &MethodInfo) -> bool {
        method.parameters().is_empty()
    }
}

unsafe impl<P> Parameters for P
where
    P: Parameter,
{
    const COUNT: usize = 1;

    fn matches(method: &MethodInfo) -> bool {
        let params = method.parameters();
        params.len() == 1 && unsafe { P::matches(params.get_unchecked(0).ty()) }
    }
}
