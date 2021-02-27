//! libil2cpp.so functions.
//!
//! This module contains functions found in libil2cpp to perform various tasks
//! such as finding a class or method.

use crate::types::{Il2CppAssembly, Il2CppClass, Il2CppDomain, Il2CppImage, MethodInfo};
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use paste::paste;
use std::lazy::SyncLazy;
use std::os::raw::c_char;

macro_rules! define_functions {
    ( $( pub fn $name:ident ( $( $arg_name:ident : $arg_type:ty ),* ) $( -> $return:ty )* ; )+ ) => {
        paste! {
            #[derive(WrapperApi)]
            struct LibIl2Cpp {
                $(
                    [<il2cpp_ $name>]: extern "C" fn ( $( $arg_name : $arg_type ),* ) $( -> $return )*
                ),*
            }
        }

        static LIBIL2CPP: SyncLazy<Container<LibIl2Cpp>> = SyncLazy::new(|| unsafe { Container::load("libil2cpp.so") }.unwrap());

        paste! {
            $(
                pub fn $name ( $( $arg_name : $arg_type ),* ) $( -> $return )* {
                    LIBIL2CPP.[<il2cpp_ $name>]( $( $arg_name ),* )
                }
            )+
        }
    }
}

define_functions! {
    pub fn class_get_method_from_name(class: &Il2CppClass, name: *const c_char, args_count: u32) -> Option<&'static MethodInfo>;
    pub fn domain_get() -> &'static Il2CppDomain;
    pub fn domain_get_assemblies(domain: &Il2CppDomain, size: &mut usize) -> &'static [&'static Il2CppAssembly];
    pub fn assembly_get_image(assembly: &Il2CppAssembly) -> Option<&'static Il2CppImage>;
    pub fn class_from_name(image: &Il2CppImage, namespace: *const c_char, name: *const c_char) -> Option<&'static Il2CppClass>;
}
