pub mod argument;
pub mod parameter;
pub mod ty;

use std::ffi::c_void;

use super::{Argument, Arguments, Il2CppType, Parameter, Parameters, Type, WrapRaw};

macro_rules! impl_argument_parameter_value {
    ($type:ty) => {
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

        unsafe impl Parameter for $type {
            type Type = $type;

            fn matches(ty: &Il2CppType) -> bool {
                let self_ty = <$type as Type>::class().raw().byval_arg;
                unsafe { self_ty.data.klassIndex == ty.raw().data.klassIndex }
            }
        }
    };
}

impl_argument_parameter_value!(u8);
impl_argument_parameter_value!(i8);
impl_argument_parameter_value!(u16);
impl_argument_parameter_value!(i16);
impl_argument_parameter_value!(u32);
impl_argument_parameter_value!(i32);
impl_argument_parameter_value!(u64);
impl_argument_parameter_value!(i64);
impl_argument_parameter_value!(usize);
impl_argument_parameter_value!(isize);
impl_argument_parameter_value!(f32);
impl_argument_parameter_value!(f64);
impl_argument_parameter_value!(bool);

quest_hook_proc_macros::impl_arguments_parameters!(1..=16);
