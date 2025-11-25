use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod extractable;
pub(crate) mod misc;

#[proc_macro_derive(DataExtract)]
pub fn derive_data_extract(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = quote! {
        #[serenity::async_trait]
        impl<T> Extractor<T> for #name {
            async fn extract(
                ctx: &serenity::prelude::Context,
                _ev: &T,
                _p: &utils::Pointer<utils::Parser>,
            ) -> Option<Self> {
                crate::data::Data::get(&ctx.data, ctx.shard_id)
                    .await
                    .as_ref()
                    .and_then(Self::retrieve)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(CacheExtract)]
pub fn derive_cache_extract(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = quote! {
        #[serenity::async_trait]
        impl<T> Extractor<T> for #name {
            async fn extract(
                ctx: &serenity::prelude::Context,
                _ev: &T,
                _p: &utils::Pointer<utils::Parser>,
            ) -> Option<Self> {
                crate::cache::Cache::get(&ctx.data, ctx.shard_id)
                    .await
                    .as_ref()
                    .and_then(Self::retrieve)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(DataExtractable)]
pub fn derive_data_extractable(input: TokenStream) -> TokenStream {
    extractable::data_extractable(input)
}

#[proc_macro_derive(CacheExtractable, attributes(cache))]
pub fn derive_cache_extractable(input: TokenStream) -> TokenStream {
    extractable::cache_extractable(input)
}
