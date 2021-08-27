use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Brace;
use syn::{
    AngleBracketedGenericArguments, Block, Error, GenericArgument, Ident, Path, PathArguments,
    Result, Token, Type, TypeGroup, TypeParen, TypePath,
};

pub struct Input {
    path: Path,
    ty: Type,
    namespace: String,
    class: String,
    rust_generics: Vec<Type>,
    cs_generics: Vec<Type>,
    class_getter: Option<Block>,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        input.parse::<Token![in]>()?;
        let path = input.parse()?;
        input.parse::<Token![for]>()?;

        let mut ty = input.parse()?;

        input.parse::<Token![=>]>()?;

        let namespace_and_class: Punctuated<Ident, Token![.]> =
            Punctuated::parse_separated_nonempty(input)?;
        let generics: Option<AngleBracketedGenericArguments> = if input.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            None
        };

        let class_getter = if input.peek(Brace) {
            Some(input.parse()?)
        } else {
            None
        };

        let namespace_and_class: Vec<Ident> = namespace_and_class.into_iter().collect();
        let (class, namespace) = namespace_and_class
            .split_last()
            .ok_or_else(|| Error::new(Span::call_site(), "no C# class specified"))?;
        let namespace = namespace
            .iter()
            .map(Ident::to_string)
            .collect::<Vec<_>>()
            .join(".");
        let class = class.to_string();

        let rust_generics = extract_generics(&mut ty)
            .map(collect_generics)
            .transpose()?
            .unwrap_or_default();
        let cs_generics = generics
            .map(collect_generics)
            .transpose()?
            .unwrap_or_default();

        if rust_generics.len() != cs_generics.len() {
            return Err(Error::new(
                Span::call_site(),
                "mismatched Rust and C# generics",
            ));
        }

        if let Some(g) = cs_generics.iter().find(|g| !rust_generics.contains(g)) {
            return Err(Error::new_spanned(g, "mismatched Rust and C# generics"));
        }

        Ok(Self {
            path,
            ty,
            namespace,
            class,
            rust_generics,
            cs_generics,
            class_getter,
        })
    }
}

#[derive(Clone, Copy)]
pub enum Semantics {
    Reference,
    Value,
}

pub fn expand(input: &Input, semantics: Semantics) -> TokenStream {
    let header = input.impl_header();

    let held = match semantics {
        Semantics::Reference => quote! {
            type Held<'a> = ::std::option::Option<&'a mut Self>;
        },
        Semantics::Value => quote! {
            type Held<'a> = Self;
        },
    };

    let namespace_and_class = input.namespace_and_class();
    let class_getter = input.class_getter();

    let match_fns = match semantics {
        Semantics::Reference => input.reference_match_fns(),
        Semantics::Value => input.value_match_fns(),
    };

    let extras = match semantics {
        Semantics::Reference => None,
        Semantics::Value => Some(input.value_extras()),
    };

    TokenStream::from(quote! {
        #header {
            #held
            #namespace_and_class
            #class_getter
            #match_fns
        }
        #extras
    })
}

