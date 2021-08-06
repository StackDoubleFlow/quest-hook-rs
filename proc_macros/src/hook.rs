use heck::{CamelCase, SnakeCase};
use proc_macro::TokenStream;
use proc_macro2::{Group, TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Abi, Attribute, Error, FnArg, Generics, Ident, ItemFn, LitStr, Pat, PatType, ReturnType, Token,
    Type, TypeTuple,
};

pub fn expand(args: Punctuated<LitStr, Token![,]>, input: ItemFn) -> Result<TokenStream, Error> {
    let metadata = Metadata::new(args, input)?;
    metadata.validate()?;

    let outer_fn = metadata.outer_fn();
    let struct_def = metadata.struct_def();
    let struct_impl = metadata.struct_impl();
    let static_def = metadata.static_def();
    let trait_impl = metadata.trait_impl();

    let ts = quote! {
        #outer_fn
        #struct_def
        #struct_impl
        #static_def
        #trait_impl
    };
    Ok(ts.into())
}

pub struct Metadata {
    namespace: String,
    class: String,
    method: String,
    input: ItemFn,
}

impl Metadata {
    fn new(args: Punctuated<LitStr, Token![,]>, input: ItemFn) -> Result<Self, Error> {
        let mut iter = args.iter().map(LitStr::value);

        macro_rules! parse {
            () => {
                match iter.next() {
                    Some(a) => a,
                    None => return Err(Error::new_spanned(&args, "Expected 3 arguments")),
                }
            };
        }

        let namespace = parse!();
        let class = parse!();
        let method = parse!();

        if iter.next().is_some() {
            return Err(Error::new_spanned(&args, "Expected 3 arguments"));
        }

        Ok(Self {
            namespace,
            class,
            method,
            input,
        })
    }

    fn validate(&self) -> Result<(), Error> {
        if let Some(constness) = self.input.sig.constness {
            return Err(Error::new_spanned(constness, "Cannot hook const functions"));
        }

        if let Some(asyncness) = self.input.sig.asyncness {
            return Err(Error::new_spanned(asyncness, "Cannot hook async functions"));
        }

        if let Some(Abi {
            name: Some(abi), ..
        }) = &self.input.sig.abi
        {
            if abi.value() != "C" {
                return Err(Error::new_spanned(
                    abi,
                    "Cannot hook functions with non-C ABIs",
                ));
            }
        }

        let generics = &self.input.sig.generics;
        if let Generics {
            lt_token: Some(_),
            gt_token: Some(_),
            ..
        } = generics
        {
            return Err(Error::new_spanned(
                generics,
                "Cannot hook generic functions",
            ));
        }

        for (i, arg) in self.input.sig.inputs.iter().enumerate() {
            match arg {
                FnArg::Receiver(_) => {
                    return Err(Error::new_spanned(
                        arg,
                        "Cannot hook functions taking a `self` parameter",
                    ))
                }
                FnArg::Typed(PatType {
                    pat: box Pat::Ident(ident),
                    ..
                }) if ident.ident == "self" => {
                    return Err(Error::new_spanned(
                        arg,
                        "Cannot hook functions taking a `self` parameter",
                    ))
                }
                FnArg::Typed(PatType { attrs, .. }) if i != 0 => {
                    let has_this_attr = attrs.iter().any(|a| attr_is(a, "this"));
                    if has_this_attr {
                        return Err(Error::new_spanned(
                            arg,
                            "`this` can only be the first parameter",
                        ));
                    }
                }
                _ => (),
            }
        }

        if let Some(variadic) = &self.input.sig.variadic {
            return Err(Error::new_spanned(
                variadic,
                "Cannot hook variadic functions",
            ));
        }

        Ok(())
    }

    fn hook_name(&self) -> &Ident {
        &self.input.sig.ident
    }

    fn struct_name(&self) -> Ident {
        let hook_name = self.hook_name().to_string();
        let struct_name = hook_name.to_camel_case();
        format_ident!("{}Struct", struct_name)
    }

    fn fn_name(&self) -> Ident {
        let hook_name = self.hook_name().to_string();
        let fn_name = hook_name.to_snake_case();
        format_ident!("_{}_fn", fn_name)
    }

    fn filtered_attrs(&self) -> impl Iterator<Item = &'_ Attribute> + '_ {
        self.input.attrs.iter().filter(|a| !attr_is(a, "hook"))
    }

    fn this(&self) -> Option<&PatType> {
        let first_input = match self.input.sig.inputs.iter().next()? {
            FnArg::Typed(arg) => arg,
            _ => unreachable!(),
        };

        let is_this = match &first_input.pat {
            box Pat::Ident(ident) if ident.ident == "this" => true,
            _ => first_input.attrs.iter().any(|a| attr_is(a, "this")),
        };
        if !is_this {
            return None;
        }

        Some(first_input)
    }

    fn has_this(&self) -> bool {
        self.this().is_some()
    }

