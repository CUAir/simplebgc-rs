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
            BitFlags::from_bits($buf."get"()).or(Err(PayloadParseError::InvalidFlags { name: $name.into() }))
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

#[macro_export]
macro_rules! axes_payload {
    ($type: ty, $size: literal) => {
        impl Payload for RollPitchYaw<$type> {
            fn from_bytes(mut b: Bytes) -> Result<Self, PayloadParseError>
            where
                Self: Sized,
            {
                Ok(RollPitchYaw {
                    roll: Payload::from_bytes(b.split_to($size))?,
                    pitch: Payload::from_bytes(b.split_to($size))?,
                    yaw: Payload::from_bytes(b.split_to($size))?,
                })
            }

            fn to_bytes(&self) -> Bytes
            where
                Self: Sized,
            {
                let mut b = BytesMut::with_capacity($size * 3);
                b.put(Payload::to_bytes(&self.roll));
                b.put(Payload::to_bytes(&self.pitch));
                b.put(Payload::to_bytes(&self.yaw));
                b.freeze()
            }
        }
    };
}