impl Input {
    fn type_trait(&self) -> TokenStream2 {
        let path = &self.path;
        quote!(#path :: Type)
    }

    fn class_ty(&self) -> TokenStream2 {
        let path = &self.path;
        quote!(#path :: Il2CppClass)
    }

    fn type_ty(&self) -> TokenStream2 {
        let path = &self.path;
        quote!(#path :: Il2CppType)
    }

    fn reflection_type_ty(&self) -> TokenStream2 {
        let path = &self.path;
        quote!(#path :: Il2CppType)
    }

    fn array_ty(&self) -> TokenStream2 {
        let path = &self.path;
        quote!(#path :: Il2CppArray)
    }

    fn impl_header(&self) -> TokenStream2 {
        let type_trait = self.type_trait();
        let ty = &self.ty;
        let generics = &self.rust_generics;
        quote!(unsafe impl<#(#generics: #type_trait),*> #type_trait for #ty<#(#generics),*>)
    }

    fn namespace_and_class(&self) -> TokenStream2 {
        let namespace = &self.namespace;
        let class = &self.class;
        quote! {
            const NAMESPACE: &'static str = #namespace;
            const CLASS_NAME: &'static str = #class;
        }
    }

    fn class_getter(&self) -> Option<TokenStream2> {
        let class_ty = self.class_ty();

        if let Some(getter) = &self.class_getter {
            return Some(quote! {
                fn class() -> &'static #class_ty #getter
            });
        }

        if self.cs_generics.is_empty() {
            return None;
        }

        let reflection_type_ty = self.reflection_type_ty();
        let type_trait = self.type_trait();
        let namespace = &self.namespace;
        let generic_class = format!("{}_{}", self.class, self.cs_generics.len());
        let array_ty = self.array_ty();
        let generics = &self.cs_generics;

        Some(quote! {
            fn class() -> &'static #class_ty {
                static CLASS: ::std::lazy::SyncLazy<&'static Il2CppClass> = ::std::lazy::SyncLazy::new(|| {
                    let reflection_type_class = <#reflection_type_ty as #type_trait>::class();
                    let generic_ty = #class_ty::find(#namespace, #generic_class).unwrap().ty().reflection_object();
                    let generic_ty = unsafe { &mut *(generic_ty as *const _ as *mut _)  };
                    let generics = #array_ty::from_slice(&[#(<#generics as #type_trait>::class()),*]);
                    reflection_type_class.invoke("MakeGenericType", (generic_ty, generics)).unwrap()
                });
                *class
            }
        })
    }

    fn reference_match_fns(&self) -> TokenStream2 {
        let type_trait = self.type_trait();
        let type_ty = self.type_ty();

        quote! {
            fn matches_reference_argument(ty: &#type_ty) -> bool {
                ty.class().is_assignable_from(<Self as #type_trait>::class())
            }
            fn matches_value_argument(_: &#type_ty) -> bool {
                false
            }
            fn matches_reference_parameter(ty: &#type_ty) -> bool {
                <Self as #type_trait>::class().is_assignable_from(ty.class())
            }
            fn matches_value_parameter(_: &#type_ty) -> bool {
                false
            }
        }
    }

    fn value_match_fns(&self) -> TokenStream2 {
        let type_trait = self.type_trait();
        let type_ty = self.type_ty();

        quote! {
            fn matches_value_argument(ty: &#type_ty) -> bool {
                !ty.is_ref() && ty.class().is_assignable_from(<Self as #type_trait>::class())
            }
            fn matches_reference_argument(ty: &#type_ty) -> bool {
                ty.is_ref() && ty.class().is_assignable_from(<Self as #type_trait>::class())
            }
            fn matches_value_parameter(ty: &#type_ty) -> bool {
                !ty.is_ref() && <Self as #type_trait>::class().is_assignable_from(ty.class())
            }
            fn matches_reference_parameter(ty: &#type_ty) -> bool {
                ty.is_ref() && <Self as #type_trait>::class().is_assignable_from(ty.class())
            }
        }
    }

    fn value_extras(&self) -> TokenStream2 {
        let ty = &self.ty;
        let type_trait = self.type_trait();
        let generics = &self.rust_generics;
        let path = &self.path;

        let impl_ = quote!(impl<#(#generics: #type_trait),*>);
        let type_ = quote!(#ty<#(#generics),*>);

        quote! {
            unsafe #impl_ #path::Argument for #type_ {
                type Type = Self;

                fn matches(ty: &#path::Il2CppType) -> bool {
                    <Self as #type_trait>::matches_value_argument(ty)
                }

                fn invokable(&mut self) -> *mut ::std::ffi::c_void {
                    self as *mut Self as *mut ::std::ffi::c_void
                }
            }

            unsafe #impl_ #path::Parameter for #type_ {
                type Actual = Self;

                fn matches(ty: &#path::Il2CppType) -> bool {
                    <Self as #type_trait>::matches_value_parameter(ty)
                }

                fn from_actual(actual: Self::Actual) -> Self {
                    actual
                }
                fn into_actual(self) -> Self::Actual {
                    self
                }
            }

            unsafe #impl_ #path::Returned for #type_ {
                type Type = Self;

                fn matches(ty: &#path::Il2CppType) -> bool {
                    <Self as #type_trait>::matches_returned(ty)
                }

                fn from_object(object: Option<&mut #path::Il2CppObject>) -> Self {
                    unsafe { #path::raw::unbox(#path::WrapRaw::raw(object.unwrap())) }
                }
            }

            unsafe #impl_ #path::Return for #type_ {
                type Actual = Self;

                fn matches(ty: &#path::Il2CppType) -> bool {
                    <Self as #type_trait>::matches_return(ty)
                }

                fn into_actual(self) -> Self::Actual {
                    self
                }
                fn from_actual(actual: Self::Actual) -> Self {
                    actual
                }
            }
        }
    }
}

fn extract_generics(ty: &mut Type) -> Option<AngleBracketedGenericArguments> {
    match ty {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => segments.last_mut().and_then(|ps| match &mut ps.arguments {
            PathArguments::AngleBracketed(args) => {
                let args = args.clone();
                ps.arguments = PathArguments::None;
                Some(args)
            }
            _ => None,
        }),
        Type::Group(TypeGroup { box elem, .. }) | Type::Paren(TypeParen { box elem, .. }) => {
            extract_generics(elem)
        }
        _ => None,
    }
}

fn collect_generics(generics: AngleBracketedGenericArguments) -> Result<Vec<Type>> {
    let mut g = Vec::new();
    for param in generics.args {
        match param {
            GenericArgument::Type(t) => g.push(t),
            _ => return Err(Error::new_spanned(param, "unsupported generic parameter")),
        }
    }
    Ok(g)
}
