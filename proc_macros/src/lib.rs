#![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]
#![cfg_attr(feature = "strict", deny(warnings))]
#![doc(html_root_url = "https://stackdoubleflow.github.io/quest-hook-rs/quest_hook_proc_macros")]

//! Procedural macros for the quest_hook crate

use std::num::NonZeroUsize;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Error, Expr, ExprLit, ExprRange, FnArg, Index, ItemFn, Lit, LitStr, Pat,
    RangeLimits, ReturnType, Token, Type,
};

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
/// use tracing_android::tracing::info;
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
    let typecheck_return_type = match &return_type {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    let hook_name = format_ident!("{}_hook", name);
    let hook_args = sig.inputs;

    let mut this_arg_type = None;

    let mut num_hook_args: usize = 0;
    for hook_arg in &hook_args {
        let arg_type = match hook_arg {
            FnArg::Typed(arg_type) => arg_type,
            FnArg::Receiver(_) => {
                let msg = "Hook argument cannot be `self`";
                return Err(Error::new_spanned(hook_arg, msg));
            }
        };

        match &*arg_type.pat {
            // `il2cpp_class_get_method_from_name` does not count `this` in its argument count
            Pat::Ident(pat_ident) if pat_ident.ident == "this" => {
                if this_arg_type.is_some() {
                    let msg = "There cannot be more than one `this` argument.";
                    return Err(Error::new_spanned(hook_arg, msg));
                }
                if num_hook_args > 0 {
                    let msg = "`this` must be the first argument.";
                    return Err(Error::new_spanned(hook_arg, msg));
                }
                this_arg_type = Some(arg_type.ty.clone());
            }
            _ => num_hook_args += 1,
        }
    }

    let hook_struct_name = format_ident!("{}_Struct", name);

    let mut hook_args_untyped: Punctuated<Pat, Token![,]> = Punctuated::new();
    let mut typecheck_arg_types: Punctuated<Type, Token![,]> = Punctuated::new();
    for arg in &hook_args {
        if let FnArg::Typed(arg) = arg {
            hook_args_untyped.push((*arg.pat).clone());
            match &*arg.pat {
                Pat::Ident(pat_ident) if pat_ident.ident == "this" => continue,
                _ => typecheck_arg_types.push((*arg.ty).clone()),
            }
        }
    }

    let typecheck_this_type = match &this_arg_type {
        None => quote! { () },
        Some(ty) => quote! { #ty },
    };

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
                use ::quest_hook::libil2cpp::WrapRaw;

                let class = ::quest_hook::libil2cpp::Il2CppClass::find(self.namespace, self.class_name).expect("Class not found");
                let method = class.find_method_callee::<
                    #typecheck_this_type,
                    ( #typecheck_arg_types ),
                    #typecheck_return_type,
                    #num_hook_args
                >(self.method_name).expect("Method not found");
                let mut temp = ::std::ptr::null_mut();

                unsafe {
                    ::quest_hook::inline_hook::A64HookFunction(
                        method.raw().methodPointer.unwrap() as *mut _,
                        #hook_name as *mut _,
                        &mut temp,
                    );
                }

                self.original.store(
                    temp as *mut _,
                    ::std::sync::atomic::Ordering::Relaxed
                );
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
                #hook_name as *mut _
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

#[doc(hidden)]
#[proc_macro]
pub fn impl_arguments_parameters(input: TokenStream) -> TokenStream {
    let range = parse_macro_input!(input as ExprRange);
    match create_impl_arguments_parameters(range) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}

fn create_impl_arguments_parameters(range: ExprRange) -> Result<TokenStream, Error> {
    let span = range.span();

    let start = range
        .from
        .ok_or_else(|| Error::new(span, "Tuple length range must have a lower bound"))?;
    let start = parse_range_bound(*start)?;

    let end = range
        .to
        .ok_or_else(|| Error::new(span, "Tuple length range must have an upper bound"))?;
    let end = parse_range_bound(*end)?;

    let range = match range.limits {
        RangeLimits::HalfOpen(_) if end <= start => {
            return Err(Error::new(span, "Tuple length range must be valid"))
        }
        RangeLimits::HalfOpen(_) => start..end,

        RangeLimits::Closed(_) if end < start => {
            return Err(Error::new(span, "Tuple length range must be valid"))
        }
        RangeLimits::Closed(_) => start..(end + 1),
    };

    let mut ts = TokenStream::new();
    for n in range {
        let generic_params_argument = (1..=n).map(|n| format_ident!("A{}", n));
        let matches_argument = generic_params_argument
            .clone()
            .enumerate()
            .map(|(n, gp)| quote!(<#gp>::matches(args[#n].ty())));
        let invokables = (0..n).map(Index::from).map(|n| quote!(self.#n.invokable()));

        let generic_params_parameter = (1..=n).map(|n| format_ident!("P{}", n));
        let matches_parameter = generic_params_parameter
            .clone()
            .enumerate()
            .map(|(n, gp)| quote!(<#gp>::matches(params[#n].ty())));

        let generic_params_argument_tuple = generic_params_argument.clone();
        let generic_params_argument_where = generic_params_argument.clone();
        let generic_params_argument_type = generic_params_argument.clone();

        let generic_params_parameter_tuple = generic_params_parameter.clone();
        let generic_params_parameter_where = generic_params_parameter.clone();
        let generic_params_parameter_type = generic_params_parameter.clone();

        let impl_ts = quote! {
            unsafe impl<#(#generic_params_argument),*> Arguments<#n> for (#(#generic_params_argument_tuple,)*)
            where
                #(#generic_params_argument_where: Argument),*
            {
                type Type = (#(#generic_params_argument_type::Type,)*);

                fn matches(args: &[ParameterInfo]) -> bool {
                    args.len() == #n #( && #matches_argument)*
                }

                fn invokable(&self) -> [*mut c_void; #n] {
                    [#(#invokables),*]
                }
            }

            unsafe impl<#(#generic_params_parameter),*> Parameters<#n> for (#(#generic_params_parameter_tuple,)*)
            where
                #(#generic_params_parameter_where: Parameter),*
            {
                type Type = (#(#generic_params_parameter_type::Type,)*);

                fn matches(params: &[ParameterInfo]) -> bool {
                    params.len() == #n #( && #matches_parameter)*
                }
            }
        };
        ts.extend(TokenStream::from(impl_ts));
    }

    Ok(ts)
}

fn parse_range_bound(bound: Expr) -> Result<usize, Error> {
    let bound: NonZeroUsize = match bound {
        syn::Expr::Lit(ExprLit {
            lit: Lit::Int(n), ..
        }) => n.base10_parse()?,
        _ => {
            return Err(Error::new(
                bound.span(),
                "Tuple length bound must be an integer",
            ))
        }
    };
    Ok(bound.get())
}
