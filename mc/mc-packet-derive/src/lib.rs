use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::quote;
use syn::export::ToTokens;
use syn::spanned::Spanned;

use proc_macro_error::proc_macro2::TokenTree;
use syn::{Data, DeriveInput, Type};

#[proc_macro_derive(ServerBoundPacket, attributes(packet_id))]
#[proc_macro_error]
pub fn packet(input: TokenStream) -> TokenStream {
    let item: DeriveInput = syn::parse(input.clone()).expect("failed to parse input");

    let packet_id = {
        let span = item.span();
        let attribute = item
            .attrs
            .into_iter()
            .filter(|a| {
                let ident = a.path.get_ident().map(|i| i.to_string());
                matches!(ident.as_deref(), Some("packet_id"))
            })
            .next()
            .unwrap_or_else(|| abort!(span, "expected packet_id attribute"));

        let literal = attribute
            .tokens
            .into_iter()
            .filter_map(|t| match t {
                TokenTree::Literal(lit) => Some(lit),
                _ => None,
            })
            .next()
            .unwrap_or_else(|| abort!(span, "expected literal for packet id"));

        let integer_literal: syn::LitInt = syn::parse2(literal.into_token_stream())
            .expect("expected integer literal for packet id");
        let integer: i32 = integer_literal.base10_parse().expect("bad integer");
        integer
    };

    let r#struct = match item.data {
        Data::Struct(r#struct) => r#struct,
        _ => abort_call_site!("Packet must be a struct"),
    };

    let field_names: Vec<&proc_macro2::Ident> = r#struct
        .fields
        .iter()
        .map(|f| f.ident.as_ref().expect("expected field identifier"))
        .collect();

    let field_types = r#struct.fields.iter().map(|f| match &f.ty {
        Type::Path(p) => p.path.get_ident().unwrap(),
        _ => abort!(f.span(), "field should be a field type"),
    });

    let name = item.ident;
    let result = quote! {
        impl Packet for #name {
            fn id() -> PacketId { #packet_id }
        }

        impl ServerBound for #name {

            fn read(body: PacketBody) -> McResult<Self> {

            if body.id != Self::id() {
                return Err(McError::UnexpectedPacket {
                    expected: Self::id(),
                    actual: body.id,
                });
            }

            let mut cursor = Cursor::new(body.body);

            #( let #field_names = <#field_types>::read(&mut cursor)?;)*

            Ok(Self {
                #( #field_names ),*
            })

            }
        }
    };

    result.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
