extern crate proc_macro;
extern crate proc_macro_error;
extern crate quote;

use crate::field::*;
use crate::primitive::*;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::*;
use quote::{format_ident, quote, quote_spanned};
use std::convert::TryFrom;
use syn::*;
use syn::spanned::Spanned;

mod field;
mod primitive;

#[proc_macro_error]
#[proc_macro_derive(BgcPayload, attributes(kind, size, name, format))]
pub fn payload_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ty = input.ident;

    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let fields_info = fields
                    .named
                    .iter()
                    .enumerate()
                    .filter_map(|(i, field)| get_info_for_field(i, field))
                    .collect::<Vec<_>>();

                let parse_stmts = fields_info
                    .iter()
                    .filter_map(|info| get_parser_for_field(&info))
                    .collect::<Vec<_>>();

                let ser_stmts = fields_info
                    .iter()
                    .filter_map(|info| get_serializer_for_field(&info))
                    .collect::<Vec<_>>();

                let vars = fields_info
                    .iter()
                    .map(|info| &info.variable)
                    .collect::<Vec<_>>();

                let fields = fields_info
                    .iter()
                    .map(|info| (&info.ident).as_ref().unwrap())
                    .collect::<Vec<_>>();

                quote! {
                    impl Payload for #ty {
                        fn from_bytes(mut _b: Bytes) -> Result<Self, PayloadParseError>
                        where
                            Self: Sized,
                        {
                            #(#parse_stmts)*

                            Ok(#ty {
                                #(#fields: #vars),*
                            })
                        }

                        fn to_bytes(&self) -> Bytes
                        where
                            Self: Sized,
                        {
                            let mut _b = BytesMut::new();
                            let &#ty { #(#vars),* } = self;

                            #(#ser_stmts)*

                            _b.freeze()
                        }
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let fields_info: Vec<_> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter_map(|(i, field)| get_info_for_field(i, field))
                    .collect();

                let parse_stmts: Vec<_> = fields_info
                    .iter()
                    .filter_map(|info| get_parser_for_field(&info))
                    .collect();

                let ser_stmts = fields_info
                    .iter()
                    .filter_map(|info| get_serializer_for_field(&info))
                    .collect::<Vec<_>>();

                let vars = fields_info
                    .iter()
                    .map(|info| &info.variable)
                    .collect::<Vec<_>>();

                quote! {
                    impl Payload for #ty {
                        fn from_bytes(mut _b: Bytes) -> Result<Self, PayloadParseError>
                        where
                            Self: Sized,
                        {
                            #(#parse_stmts)*

                            Ok(#ty (
                                #(#vars),*
                            ))
                        }

                        fn to_bytes(&self) -> Bytes
                        where
                            Self: Sized,
                        {
                            let mut _b = BytesMut::new();
                            let &#ty ( #(#vars),* ) = self;

                            #(#ser_stmts)*

                            _b.freeze()
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

const ERR_RAW_PRIMITIVE: &str =
    "field must be primitive type, tuple of primitive types, or array of u8 for raw values";

fn get_parser_for_field(info: &FieldInfo) -> Option<TokenStream2> {
    let var = &info.variable;
    let span = info.span;
    let name = &info.name;

    match &info.kind {
        FieldKind::Payload { ty, size } => Some(quote_spanned! {span=>
            let #var: #ty = Payload::from_bytes(_b.split_to(#size))?;
        }),
        FieldKind::Flags { repr } => {
            let get_value = match repr {
                PrimitiveKind::U8 | PrimitiveKind::I8 => format_ident!("get_{}", repr),
                _ => format_ident!("get_{}_le", repr),
            };

            Some(quote_spanned! {span=>
                let #var = BitFlags::from_bits(_b.#get_value())
                    .or(Err(PayloadParseError::InvalidFlags { name: #name.into() }))?;
            })
        }
        FieldKind::Enum { repr } => {
            let get_value = match repr {
                PrimitiveKind::U8 | PrimitiveKind::I8 => format_ident!("get_{}", repr),
                _ => format_ident!("get_{}_le", repr),
            };

            let from_value = format_ident!("from_{}", repr);

            Some(quote_spanned! {span=>
                let #var = FromPrimitive::#from_value(_b.#get_value())
                    .ok_or(PayloadParseError::InvalidEnum { name: #name.into() })?;
            })
        }
        FieldKind::Raw { ty } => {
            // if it is a primitive, this is simple
            if let Ok(repr) = PrimitiveKind::try_from(ty.clone()) {
                return Some(match repr {
                    PrimitiveKind::Bool => {
                        quote_spanned! {span=>
                            let #var = _b.get_u8() != 0;
                        }
                    }
                    PrimitiveKind::U8 | PrimitiveKind::I8 => {
                        let get_value = format_ident!("get_{}", repr);
                        quote_spanned! {span=>
                            let #var = _b.#get_value();
                        }
                    }
                    _ => {
                        let get_value = format_ident!("get_{}_le", repr);
                        quote_spanned! {span=>
                            let #var = _b.#get_value();
                        }
                    }
                });
            }

            match ty {
                Type::Array(ty) => {
                    if let Ok(PrimitiveKind::U8) = PrimitiveKind::try_from(ty.elem.as_ref().clone())
                    {
                        let len = &ty.len;

                        Some(quote_spanned! {span=>
                            let mut #var = [0u8; #len];
                            _b.copy_to_slice(&mut #var[..]);
                        })
                    } else {
                        emit_error!(ty, ERR_RAW_PRIMITIVE);
                        None
                    }
                }
                Type::Tuple(ty) => {
                    let item_parse_stmts = ty
                        .elems
                        .iter()
                        .enumerate()
                        .filter_map(|(elem_idx, elem_ty)| {
                            // recursion ftw
                            get_parser_for_field(&FieldInfo {
                                name: format!("{}[{}]", &info.name, elem_idx),
                                kind: FieldKind::Raw {
                                    ty: (*elem_ty).clone(),
                                },
                                idx: elem_idx,
                                span: info.span.clone(),
                                variable: format_ident!("{}_{}", &info.variable, elem_idx),
                                ident: None,
                            })
                        })
                        .collect::<Vec<_>>();

                    let item_vars = ty
                        .elems
                        .iter()
                        .enumerate()
                        .map(|(elem_idx, _)| format_ident!("{}_{}", &info.variable, elem_idx))
                        .collect::<Vec<_>>();

                    if item_vars.len() != item_parse_stmts.len() {
                        // some of the parse statement generations failed, abort
                        return None;
                    }

                    Some(quote_spanned! {span=>
                        let #var = {
                            #(#item_parse_stmts)*
                            (#(#item_vars),*)
                        };
                    })
                }
                _ => {
                    emit_error!(ty, ERR_RAW_PRIMITIVE);
                    return None;
                }
            }
        }
    }
}

fn get_serializer_for_field(info: &FieldInfo) -> Option<TokenStream2> {
    let var = &info.variable;
    let span = info.span;

    match &info.kind {
        FieldKind::Payload { ty, size } => Some(quote_spanned! {span=>
            _b.put(Payload::to_bytes(&#var));
        }),
        FieldKind::Flags { repr } => {
            let put_value = match repr {
                PrimitiveKind::U8 | PrimitiveKind::I8 => format_ident!("put_{}", repr),
                _ => format_ident!("put_{}_le", repr),
            };

            Some(quote_spanned! {span=>
                _b.#put_value(#var.bits());
            })
        }
        FieldKind::Enum { repr } => {
            let put_value = match repr {
                PrimitiveKind::U8 | PrimitiveKind::I8 => format_ident!("put_{}", repr),
                _ => format_ident!("put_{}_le", repr),
            };

            let to_value = format_ident!("to_{}", repr);

            Some(quote_spanned! {span=>
                _b.#put_value(ToPrimitive::#to_value(&#var).unwrap());
            })
        }
        FieldKind::Raw { ty } => {
            // if it is a primitive, this is simple
            if let Ok(repr) = PrimitiveKind::try_from(ty.clone()) {
                return Some(match repr {
                    PrimitiveKind::Bool => {
                        quote_spanned! {span=>
                            _b.put_u8(#var as u8);
                        }
                    }
                    PrimitiveKind::U8 | PrimitiveKind::I8 => {
                        let put_value = format_ident!("put_{}", repr);
                        quote_spanned! {span=>
                            _b.#put_value(#var);
                        }
                    }
                    _ => {
                        let put_value = format_ident!("put_{}_le", repr);
                        let repr = format_ident!("{}", repr);
                        quote_spanned! {span=>
                            _b.#put_value(#var as #repr);
                        }
                    }
                });
            }
            match ty {
                Type::Array(ty) => {
                    if let Ok(PrimitiveKind::U8) = PrimitiveKind::try_from(ty.elem.as_ref().clone())
                    {
                        Some(quote_spanned! {span=>
                            _b.copy_from_slice(&#var[..]);
                        })
                    } else {
                        None
                    }
                }
                Type::Tuple(ty) => {
                    let item_ser_stmts = ty
                        .elems
                        .iter()
                        .enumerate()
                        .filter_map(|(elem_idx, elem_ty)| {
                            // recursion ftw
                            get_serializer_for_field(&FieldInfo {
                                name: format!("{}[{}]", &info.name, elem_idx),
                                kind: FieldKind::Raw {
                                    ty: (*elem_ty).clone(),
                                },
                                idx: elem_idx,
                                span: elem_ty.span(),
                                variable: format_ident!("{}_{}", &info.variable, elem_idx),
                                ident: None,
                            })
                        })
                        .collect::<Vec<_>>();

                    let item_vars = ty
                        .elems
                        .iter()
                        .enumerate()
                        .map(|(elem_idx, _)| format_ident!("{}_{}", &info.variable, elem_idx))
                        .collect::<Vec<_>>();

                    if item_vars.len() != item_ser_stmts.len() {
                        // some of the parse statement generations failed, abort
                        return None;
                    }

                    Some(quote_spanned! {span=>
                        {
                            let (#(#item_vars),*) = #var;
                            #(#item_ser_stmts)*
                        };
                    })
                }
                _ => {
                    emit_error!(ty, ERR_RAW_PRIMITIVE);
                    return None;
                }
            }
        }
    }
}
