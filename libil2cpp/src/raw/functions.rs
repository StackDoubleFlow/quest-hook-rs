use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use paste::paste;
use std::ffi::c_void;
use std::lazy::SyncLazy;
use std::os::raw::c_char;

use super::{
    FieldInfo, Il2CppArray, Il2CppAssembly, Il2CppClass, Il2CppDomain, Il2CppException,
    Il2CppImage, Il2CppObject, Il2CppReflectionMethod, Il2CppReflectionType, Il2CppString,
    Il2CppType, MethodInfo,
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
                #[allow(missing_docs, clippy::missing_safety_doc)]
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
    fn class_get_type(class: &Il2CppClass) -> &'static Il2CppType;
    fn field_set_value(obj: &mut Il2CppObject, field: &FieldInfo, value: *const c_void);
    fn field_get_value(obj: &mut Il2CppObject, field: &FieldInfo, value: *const c_void);
    fn method_get_object(method: &MethodInfo, refclass: Option<&Il2CppClass>) -> &'static Il2CppReflectionMethod;
    fn method_get_from_reflection(method: &Il2CppReflectionMethod) -> &'static MethodInfo;
    fn method_is_generic(method: &MethodInfo) -> bool;
    fn array_new(elem_type: &Il2CppClass, length: usize) -> *mut Il2CppArray;
    fn type_get_name(ty: &Il2CppType) -> *const c_char;
    fn type_get_object(ty: &Il2CppType) -> &'static Il2CppReflectionType;
    fn runtime_invoke(method: &MethodInfo, instance: *mut c_void, params: *mut *mut c_void, exception: &mut Option<&mut Il2CppException>) -> Option<&'static mut Il2CppObject>;
    fn string_new_len(s: *const char, len: u32) -> &'static Il2CppString;
    fn raise_exception(exc: &Il2CppException) -> !;
    fn object_unbox(obj: &mut Il2CppObject) -> *mut c_void;
}
