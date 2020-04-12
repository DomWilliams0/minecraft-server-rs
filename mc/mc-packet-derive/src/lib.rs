use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Ident, Meta, MetaNameValue};

use syn::{Data, DeriveInput, Type};

fn extract_packet_id(item: &DeriveInput) -> i32 {
    let span = item.span();
    let attribute = item
        .attrs
        .iter()
        .find(|a| a.path.is_ident("packet_id"))
        .unwrap_or_else(|| abort!(span, "expected packet_id attribute"));

    let meta = attribute
        .parse_meta()
        .unwrap_or_else(|_| abort!(span, "bad syntax"));
    match meta {
        Meta::NameValue(MetaNameValue {
            path,
            lit: syn::Lit::Int(int),
            ..
        }) if path.is_ident("packet_id") => int.base10_parse().expect("bad integer"),
        _ => abort!(span, "bad packet id"),
    }
}

fn extract_fields(item: &DeriveInput) -> (Vec<&Ident>, impl Iterator<Item = &Ident>) {
    let r#struct = match &item.data {
        Data::Struct(r#struct) => r#struct,
        _ => abort_call_site!("Packet must be a struct"),
    };

    let field_names = r#struct
        .fields
        .iter()
        .map(|f| f.ident.as_ref().expect("expected field identifier"))
        .collect();

    let field_types = r#struct.fields.iter().map(|f| match &f.ty {
        Type::Path(p) => p.path.get_ident().unwrap(),
        _ => abort!(f.span(), "field should be a field type"),
    });

    (field_names, field_types)
}

fn impl_display(name: &Ident, field_names: &[&Ident]) -> impl ToTokens {
    let out = quote! {
        impl Display for #name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}(", stringify!(#name))?;

                let mut sep = "";
                #( write!(f, "{}{}={}",
                    sep,
                    stringify!(#field_names),
                    DisplayableField(&self.#field_names .value())
                )?; sep = ", "; )*

                write!(f, ")")
            }
        }
    };
    out
}

#[proc_macro_derive(ServerBoundPacket, attributes(packet_id))]
#[proc_macro_error]
pub fn server_packet(input: TokenStream) -> TokenStream {
    let item: DeriveInput = syn::parse(input.clone()).expect("failed to parse input");

    let packet_id = extract_packet_id(&item);
    let (field_names, field_types) = extract_fields(&item);
    let name = &item.ident;
    // let test_mod = format_ident!("test_{}", name);
    let display = impl_display(name, &field_names);
    let result = quote! {
        impl Packet for #name {
            // fn id() -> PacketId { Self::ID }
        }

        impl #name {
            pub const ID: PacketId = #packet_id;
        }

        #[async_trait]
        impl ServerBound for #name {
            async fn read_packet(body: PacketBody) -> McResult<Self> {
                if body.id != Self::ID {
                    return Err(McError::UnexpectedPacket {
                        expected: Self::ID,
                        actual: body.id,
                    });
                }

                let mut cursor = Cursor::new(body.body);

                #( let #field_names = <#field_types>::read_field(&mut cursor).await?;)*

                let packet = Self {
                    #( #field_names ),*
                };

                let full_len = cursor.get_ref().len();
                let read_len = cursor.position() as usize;

                trace!("read packet id {:#x} of {} bytes: {}", body.id, read_len, packet);

                if read_len != full_len {
                    Err(McError::FullPacketNotRead {
                        length: full_len,
                        read: read_len,
                    })
                } else {
                    Ok(packet)
                }
            }
        }

        #display
    };

    result.into()
}

#[proc_macro_derive(ClientBoundPacket, attributes(packet_id))]
#[proc_macro_error]
pub fn client_packet(input: TokenStream) -> TokenStream {
    let item: DeriveInput = syn::parse(input.clone()).expect("failed to parse input");

    let packet_id = extract_packet_id(&item);
    let (field_names, _field_types) = extract_fields(&item);

    let name = &item.ident;
    // let test_mod = format_ident!("test_{}", name);
    let display = impl_display(name, &field_names);
    let result = quote! {
        impl Packet for #name {
            // fn id() -> PacketId { Self::ID }
        }

        impl #name {
            pub const ID: PacketId = #packet_id;
        }

        #[async_trait]
        impl ClientBound for #name {
            async fn write_packet(&self, w: &mut Cursor<&mut [u8]>) -> McResult<()> {
                let packet_id = VarIntField::new(Self::ID);
                let len = VarIntField::new(self.length() as i32);

                // TODO resize writer to exact size - limit to Cursor or make own trait for it?

                trace!("sending packet id {:#x} of {} bytes: {}", #name::ID, len.value(), self);

                len.write_field(w).await?;
                packet_id.write_field(w).await?;

                #( self.#field_names.write_field(w).await?; )*

                Ok(())

            }

            fn length(&self) -> usize {
                let packet_id = VarIntField::new(Self::ID);
                let mut len = 0;
                len += packet_id.size();

                #( len += self.#field_names.size(); )*

                len
            }
        }

        #display
        // #[cfg(test)]
        // mod #test_mod {
        //
        // }
    };

    result.into()
}
