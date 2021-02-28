#![warn(rust_2018_idioms)]
#![cfg_attr(feature = "strict", deny(warnings))]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Error, FnArg, ItemFn, LitStr, Pat, Token};

/// Creates an inline hook at a C# method.
///
/// # Panics
///
/// * `install` will panic if the class or method was not found.
/// * `original` will panic if the hook has not yet been installed.
///
/// # Examples
///
/// ```no_run
/// use quest_hook::inline_hook::hook;
/// use quest_hook::libil2cpp::Il2CppObject;
/// use log::info;
///
/// #[hook("", "MainSettingsModelSO", "OnEnable")]
/// fn on_enable(this: &Il2CppObject) {
///     info!("MainSettingsModelSO.OnEnable was called!");
///
///     on_enable.original(this); // Call the original C# method
/// }
///
/// #[no_mangle]
/// pub extern "C" fn load() {
///     info!("Installing hooks!");
///
///     on_enable.install(); // Install the hook
///
///     info!("Installed  hooks!");
/// }
/// ```
#[proc_macro_attribute]
pub fn hook(attr: TokenStream, item: TokenStream) -> TokenStream {
    let punctuated_args =
        parse_macro_input!(attr with Punctuated<LitStr, Token![,]>::parse_separated_nonempty);
    let input = parse_macro_input!(item as ItemFn);

    match create_hook(punctuated_args, input) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}

fn create_hook(
    punctuated_args: Punctuated<LitStr, Token![,]>,
    input: ItemFn,
) -> Result<TokenStream, Error> {
    let args: Vec<String> = punctuated_args.iter().map(LitStr::value).collect();
    let (namespace, class, method) = match args.as_slice() {
        [n, c, m] => (n, c, m),
        _ => {
            let msg = format!("Expected 3 arguments, found {}", args.len());
            return Err(Error::new_spanned(punctuated_args, msg));
        }
    };

    let ItemFn { sig, block, .. } = input;

    let name = sig.ident;
    let return_type = sig.output;

    let hook_name = format_ident!("{}_hook", name);
    let hook_args = sig.inputs;

    let mut num_hook_args = 0;
    for hook_arg in hook_args.iter() {
        match match hook_arg {
            FnArg::Typed(arg_type) => &*arg_type.pat,
            FnArg::Receiver(_) => {
                let msg = "Hook argument cannot be `self`";
                return Err(Error::new_spanned(hook_arg, msg));
            }
        } {
            // `il2cpp_class_get_method_from_name` does not count `this` in its argument count
            Pat::Ident(pat_ident) if pat_ident.ident == "this" => {}
            _ => num_hook_args += 1,
        }
    }

    let hook_struct_name = format_ident!("{}_Struct", name);

    let mut hook_args_untyped: Punctuated<Pat, Token![,]> = Punctuated::new();
    for arg in &hook_args {
        if let FnArg::Typed(arg) = arg {
            hook_args_untyped.push((*arg.pat).clone());
        }
    }

    let tokens = quote! {
        pub extern "C" fn #hook_name ( #hook_args ) #return_type #block

        #[allow(non_camel_case_types)]
        struct #hook_struct_name {
            original: ::std::sync::atomic::AtomicPtr<()>,
            namespace: &'static str,
            class_name: &'static str,
            method_name: &'static str,
            parameters_count: usize,
        }

        impl #hook_struct_name {
            fn install(&self) {
                let class = ::quest_hook::libil2cpp::Il2CppClass::find(self.namespace, self.class_name).expect("Class not found");
                let method = class.find_method(self.method_name, self.parameters_count).expect("Method not found");
                let mut temp = ::std::ptr::null_mut();

                unsafe {
                    ::quest_hook::inline_hook::A64HookFunction(
                        ::std::mem::transmute::<unsafe extern "C" fn(), *mut ::std::ffi::c_void>(method.methodPointer.unwrap()),
                        ::std::mem::transmute::<extern "C" fn( #hook_args ) #return_type, *mut ::std::ffi::c_void>( #hook_name ),
                        &mut temp,
                    );

                    self.original.store(
                        ::std::mem::transmute::<*mut ::std::ffi::c_void, *mut ()>(temp),
                        ::std::sync::atomic::Ordering::Relaxed
                    );
                }
            }

            fn original(&self, #hook_args ) #return_type {
                let ptr = self.original.load(::std::sync::atomic::Ordering::Relaxed);
                let original = unsafe {
                    ::std::mem::transmute::<*const (), Option<extern "C" fn( #hook_args ) #return_type >>(ptr)
                };
                (original.expect("Hook is not installed"))( #hook_args_untyped )
            }

            fn hook(&self, #hook_args ) #return_type {
                #hook_name( #hook_args_untyped )
            }
        }

        impl ::quest_hook::Hook for #hook_struct_name {
            fn install(&self) {
                self.install()
            }

            fn namespace(&self) -> &'static str {
                self.namespace
            }

            fn class_name(&self) -> &'static str {
                self.class_name
            }

            fn method_name(&self) -> &'static str {
                self.method_name
            }

            fn parameters_count(&self) -> usize {
                self.parameters_count
            }

            fn hook(&self) -> *mut () {
                ::std::mem::transmute::<extern "C" fn( #hook_args ) #return_type, *mut ()>( #hook_name )
            }

            fn original(&self) -> *mut () {
                self.original.load(::std::sync::atomic::Ordering::Relaxed)
            }
        }

        #[allow(non_upper_case_globals)]
        static #name: #hook_struct_name = #hook_struct_name {
            original: ::std::sync::atomic::AtomicPtr::new(::std::ptr::null_mut()),
            namespace: #namespace,
            class_name: #class,
            method_name: #method,
            parameters_count: #num_hook_args as usize
        };
    };

    Ok(tokens.into())
}
