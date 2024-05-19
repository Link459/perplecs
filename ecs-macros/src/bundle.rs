use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput};

pub fn bundle_derive(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let output = match item.data {
        Data::Struct(ref x) => impl_struct(&x, &item),
        Data::Enum(_) => impl_enum(&item),
        Data::Union(_) => impl_union(&item),
    };

    return output;
}

fn impl_struct(data: &DataStruct, item: &DeriveInput) -> TokenStream {
    let types = data.fields.iter().map(|x| &x.ty).collect::<Vec<_>>();
    let names = data.fields.iter().map(|x| &x.ident).collect::<Vec<_>>();

    let generics = &item.generics;
    let where_clause = &item.generics.where_clause;
    let name = &item.ident;

    let output = quote! {
        impl #generics perplecs::prelude::Bundle<'_> for #name #generics #where_clause {
            type Target = ();
            type TargetMut = ();
            fn type_info() -> Box<[perplecs::archetype::TypeInfo]> {
                Box::new([
                    #(perplecs::archetype::TypeInfo::new::<#types>()),*
                ])
            }

            fn type_ids() -> Box<[std::any::TypeId]> {
                Box::new([
                    #(std::any::TypeId::of::<#types>()),*
                ])
            }

            unsafe fn as_ptrs(&mut self) -> Box<[*mut u8]> {
                Box::new([
                    #(perplecs::bundle::into_ptr::<#types>(&mut self.#names)),*
                ])
            }

            unsafe fn from_ptr(data: &[*mut u8]) -> Self::Target {
                ()
            }

            unsafe fn from_ptr_mut(data: &[*mut u8]) -> Self::TargetMut {
                ()
            }
        }
    };

    return output.into();
}

fn impl_enum(item: &DeriveInput) -> TokenStream {
    return TokenStream::from(
        quote_spanned! { item.span() => compile_error!("enums can't be used as bundles") },
    );
}

fn impl_union(item: &DeriveInput) -> TokenStream {
    return TokenStream::from(
        quote_spanned! { item.span() => compile_error!("unions can't be used as bundles") },
    );
}
