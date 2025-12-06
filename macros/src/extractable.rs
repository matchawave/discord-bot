use crate::misc::parse_time;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Fields, Lit, PathArguments,
    Token, Type,
};

use crate::misc::CacheAttribute;

pub fn data_extractable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let expanded = quote! {
        #[serenity::async_trait]
        impl Extractable for #name {
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

pub fn cache_extractable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    if let Err(e) = validate_tuple_struct(&ast) {
        panic!("{}", e);
    }

    let (max_capacity, time_to_live, time_to_idle) = match get_options(&ast) {
        Ok(opts) => opts,
        Err(e) => panic!("{}", e),
    };

    let expanded = quote! {
        #[serenity::async_trait]
        impl Default for #name {
            fn default() -> Self {
                #name(
                    moka::future::Cache::builder()
                        #max_capacity
                        #time_to_live
                        #time_to_idle
                        .build()
                )
            }
        }
    };

    TokenStream::from(expanded)
}

fn validate_tuple_struct(ast: &DeriveInput) -> Result<(), String> {
    let Data::Struct(data_struct) = &ast.data else {
        return Err("only defined for structs".into());
    };

    let Fields::Unnamed(fields) = &data_struct.fields else {
        return Err("only defined for tuple structs".into());
    };

    if fields.unnamed.len() != 1 {
        return Err("requires exactly one field".into());
    }

    let Some(field) = fields.unnamed.first() else {
        return Err("requires at least one field".into());
    };

    let Type::Path(type_path) = &field.ty else {
        return Err("field must be a type path".into());
    };

    let Some(segment) = type_path.path.segments.last() else {
        return Err("field must be a type path with segments".into());
    };

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Err("requires a key and value type, e.g., Struct(Hash<Key, Value>)".into());
    };

    if args.args.len() != 2 {
        return Err("requires a key and value type, e.g., Struct(Hash<Key, Value>)".into());
    }

    match (args.args.first(), args.args.last()) {
        (Some(_), Some(_)) => Ok(()),
        _ => Err("requires a key and value type, e.g., Struct(Hash<Key, Value>)".into()),
    }
}

fn get_options(ast: &DeriveInput) -> Result<(TokenStream2, TokenStream2, TokenStream2), String> {
    let mut max_capacity = quote! {};
    let mut time_to_live = quote! {};
    let mut time_to_idle = quote! {};

    if let Some(attr) = ast.attrs.iter().find(|a| a.path().is_ident("cache")) {
        let parsed_attrs = attr
            .parse_args_with(Punctuated::<CacheAttribute, Token![,]>::parse_terminated)
            .expect("Failed to parse cache attributes");

        for attr in parsed_attrs {
            let key = attr.key().to_string();
            let value = attr.value();
            match key.as_str() {
                "capacity" => {
                    let value = parse_capacity(value)?;
                    max_capacity = quote! {
                        .max_capacity(#value)
                    }
                }
                "live" => {
                    let value = parse_duration(value)?;
                    time_to_live = quote! {
                        .time_to_live(std::time::Duration::from_secs(#value))
                    };
                }
                "idle" => {
                    let value = parse_duration(value)?;
                    time_to_idle = quote! {
                        .time_to_idle(std::time::Duration::from_secs(#value))
                    };
                }
                _ => Err(format!("Unknown cache attribute key: {}", key))?,
            }
        }
    }
    Ok((max_capacity, time_to_live, time_to_idle))
}

fn parse_capacity(input: &Lit) -> Result<u64, String> {
    if let Lit::Int(lit_int) = input {
        lit_int
            .base10_parse::<u64>()
            .map_err(|_| format!("Invalid max_capacity value: {:?}", input))
    } else {
        Err(format!(
            "Expected integer literal for capacity, found: {:?}",
            input
        ))
    }
}

fn parse_duration(input: &Lit) -> Result<u64, String> {
    match input {
        Lit::Int(l) => {
            let value = l
                .base10_parse::<u64>()
                .map_err(|_| format!("Invalid duration integer value: {:?}", input))?;
            Ok(value)
        }
        Lit::Str(lit_str) => {
            let value = lit_str.value();
            parse_time(&value)
        }
        _ => Err(format!(
            "Expected string or integer literal for duration, found: {:?}",
            input
        )),
    }
}
