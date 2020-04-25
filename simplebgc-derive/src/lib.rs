extern crate proc_macro;
extern crate proc_macro_error;
extern crate quote;

use proc_macro::{TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::*;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::*;

enum FieldKind {
    Flags,
    Enum,
    Payload,
    Raw,
}

#[proc_macro_error]
#[proc_macro_derive(
    BgcPayload,
    attributes(bgc_flags, bgc_enum, bgc_payload, bgc_raw, bgc_size, bgc_repr)
)]
pub fn payload_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_ident = input.ident;

    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let parse_info: Vec<ParseStatement> = fields
                    .named
                    .iter()
                    .enumerate()
                    .filter_map(|(i, field)| get_stmt_for_field(i, field))
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
                let parse_info: Vec<ParseStatement> = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter_map(|(i, field)| get_stmt_for_field(i, field))
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

/// Returns (variable identifier, field identifier, parse statement)
fn get_stmt_for_field(idx: usize, field: &Field) -> Option<ParseStatement> {
    let info = get_info_for_field(idx, field)?;
    let spec_name = info.spec_name;
    let spec_repr = info.spec_repr;
    let variable_ident = info.variable_ident;
    let field_kind = info.kind;
    let field_ident = &field.ident;

    match field.vis {
        Visibility::Public(_) => {},
        _ => emit_error!(&field, "bgc payload fields should be public")
    }

    match field_kind {
        FieldKind::Payload => {
            let field_type = &field.ty;
            let spec_size = match info.spec_size {
                Some(s) => s,
                None => {
                    emit_error!(field, "size must be specified for payload fields");
                    return None;
                }
            };

            Some(ParseStatement {
                variable_ident: variable_ident.clone(),
                field_ident: field_ident.clone(),
                stmt: quote_spanned! {field.span()=>
                    let #variable_ident: #field_type = Payload::from_bytes(b.split_to(#spec_size))?;
                },
            })
        }
        FieldKind::Flags | FieldKind::Enum => {
            let repr = match spec_repr {
                Some(r) => r,
                None => {
                    emit_error!(field.ty, "repr must be specified for enum and flags values");
                    return None;
                }
            };

            let get_value = if repr == "u8" || repr == "i8" {
                format_ident!("get_{}", repr)
            } else {
                format_ident!("get_{}_le", repr)
            };

            match field_kind {
                FieldKind::Flags => Some(ParseStatement {
                    variable_ident: variable_ident.clone(),
                    field_ident: field_ident.clone(),
                    stmt: quote! {
                        let #variable_ident = BitFlags::from_bits(b.#get_value())
                            .or(Err(PayloadParseError::InvalidFlags { name: #spec_name.into() }))?;
                    },
                }),
                FieldKind::Enum => {
                    let from_value = format_ident!("from_{}", repr);
                    Some(ParseStatement {
                        variable_ident: variable_ident.clone(),
                        field_ident: field_ident.clone(),
                        stmt: quote! {
                            let #variable_ident = FromPrimitive::#from_value(b.#get_value())
                                .ok_or(PayloadParseError::InvalidEnum { name: #spec_name.into() })?;
                        },
                    })
                }
                _ => unreachable!(),
            }
        }
        FieldKind::Raw => match &field.ty {
            Type::Path(ty) => match ty.path.get_ident() {
                Some(repr)
                    if repr == "u8"
                        || repr == "i8"
                        || repr == "u16"
                        || repr == "i16"
                        || repr == "u32"
                        || repr == "i32"
                        || repr == "u64"
                        || repr == "i64"
                        || repr == "u128"
                        || repr == "i128" =>
                {
                    let get_value = if repr == "u8" || repr == "i8" {
                        format_ident!("get_{}", repr)
                    } else {
                        format_ident!("get_{}_le", repr)
                    };

                    Some(ParseStatement {
                        variable_ident: variable_ident.clone(),
                        field_ident: field_ident.clone(),
                        stmt: quote_spanned! {field.span()=>
                            let #variable_ident = b.#get_value();
                        },
                    })
                }
                Some(i) if i == "bool" => {
                    let spec_repr = match spec_repr {
                        Some(s) => s,
                        None => {
                            emit_error!(field, "bgc_repr is required for boolean fields");
                            return None;
                        }
                    };

                    let get_value = if spec_repr == "u8" || spec_repr == "i8" {
                        format_ident!("get_{}", spec_repr)
                    } else {
                        format_ident!("get_{}_le", spec_repr)
                    };

                    Some(ParseStatement {
                        variable_ident: variable_ident.clone(),
                        field_ident: field_ident.clone(),
                        stmt: quote_spanned! {field.span()=>
                            let #variable_ident = b.#get_value() != 0;
                        },
                    })
                }
                _ => {
                    emit_error!(field.ty, ERR_RAW_PRIMITIVE);
                    return None;
                }
            },
            Type::Array(ty) => match ty.elem.as_ref() {
                Type::Path(elem_ty) => match elem_ty.path.get_ident() {
                    Some(i) if i == "u8" => {
                        let len = &ty.len;

                        Some(ParseStatement {
                            variable_ident: variable_ident.clone(),
                            field_ident: field_ident.clone(),
                            stmt: quote_spanned! {field.span()=>
                                let mut #variable_ident = [0u8; #len];
                                b.copy_to_slice(&mut #variable_ident[..]);
                            },
                        })
                    }
                    _ => {
                        emit_error!(field.ty, ERR_RAW_PRIMITIVE);
                        return None;
                    }
                },
                _ => {
                    emit_error!(field.ty, ERR_RAW_PRIMITIVE);
                    return None;
                }
            },
            Type::Tuple(ty) => {
                let mut item_parse_stmts = vec![];
                for elem_ty in ty.elems.iter() {
                    match elem_ty {
                        Type::Path(ty) => match ty.path.get_ident() {
                            Some(repr)
                                if repr == "u8"
                                    || repr == "i8"
                                    || repr == "u16"
                                    || repr == "i16"
                                    || repr == "u32"
                                    || repr == "i32"
                                    || repr == "u64"
                                    || repr == "i64"
                                    || repr == "u128"
                                    || repr == "i128" =>
                            {
                                let get_value = if repr == "u8" || repr == "i8" {
                                    format_ident!("get_{}", repr)
                                } else {
                                    format_ident!("get_{}_le", repr)
                                };

                                item_parse_stmts
                                    .push(quote_spanned! {field.span()=> b.#get_value() });
                            }
                            _ => {
                                emit_error!(elem_ty, ERR_RAW_PRIMITIVE);
                                return None;
                            }
                        },
                        _ => {
                            emit_error!(elem_ty, ERR_RAW_PRIMITIVE);
                            return None;
                        }
                    }
                }

                Some(ParseStatement {
                    variable_ident: variable_ident.clone(),
                    field_ident: field_ident.clone(),
                    stmt: quote_spanned! {field.span()=>
                        let #variable_ident = (#(#item_parse_stmts),*);
                    },
                })
            }
            _ => {
                emit_error!(field.ty, ERR_RAW_PRIMITIVE);
                return None;
            }
        },
    }
}

fn get_info_for_field(idx: usize, field: &Field) -> Option<FieldInfo> {
    // get name of field in Rust
    // or generate one if it doesn't have a name
    let field_ident = field.ident.clone();
    let variable_ident = field_ident.clone().unwrap_or(format_ident!("field{}", idx));
    let mut field_kind = None;
    let mut spec_name = None;
    let mut spec_size = None;
    let mut spec_repr = None;

    for attr in field.attrs.iter() {
        if attr.path.is_ident("bgc_repr") {
            if spec_repr.is_some() {
                emit_error!(attr, "multiple repr attributes on field not allowed");
                return None;
            }

            spec_repr = Some(match attr.parse_args::<Type>() {
                Ok(ty) => match ty {
                    Type::Path(ty) => match get_primitive_int_kind(&ty) {
                        Some(ty) => ty.clone(),
                        _ => {
                            emit_error!(attr, "invalid repr attribute");
                            return None;
                        }
                    },
                    _ => {
                        emit_error!(attr, "invalid repr attribute");
                        return None;
                    }
                },
                _ => {
                    emit_error!(attr, "invalid repr attribute");
                    return None;
                }
            });

            continue;
        }

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
                },
                Err(_) => {
                    emit_error!(attr, "invalid size attribute");
                    return None;
                }
            });

            continue;
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
                        if let Some(i) = &field.ident {
                            Some(i.to_string().to_uppercase())
                        } else {
                            emit_error!(
                                attr,
                                "unnamed fields must include name of this field in the SimpleBGC spec"
                            );
                            return None;
                        }
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

    Some(FieldInfo {
        kind: field_kind,
        variable_ident,
        spec_name,
        spec_repr,
        spec_size,
    })
}

struct FieldInfo {
    kind: FieldKind,
    variable_ident: Ident,
    spec_repr: Option<Ident>,
    spec_name: String,
    spec_size: Option<usize>,
}

struct ParseStatement {
    variable_ident: Ident,
    field_ident: Option<Ident>,
    stmt: TokenStream2,
}
