use proc_macro::TokenStream;

mod bundle;

#[proc_macro_derive(Bundle)]
pub fn bundle_derive(item: TokenStream) -> TokenStream {
    return bundle::bundle_derive(item.into()).into();
}
