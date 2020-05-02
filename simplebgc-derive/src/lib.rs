extern crate proc_macro;
extern crate proc_macro_error;
extern crate quote;

use crate::field::{get_info_for_field, FieldInfo, FieldKind};
use crate::primitive::PrimitiveKind;
use enumflags2::_internal::core::convert::TryFrom;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::*;
use quote::{format_ident, quote, quote_spanned};
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::*;

mod field;
mod primitive;

#[proc_macro_error]
#[proc_macro_derive(BgcPayload, attributes(kind, size, name, repr))]
pub fn payload_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_ident = input.ident;

    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let parse_info: Vec<ParseStmtInfo> = fields
                    .named
                    .iter()
                    .enumerate()
                    .filter_map(|(i, field)| get_parser_for_field(i, field))
                    .collect();
                let var_idents: Vec<&Ident> =
                    parse_info.iter().map(|stmt| &stmt.variable_ident).collect();
                let field_idents: Vec<&Ident> = parse_info
                    .iter()
                    .map(|stmt| stmt.field_ident.as_ref().unwrap())
                    .collect();
                let parse_stmts: Vec<&TokenStream2> =
                    parse_info.iter().map(|stmt| &stmt.stmt).collect();

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
                let parse_info: Vec<ParseStmtInfo> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter_map(|(i, field)| get_parser_for_field(i, field))
                    .collect();
                let var_idents: Vec<&Ident> =
                    parse_info.iter().map(|stmt| &stmt.variable_ident).collect();
                let parse_stmts: Vec<&TokenStream2> =
                    parse_info.iter().map(|stmt| &stmt.stmt).collect();

                quote! {

                    impl Payload for #type_ident {
                        fn from_bytes(mut b: Bytes) -> Result<Self, PayloadParseError>
                        where
                            Self: Sized,
                        {
                            #(#parse_stmts)*

                            Ok(#type_ident (
                                #(#var_idents),*
                            ))
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

/// Returns true if ty is a primitive integer type
/// usize and isize don't count, but u128 and i128 do
fn get_primitive_int_kind(ty: &TypePath) -> Option<&Ident> {
    match ty.path.get_ident() {
        Some(ident) => {
            if ident == "u8"
                || ident == "i8"
                || ident == "u16"
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
    }
}

const ERR_RAW_PRIMITIVE: &str =
    "field must be primitive type, tuple of primitive types, or array of u8 for raw values";

/// idx: the counter for auto-generated variable names
/// info: information about the current field
fn get_parser_for_field(info: &FieldInfo) -> Option<TokenStream2> {
    let var = &info.variable;
    let span = &info.span;

    match &info.kind {
        FieldKind::Payload { ty, size } => Some(quote_spanned! {span=>
            let #var: #ty = Payload::from_bytes(b.split_to(#size))?;
        }),
        FieldKind::Flags { repr } => {
            let get_value = match repr {
                PrimitiveKind::U8 | PrimitiveKind::I8 => format_ident!("get_{}", repr),
                _ => format_ident!("get_{}_le", repr),
            };

            Some(quote_spanned! {span=>
                let #var = BitFlags::from_bits(b.#get_value())
                    .or(Err(PayloadParseError::InvalidFlags { name: #spec_name.into() }))?;
            })
        }
        FieldKind::Enum { repr } => {
            let get_value = match repr {
                PrimitiveKind::U8 | PrimitiveKind::I8 => format_ident!("get_{}", repr),
                _ => format_ident!("get_{}_le", repr),
            };

            Some(quote_spanned! {span=>
                let #var = FromPrimitive::#from_value(b.#get_value())
                    .ok_or(PayloadParseError::InvalidEnum { name: #spec_name.into() })?;
            })
        }
        FieldKind::Raw { ty } => {
            // if it is a primitive, this is simple
            if let Ok(repr) = PrimitiveKind::try_from(ty) {
                let get_value = match repr {
                    PrimitiveKind::U8 | PrimitiveKind::I8 => format_ident!("get_{}", repr),
                    _ => format_ident!("get_{}_le", repr),
                };

                return Some(quote_spanned! {span=>
                    let #var = b.#get_value();
                });
            }

            match &field.ty {
                Type::Array(ty) => {
                    if Ok(PrimitiveKind::U8) == PrimitiveKind::try_from(ty.elem.as_ref()) {
                        let len = &ty.len;

                        Some(quote_spanned! {span=>
                            let mut #var = [0u8; #len];
                            b.copy_to_slice(&mut #var[..]);
                        })
                    } else {
                        emit_error!(field.ty, ERR_RAW_PRIMITIVE);
                        None
                    }
                }
                Type::Tuple(ty) => {
                    let mut item_parse_stmts =
                        ty.elems.iter().enumerate().map(|(elem_idx, elem_ty)| {
                            get_parser_for_field(&FieldInfo {
                                name: format!("{}[{}]", &info.name, elem_idx),
                                kind: FieldKind::Raw {
                                    ty: (*elem_ty).clone(),
                                },
                                span: info.span.clone(),
                                variable: format_ident!("{}_{}", &info.variable, elem_idx),
                                ident: None,
                            });
                        });

                    let mut item_vars = ty.elems.iter().enumerate().map(|(elem_idx, elem_ty)| {
                        format_ident!("{}_{}", &info.variable, elem_idx)
                    });

                    Some(quote_spanned! {span=>
                        let #variable_ident = {
                            #(#item_parse_stmts)*
                            (#(item_vars),*)
                        };
                    })
                }
                _ => {
                    emit_error!(field.ty, ERR_RAW_PRIMITIVE);
                    return None;
                }
            }
        }
    }
}
