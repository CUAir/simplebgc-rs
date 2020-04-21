extern crate proc_macro;
extern crate proc_macro_error;
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::*;
use quote::{format_ident, quote};
use syn::group::Group;
use syn::punctuated::Punctuated;
use syn::*;

enum FieldKind {
    Flags,
    Enum,
    Payload,
    Raw,
}

#[proc_macro_error]
#[proc_macro_derive(BgcPayload, attributes(bgc_flags, bgc_enum, bgc_payload, bgc_raw, bgc_size))]
pub fn payload_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_ident = input.ident;

    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let parse_info: Vec<(Ident, Option<Ident>, TokenStream2)> = fields
                    .named
                    .iter()
                    .enumerate()
                    .filter_map(|(i, field)| get_stmt_for_field(i, field))
                    .collect();
                let var_idents: Vec<&Ident> = parse_info
                    .iter()
                    .map(|(var_ident, _, _)| var_ident)
                    .collect();
                let field_idents: Vec<&Ident> = parse_info
                    .iter()
                    .map(|(_, field_ident, _)| field_ident.as_ref().unwrap())
                    .collect();
                let parse_stmts: Vec<&TokenStream2> =
                    parse_info.iter().map(|(_, _, stmt)| stmt).collect();

                quote! {
                    impl Payload for #type_ident {
                        fn from_bytes(mut b: Bytes) -> Result<Self, PayloadParseError>
                        where
                            Self: Sized,
                        {
                            #(#parse_stmts)*

                            Ok(#type_ident {
                                #(#field_idents: #var_idents),*
                            })
                        }

                        fn to_bytes(&self) -> Bytes
                        where
                            Self: Sized,
                        {
                            unimplemented!()
                        }
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let parse_info: Vec<(Ident, Option<Ident>, TokenStream2)> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter_map(|(i, field)| get_stmt_for_field(i, field))
                    .collect();
                let var_idents: Vec<&Ident> = parse_info
                    .iter()
                    .map(|(var_ident, _, _)| var_ident)
                    .collect();
                let parse_stmts: Vec<&TokenStream2> =
                    parse_info.iter().map(|(_, _, stmt)| stmt).collect();

                quote! {

                    impl Payload for #type_ident {
                        fn from_bytes(mut b: Bytes) -> Result<Self, PayloadParseError>
                        where
                            Self: Sized,
                        {
                            #(#parse_stmts)*

                            Ok(#type_ident {
                                #(#var_idents),*
                            })
                        }

                        fn to_bytes(&self) -> Bytes
                        where
                            Self: Sized,
                        {
                            unimplemented!()
                        }
                    }
                }
            }
            Fields::Unit => abort!(data.struct_token, "this does not work on unit structs"),
        },
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
    .into()
}

fn get_primitive_type(ty: &Type) -> Option<&Ident> {
    match ty {
        Type::Path(path) => match path.path.get_ident() {
            Some(ident) => {
                if ident == "u8" || ident == "i8" {
                    Some(ident)
                } else if ident == "u16"
                    || ident == "i16"
                    || ident == "u32"
                    || ident == "i32"
                    || ident == "u64"
                    || ident == "i64"
                    || ident == "u128"
                    || ident == "i128"
                {
                    Some(ident)
                } else {
                    None
                }
            }
            None => None,
        },
        _ => None,
    }
}

/// Returns (variable identifier, field identifier, parse statement)
fn get_stmt_for_field(idx: usize, field: &Field) -> Option<(Ident, Option<Ident>, TokenStream2)> {
    // get name of field in Rust
    // or generate one if it doesn't have a name
    let field_ident = field.ident.clone();
    let variable_ident = field_ident.clone().unwrap_or(format_ident!("field{}", idx));
    let mut field_kind = None;
    let mut spec_name = None;
    let mut spec_size = None;

    for attr in field.attrs.iter() {
        if attr.path.is_ident("bgc_size") {
            if spec_size.is_some() {
                emit_error!(attr, "multiple size attributes on field not allowed");
                return None;
            }

            spec_size = Some(match attr.parse_args::<LitInt>() {
                Ok(s) => match s.base10_parse::<usize>() {
                    Ok(s) => s,
                    Err(_) => {
                        emit_error!(attr, "invalid size attribute");
                        return None;
                    }
                }
                Err(_) => {
                    emit_error!(attr, "invalid size attribute");
                    return None;
                }
            });
        }

        let new_field_kind = if attr.path.is_ident("bgc_flags") {
            Some(FieldKind::Flags)
        } else if attr.path.is_ident("bgc_enum") {
            Some(FieldKind::Enum)
        } else if attr.path.is_ident("bgc_payload") {
            Some(FieldKind::Payload)
        } else if attr.path.is_ident("bgc_raw") {
            Some(FieldKind::Raw)
        } else {
            None
        };

        if new_field_kind.is_some() {
            if field_kind.is_some() {
                emit_error!(attr, "multiple annotations on field not allowed");
                return None;
            } else {
                field_kind = new_field_kind;

                // get name of field in spec
                spec_name = match attr.parse_args::<LitStr>() {
                    Ok(s) => Some(s.value()),
                    Err(_) => {
                        emit_error!(
                            attr,
                            "must include name of this field in the SimpleBGC spec"
                        );
                        return None;
                    }
                };
            }
        }
    }

    // if kind is still not assigned here, then we didn't find
    // anything
    let field_kind = match field_kind {
        Some(k) => k,
        None => {
            emit_error!(
                field,
                "field must be annotated as flag, enumeration, payload, or raw"
            );
            return None;
        }
    };

    // if kind is assigned, then spec_name will be as well
    let spec_name = spec_name.unwrap();

    match field_kind {
        FieldKind::Payload => {
            let field_type = &field.ty;
            let spec_size = match spec_size {
                Some(s) => s,
                None => {
                    emit_error!(field, "size must be specified for payload fields");
                    return None;
                }
            };

            Some((
                variable_ident.clone(),
                field_ident,
                quote! {
                    let #variable_ident: #field_type = Payload::from_bytes(b.split_to(#spec_size))
                },
            ))
        },
        FieldKind::Flags | FieldKind::Enum | FieldKind::Raw => {
            let repr = match get_primitive_type(&field.ty) {
                Some(r) => r.clone(),
                None => {
                    emit_error!(field.ty, "field must be primitive type");
                    return None;
                }
            };

            let repr_endian = if repr == "u8" || repr == "i8" {
                repr.clone()
            } else {
                format_ident!("{}_le", repr.clone())
            };

            let get_value = format_ident!("get_{}", repr_endian);

            match field_kind {
                FieldKind::Flags => Some((
                    variable_ident.clone(),
                    field_ident,
                    quote! {
                        let #variable_ident = BitFlags::from_bits(b.#get_value())
                            .or(Err(PayloadParseError::InvalidFlags { name: #spec_name.into() }))?;
                    },
                )),
                FieldKind::Enum => {
                    let from_value = format_ident!("from_{}", repr);
                    Some((
                        variable_ident.clone(),
                        field_ident,
                        quote! {
                            let #variable_ident = FromPrimitive::#from_value(b.#get_value())
                                .ok_or(PayloadParseError::InvalidEnum { name: #spec_name })?;
                        },
                    ))
                }
                FieldKind::Raw => Some((
                    variable_ident.clone(),
                    field_ident,
                    quote! {
                        let #variable_ident = b.#get_value();
                    },
                )),
                _ => unreachable!(),
            }
        }
    }
}
