use super::PayloadParseError;
use enumflags2::BitFlags;
use num_traits::{FromPrimitive, ToPrimitive};

#[macro_export]
macro_rules! read_enum {
    ($buf: ident, $name: literal, $repr: ident) => {{
        mashup! {
            m["from"] = from_ $repr;
            m["get"] = get_ $repr;
        }

        m! {
            FromPrimitive::"from"($buf."get"()).ok_or(PayloadParseError::InvalidEnum { name: $name.into() })
        }
    }}
}

#[macro_export]
macro_rules! read_flags {
    ($buf: ident, $name: literal, $repr: ident) => {{
        mashup! {
            m["get"] = get_ $repr;
        }

        m! {
            BitFlags::from_bits($buf."get"()).or(Err(PayloadParseError::InvalidFlags { name: $name.into() }))
        }
    }}
}
