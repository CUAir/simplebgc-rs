#[macro_export]
macro_rules! read_enum {
    ($buf: ident, $name: literal, $repr: ident) => {
        read_enum!($buf, $name, $repr, $repr)
    };
    ($buf: ident, $name: literal, $repr: ident, $kind: ident) => {{
        use num_traits::{FromPrimitive};

        mashup! {
            m["from"] = from_ $kind;
            m["get"] = get_ $repr;
        }

        m! {
            FromPrimitive::"from"($buf."get"()).ok_or(PayloadParseError::InvalidEnum { name: $name.into() })
        }
    }};
}

#[macro_export]
macro_rules! read_flags {
    ($buf: ident, $name: literal, $repr: ident) => {{
        mashup! {
            m["get"] = get_ $repr;
        }

        m! {
            BitFlags::from_bits_truncate($buf."get"())
        }
    }}
}

#[macro_export]
macro_rules! read_flags_truncate {
    ($buf: ident, $name: literal, $repr: ident) => {{
        mashup! {
            m["get"] = get_ $repr;
        }

        m! {
            BitFlags::from_bits_truncate($buf."get"())
        }
    }};
}
