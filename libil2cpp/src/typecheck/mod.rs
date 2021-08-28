pub mod callee;
pub mod caller;
pub mod generic;
pub mod ty;

use std::ffi::c_void;

use crate::{
    raw, Argument, Arguments, Generics, Il2CppReflectionType, MethodInfo, Parameter, Parameters,
    Type, WrapRaw,
};

quest_hook_proc_macros::impl_arguments_parameters!(1..=16);
quest_hook_proc_macros::impl_generics!(1..=8);