    fn this_ty(&self) -> Option<&Type> {
        self.this().map(|this| &*this.ty)
    }

    fn params(&self) -> impl Iterator<Item = &'_ PatType> + '_ {
        let skip = if self.has_this() { 1 } else { 0 };

        self.input
            .sig
            .inputs
            .iter()
            .skip(skip)
            .map(|arg| match arg {
                FnArg::Typed(arg) => arg,
                _ => unreachable!(),
            })
    }

    fn params_ty(&self) -> impl Iterator<Item = &'_ Type> + '_ {
        self.params().map(|param| &*param.ty)
    }

    fn return_ty(&self) -> Type {
        match &self.input.sig.output {
            ReturnType::Default => unit_ty(),
            ReturnType::Type(_, box ty) => ty.clone(),
        }
    }

    fn typechecking_this_ty(&self) -> Type {
        self.this_ty().cloned().unwrap_or_else(unit_ty)
    }

    fn typechecking_params_ty(&self) -> TokenStream2 {
        let params_ty = self.params_ty();
        quote!((#(#params_ty,)*))
    }

    fn actual_this_ty(&self) -> Option<TokenStream2> {
        self.this_ty().map(
            |t| quote_spanned!(t.span()=> <#t as ::quest_hook::libil2cpp::ThisParameter>::Actual),
        )
    }

    fn actual_params_ty(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.params_ty()
            .map(|t| quote_spanned!(t.span()=> <#t as ::quest_hook::libil2cpp::Parameter>::Actual))
    }

    fn actual_return_ty(&self) -> TokenStream2 {
        let return_ty = self.return_ty();
        quote_spanned!(return_ty.span()=> <#return_ty as ::quest_hook::libil2cpp::Return>::Actual)
    }

    fn this_ident(&self) -> Option<Ident> {
        self.this().map(|_| format_ident!("this"))
    }

    fn params_ident(&self) -> impl Iterator<Item = Ident> + '_ {
        self.params()
            .enumerate()
            .map(|(i, _)| format_ident!("p{}", i))
    }

    fn inner_fn(&self) -> TokenStream2 {
        let attrs = self.filtered_attrs();
        let unsafety = &self.input.sig.unsafety;
        let inputs = &self.input.sig.inputs;
        let return_ty = self.return_ty();
        let block = &self.input.block;

        quote! {
            #(#attrs) *
            #unsafety fn inner(#inputs) -> #return_ty #block
        }
    }

    fn outer_fn(&self) -> TokenStream2 {
        let unsafety = self.input.sig.unsafety;
        let name = self.fn_name();
        let return_ty = self.actual_return_ty();
        let inner_fn = self.inner_fn();

        let this_param = self
            .this_ident()
            .zip(self.actual_this_ty())
            .map(|(i, t)| quote!(#i: #t,));

        let this_arg = self
            .this_ident()
            .map(|i| quote!(::quest_hook::libil2cpp::ThisParameter::from_actual(#i),));

        let params_params = self
            .params_ident()
            .zip(self.actual_params_ty())
            .map(|(i, t)| quote!(#i: #t,));

        let params_args = self
            .params_ident()
            .map(|i| quote!(::quest_hook::libil2cpp::Parameter::from_actual(#i),));

        quote! {
            pub #unsafety extern "C" fn #name(#this_param #(#params_params)*) -> #return_ty {
                #inner_fn
                let r = inner(#this_arg #(#params_args)*);
                ::quest_hook::libil2cpp::Return::into_actual(r)
            }
        }
    }

    fn struct_def(&self) -> TokenStream2 {
        let vis = &self.input.vis;
        let struct_name = self.struct_name();

        quote! {
            #vis struct #struct_name {
                original: ::std::sync::atomic::AtomicPtr<()>,
            }
        }
    }

    fn static_def(&self) -> TokenStream2 {
        let vis = &self.input.vis;
        let name = self.hook_name();
        let struct_name = self.struct_name();

        quote! {
            #[allow(non_upper_case_globals)]
            #vis static #name: #struct_name = #struct_name {
                original: ::std::sync::atomic::AtomicPtr::new(::std::ptr::null_mut()),
            };
        }
    }

    fn install_fn(&self) -> TokenStream2 {
        let vis = &self.input.vis;

        let namespace = &self.namespace;
        let class = &self.class;
        let method = &self.method;

        let this_ty = self.typechecking_this_ty();
        let params_ty = self.typechecking_params_ty();
        let return_ty = self.return_ty();

        let fn_name = self.fn_name();

        quote! {
            #vis fn install(&self) -> Result<(), quest_hook::HookInstallError> {
                use ::std::ptr::null_mut;
                use ::std::sync::atomic::Ordering;
                use ::quest_hook::HookInstallError;
                use ::quest_hook::inline_hook::A64HookFunction;
                use ::quest_hook::libil2cpp::{Il2CppClass, WrapRaw};

                if !self.original.load(Ordering::Relaxed).is_null() {
                    return Err(HookInstallError::AlreadyInstalled);
                }

                let class = match Il2CppClass::find(#namespace, #class) {
                    Some(class) => class,
                    None => return Err(HookInstallError::ClassNotFound),
                };
                let method = match class.find_method_callee::<#this_ty, #params_ty, #return_ty>(#method) {
                    Some(method) => method,
                    None => return Err(HookInstallError::MethodNotFound),
                };

                let mut temp = null_mut();
                unsafe {
                    A64HookFunction(
                        method.raw().methodPointer.unwrap() as *mut _,
                        #fn_name as *mut _,
                        &mut temp,
                    );
                }

                match self.original.compare_exchange(
                    null_mut(),
                    temp as *mut _,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => Ok(()),
                    Err(_) => panic!("method hooked twice"),
                }
            }
        }
    }

    fn original_ty(&self) -> TokenStream2 {
        let this_ty = self.actual_this_ty().map(|t| quote!(#t,));
        let params_ty = self.actual_params_ty().map(|t| quote!(#t,));
        let return_ty = self.actual_return_ty();

        quote!(extern "C" fn(#this_ty #(#params_ty)*) -> #return_ty)
    }

    fn original_fn(&self) -> TokenStream2 {
        let vis = &self.input.vis;
        let return_ty = self.return_ty();
        let original_ty = self.original_ty();

        let this_param = self
            .this_ident()
            .zip(self.this_ty())
            .map(|(i, t)| quote!(#i: #t,));

        let this_arg = self
            .this_ident()
            .map(|i| quote!(::quest_hook::libil2cpp::ThisParameter::into_actual(#i),));

        let params_params = self
            .params_ident()
            .zip(self.params_ty())
            .map(|(i, t)| quote!(#i: #t,));

        let params_args = self
            .params_ident()
            .map(|i| quote!(::quest_hook::libil2cpp::Parameter::into_actual(#i),));

        quote! {
            #vis fn original(&self, #this_param #(#params_params)*) -> #return_ty {
                use ::std::mem::transmute;
                use ::std::sync::atomic::Ordering;

                let ptr = self.original.load(Ordering::Relaxed);
                let original = unsafe { transmute::<*mut (), Option<#original_ty>>(ptr) };
                let original = original.expect("hook is not installed");

                let r = original(#this_arg #(#params_args)*);
                ::quest_hook::libil2cpp::Return::from_actual(r)
            }
        }
    }

    fn struct_impl(&self) -> TokenStream2 {
        let struct_name = self.struct_name();
        let install_fn = self.install_fn();
        let original_fn = self.original_fn();

        quote! {
            impl #struct_name {
                #install_fn
                #original_fn
            }
        }
    }

    fn trait_impl(&self) -> TokenStream2 {
        let struct_name = self.struct_name();

        let namespace = &self.namespace;
        let class = &self.class;
        let method = &self.method;

        let this_ty = staticify(self.typechecking_this_ty());
        let params_ty = staticify(self.typechecking_params_ty());
        let return_ty = staticify(self.return_ty());

        let fn_name = self.fn_name();

        quote! {
            impl ::quest_hook::Hook for #struct_name {
                type This = #this_ty;
                type Parameters = #params_ty;
                type Return = #return_ty;

                const NAMESPACE: &'static str = #namespace;
                const CLASS_NAME: &'static str = #class;
                const METHOD_NAME: &'static str = #method;

                fn install(&self) -> Result<(), ::quest_hook::HookInstallError> {
                    self.install()
                }

                fn original(&self) -> *mut () {
                    self.original.load(::std::sync::atomic::Ordering::Relaxed)
                }
                fn hook(&self) -> *mut () {
                    #fn_name as _
                }
            }
        }
    }
}

fn unit_ty() -> Type {
    Type::Tuple(TypeTuple {
        paren_token: Default::default(),
        elems: Default::default(),
    })
}

fn attr_is(attr: &Attribute, ident: &str) -> bool {
    matches!(attr.path.get_ident(), Some(ai) if ai == ident)
}

fn staticify(tokens: impl ToTokens) -> TokenStream2 {
    let mut ts = TokenStream2::new();
    let mut iter = tokens.to_token_stream().into_iter().peekable();
    while let Some(tt) = iter.next() {
        match &tt {
            TokenTree2::Group(g) => {
                let delimiter = g.delimiter();
                let stream = staticify(g.stream());
                ts.extend_one(TokenTree2::Group(Group::new(delimiter, stream)))
            }
            TokenTree2::Punct(p) if p.as_char() == '&' => match iter.peek() {
                Some(TokenTree2::Punct(p)) if p.as_char() == '\'' => ts.extend_one(tt),
                _ => ts.extend_one(quote_spanned!(tt.span()=> &'static)),
            },
            _ => ts.extend_one(tt),
        }
    }
    ts
}
