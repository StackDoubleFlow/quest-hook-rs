use std::ops::Range;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::Result;

pub fn expand(range: Range<usize>) -> Result<TokenStream> {
    let mut ts = TokenStream::new();
    for n in range {
        let generics = (1..=n).map(|n| format_ident!("T{}", n));
        let ptr_write = generics
            .clone()
            .enumerate()
            .map(|(n, g)| quote! {
                (((arr as *mut _ as isize) + (raw::kIl2CppSizeOfArray as isize)) as *mut &Il2CppReflectionType)
                .add(#n)
                .write_unaligned(#g::class().ty().reflection_object());
            });

        let generics_impl = generics.clone();
        let generics_ty = generics.clone();
        let generics_where = generics.clone();

        let impl_ts = quote! {
            impl<#(#generics_impl),*> Generics for (#(#generics_ty,)*)
            where
                #(#generics_where: Type),*
            {
                const COUNT: usize = #n;

                fn type_array() -> &'static mut raw::Il2CppArray {
                    let arr = unsafe { raw::array_new(Il2CppReflectionType::class().raw(), #n) }.unwrap();
                    unsafe {
                        #(#ptr_write)*
                    }
                    arr
                }
            }
        };
        ts.extend(TokenStream::from(impl_ts));
    }

    Ok(ts)
}
