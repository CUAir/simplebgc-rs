use proc_macro::Ident;
use std::convert::{TryFrom, TryInto};
use std::result::Result;
use syn::{Path, Type, TypePath};

#[derive(Copy, Clone, PartialEq)]
pub enum PrimitiveKind {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    I128,
    U128,
    Bool,
}

pub struct InvalidPrimitiveError {
    ty: Type,
}

impl TryFrom<Ident> for PrimitiveKind {
    type Error = InvalidPrimitiveError;

    fn try_from(ident: Ident) -> Result<Self, Self::Error> {
        use PrimitiveKind::*;
        if ident == "u8" {
            Ok(U8)
        } else if ident == "i8" {
            Ok(I8)
        } else if ident == "u16" {
            Ok(U16)
        } else if ident == "i16" {
            Ok(I16)
        } else if ident == "u32" {
            Ok(U32)
        } else if ident == "i32" {
            Ok(I32)
        } else if ident == "u64" {
            Ok(U64)
        } else if ident == "i64" {
            Ok(I64)
        } else if ident == "u128" {
            Ok(U128)
        } else if ident == "i128" {
            Ok(I128)
        } else if ident == "bool" {
            Ok(Bool)
        } else {
            Err(InvalidPrimitiveError {
                ty: Type::Path(TypePath {
                    qself: None,
                    path: Path::from(ident),
                }),
            })
        }
    }
}

impl TryFrom<Type> for PrimitiveKind {
    type Error = InvalidPrimitiveError;

    fn try_from(ty: Type) -> Result<Self, Self::Error> {
        match ty {
            Type::Path(ref path) => match path.path.get_ident() {
                Some(ident) => ident.try_into(),
                _ => InvalidPrimitiveError { ty },
            },
            _ => InvalidPrimitiveError { ty },
        }
    }
}
