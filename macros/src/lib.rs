use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(DataExtractable)]
pub fn derive_data_extractable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = quote! {
        #[serenity::async_trait]
        impl DataExtractable for #name {
            fn init(map: &mut serenity::prelude::TypeMap) {
                map.insert::<Self>(Pointer::new(HashMap::new()));
            }

            fn retrieve(map: &std::sync::Arc<serenity::prelude::TypeMap>) -> Option<Self> {
                map.get::<Self>().cloned().map(Self)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(DefaultExtract)]
pub fn derive_default_extract(input: TokenStream) -> TokenStream {
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
