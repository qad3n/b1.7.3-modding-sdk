use proc_macro::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemFn, ItemImpl, parse_macro_input};

// marks the register function, the loader calls it once in the register phase
#[proc_macro_attribute]
pub fn mod_main(_args: TokenStream, input: TokenStream) -> TokenStream
{
    let function: ItemFn = parse_macro_input!(input as ItemFn);
    let name = &function.sig.ident;
    quote! {
        #function

        #[unsafe(no_mangle)]
        pub fn __b173_main(ctx: &mut ::b173_api::RegisterCtx) -> ::b173_api::Result<()> { #name(ctx) }
    }
    .into()
}

fn behaviorImpl(input: TokenStream) -> TokenStream
{
    let mut item: ItemImpl = parse_macro_input!(input as ItemImpl);
    let names: Vec<String> = item
        .items
        .iter()
        .filter_map(|member| match member
        {
            ImplItem::Fn(function) => Some(function.sig.ident.to_string()),
            _ => None,
        })
        .collect();
    item.items.push(syn::parse_quote! {
        fn overridden(&self) -> &'static [&'static str] { &[#(#names),*] }
    });
    quote!(#item).into()
}

// records which trait methods the impl overrides, only those are routed from the host into wasm
#[proc_macro_attribute]
pub fn block_behavior(_args: TokenStream, input: TokenStream) -> TokenStream { behaviorImpl(input) }

#[proc_macro_attribute]
pub fn item_behavior(_args: TokenStream, input: TokenStream) -> TokenStream { behaviorImpl(input) }

#[proc_macro_attribute]
pub fn entity_behavior(_args: TokenStream, input: TokenStream) -> TokenStream { behaviorImpl(input) }

#[proc_macro_attribute]
pub fn tile_behavior(_args: TokenStream, input: TokenStream) -> TokenStream { behaviorImpl(input) }
