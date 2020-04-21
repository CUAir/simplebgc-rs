extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::*;

#[proc_macro_derive(Payload, attributes(flags, enumeration, payload))]
pub fn payload_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut parse_statements = vec![];

    match input.data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields) => {
                    for field in fields.named.iter() {
                        // get name of field in Rust
                        let ident = field.ident.as_ref().unwrap();
                        for attr in field.attrs.iter() {
                            if attr.path.is_ident("flags") {
                                // get name of field in spec
                                let spec_name = attr.parse_args::<Lit>().expect(
                                    "Must include name of this field in the SimpleBGC spec.",
                                );

                                let repr = match &field.ty {
                                    Type::Path(path) => path.path.get_ident(),
                                    _ => panic!("Flag types should be primitives"),
                                };

                                parse_statements.push(quote! {
                                    let #ident = read_enum!(b, #spec_name);
                                });
                            }
                        }
                    }
                }
                Fields::Unnamed(_) => {}
                Fields::Unit => {}
            }
        }
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }

    (quote! {
        let x = "hi";
    }).into()
}
