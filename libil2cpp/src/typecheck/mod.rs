pub mod callee;
pub mod caller;
pub mod ty;

use std::ffi::c_void;

use crate::{
    Argument, Arguments, Builtin, Il2CppObject, Il2CppType, Parameter, ParameterInfo, Parameters,
    Return, WrapRaw,
};

macro_rules! impl_builtin_value {
    ($type:ty, $($builtin:ident),+) => {
        unsafe impl Argument for $type {
            type Type = $type;

            fn matches(ty: &Il2CppType) -> bool {
                ty.raw().byref() == 0 && ($(ty.is_builtin(Builtin::$builtin))||+)
            }

            fn invokable(&self) -> *mut c_void {
                self as *const $type as *mut c_void
            }
        }
        unsafe impl Argument for &mut $type {
            fn matches(ty: &Il2CppType) -> bool {
                ty.raw().byref() != 0 && ($(ty.is_builtin(Builtin::$builtin))||+)
            }
        }

        unsafe impl Parameter for $type {
            type Type = $type;

            fn matches(ty: &Il2CppType) -> bool {
                ty.raw().byref() == 0 && ($(ty.is_builtin(Builtin::$builtin))||+)
            }
        }
        unsafe impl Parameter for Option<&$type> {
            fn matches(ty: &Il2CppType) -> bool {
                ty.raw().byref() != 0 && ($(ty.is_builtin(Builtin::$builtin))||+)
            }
        }

        unsafe impl Return for $type {
            type Type = $type;

            fn matches(ty: &Il2CppType) -> bool {
                ty.raw().byref() == 0 && ($(ty.is_builtin(Builtin::$builtin))||+)
            }

            fn from_object(object: Option<&mut Il2CppObject>) -> Self {
                unsafe { *(object.unwrap() as *mut Il2CppObject as *const $type) }
            }
        }
        unsafe impl Return for Option<&$type> {
            fn matches(ty: &Il2CppType) -> bool {
                ty.raw().byref() != 0 && ($(ty.is_builtin(Builtin::$builtin))||+)
            }
        }
    };
}

impl_builtin_value!(u8, Byte);
impl_builtin_value!(i8, SByte);
impl_builtin_value!(u16, UShort, Char);
impl_builtin_value!(i16, Short);
impl_builtin_value!(u32, UInt);
impl_builtin_value!(i32, Int);
impl_builtin_value!(u64, ULong);
impl_builtin_value!(i64, Long);
impl_builtin_value!(f32, Single);
impl_builtin_value!(f64, Double);
impl_builtin_value!(bool, Bool);

quest_hook_proc_macros::impl_arguments_parameters!(1..=16);
