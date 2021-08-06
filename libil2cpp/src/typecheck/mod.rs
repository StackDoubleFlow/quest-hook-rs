pub mod callee;
pub mod caller;
pub mod ty;

use std::ffi::c_void;

use crate::{Argument, Arguments, MethodInfo, Parameter, Parameters};

quest_hook_proc_macros::impl_arguments_parameters!(1..=16);
