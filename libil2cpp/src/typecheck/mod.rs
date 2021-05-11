pub mod callee;
pub mod caller;
pub mod ty;

use std::ffi::c_void;

use crate::{
    Argument, Arguments, Builtin, Il2CppType, Parameter, ParameterInfo, Parameters, Type, WrapRaw,
};

macro_rules! impl_argument_parameter_value {
    ($type:ty, $($builtin:ident),+) => {
        unsafe impl Argument for $type {
            type Type = $type;

            fn matches(ty: &Il2CppType) -> bool {
                let self_ty = <$type as Type>::class().raw().byval_arg;
                unsafe { self_ty.data.klassIndex == ty.raw().data.klassIndex }
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
                let self_ty = <$type as Type>::class().raw().byval_arg;
                unsafe { self_ty.data.klassIndex == ty.raw().data.klassIndex }
            }
        }
        unsafe impl Parameter for Option<&$type> {
            fn matches(ty: &Il2CppType) -> bool {
                ty.raw().byref() != 0 && ($(ty.is_builtin(Builtin::$builtin))||+)
            }
        }
    };
}

impl_argument_parameter_value!(u8, Byte);
impl_argument_parameter_value!(i8, SByte);
impl_argument_parameter_value!(u16, UShort, Char);
impl_argument_parameter_value!(i16, Short);
impl_argument_parameter_value!(u32, UInt);
impl_argument_parameter_value!(i32, Int);
impl_argument_parameter_value!(u64, ULong);
impl_argument_parameter_value!(i64, Long);
impl_argument_parameter_value!(f32, Single);
impl_argument_parameter_value!(f64, Double);
impl_argument_parameter_value!(bool, Bool);

quest_hook_proc_macros::impl_arguments_parameters!(1..=16);
