use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use paste::paste;
use std::ffi::c_void;
use std::lazy::SyncLazy;
use std::os::raw::c_char;

use super::{
    FieldInfo, Il2CppAssembly, Il2CppClass, Il2CppDomain, Il2CppException, Il2CppImage,
    Il2CppObject, Il2CppString, Il2CppType, MethodInfo,
};

macro_rules! define_functions {
    ( $( fn $name:ident ( $( $arg_name:ident : $arg_type:ty ),* ) $( -> $return:ty )? ; )+ ) => {
        paste! {
            #[derive(WrapperApi)]
            struct LibIl2Cpp {
                $(
                    [<il2cpp_ $name>]: unsafe extern "C" fn ( $( $arg_name : $arg_type ),* ) $( -> $return )?
                ),*
            }
        }

        static LIBIL2CPP: SyncLazy<Container<LibIl2Cpp>> =
            SyncLazy::new(|| unsafe { Container::load("libil2cpp.so") }.unwrap());

        paste! {
            $(
                #[allow(clippy::missing_safety_doc)]
                pub unsafe fn $name ( $( $arg_name : $arg_type ),* ) $( -> $return )? {
                    LIBIL2CPP.[<il2cpp_ $name>]( $( $arg_name ),* )
                }
            )+
        }
    }
}

define_functions! {
    fn domain_get() -> &'static Il2CppDomain;
    fn domain_get_assemblies(domain: &Il2CppDomain, size: &mut usize) -> &'static [&'static Il2CppAssembly];
    fn assembly_get_image(assembly: &Il2CppAssembly) -> Option<&'static Il2CppImage>;
    fn class_from_name(image: &Il2CppImage, namespace: *const c_char, name: *const c_char) -> Option<&'static Il2CppClass>;
    fn class_from_il2cpp_type(ty: &Il2CppType) -> &'static Il2CppClass;
    fn class_is_assignable_from(class: &Il2CppClass, other_class: &Il2CppClass) -> bool;
    fn class_get_method_from_name(class: &Il2CppClass, name: *const c_char, args_count: u32) -> Option<&'static MethodInfo>;
    fn field_set_value(obj: &mut Il2CppObject, field: &FieldInfo, value: *const c_void);
    fn field_get_value(obj: &mut Il2CppObject, field: &FieldInfo, value: *const c_void);
    fn type_get_name(ty: &Il2CppType) -> *const c_char;
    fn runtime_invoke(method: &MethodInfo, instance: *mut c_void, params: *mut *mut c_void, exception: &mut Option<&Il2CppException>) -> Option<&'static mut Il2CppObject>;
    fn string_new_len(s: *const char, len: u32) -> Option<&'static Il2CppString>;
}
