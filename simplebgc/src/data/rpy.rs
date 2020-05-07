#[derive(Clone, Debug, PartialEq)]
pub struct RollPitchYaw<T> {
    pub roll: T,
    pub pitch: T,
    pub yaw: T,
}

impl<T> RollPitchYaw<T> {
    pub fn combine<U>(self, other: RollPitchYaw<U>) -> RollPitchYaw<(T, U)> {
        RollPitchYaw {
            roll: (self.roll, other.roll),
            pitch: (self.pitch, other.pitch),
            yaw: (self.yaw, other.yaw),
        }
    }

    pub fn combine_ref<'a, U>(&self, other: &'a RollPitchYaw<U>) -> RollPitchYaw<(&T, &'a U)> {
        RollPitchYaw {
            roll: (&self.roll, &other.roll),
            pitch: (&self.pitch, &other.pitch),
            yaw: (&self.yaw, &other.yaw),
        }
    }

    pub fn combine_mut<'a, U>(
        &mut self,
        other: &'a mut RollPitchYaw<U>,
    ) -> RollPitchYaw<(&mut T, &'a mut U)> {
        RollPitchYaw {
            roll: (&mut self.roll, &mut other.roll),
            pitch: (&mut self.pitch, &mut other.pitch),
            yaw: (&mut self.yaw, &mut other.yaw),
        }
    }

    pub fn map<U, F: Fn(T) -> U>(self, op: F) -> RollPitchYaw<U> {
        RollPitchYaw {
            roll: op(self.roll),
            pitch: op(self.pitch),
            yaw: op(self.yaw),
        }
    }

    pub fn update<U, F: Fn(&mut T) -> U>(&mut self, op: F) -> RollPitchYaw<U> {
        RollPitchYaw {
            roll: op(&mut self.roll),
            pitch: op(&mut self.pitch),
            yaw: op(&mut self.yaw),
        }
    }

    pub fn exec<U, F: Fn(&T) -> U>(&self, op: F) {
        op(&self.roll);
        op(&self.pitch);
        op(&self.yaw);
    }
}

impl<T> Into<(T, T, T)> for RollPitchYaw<T> {
    fn into(self) -> (T, T, T) {
        (self.roll, self.pitch, self.yaw)
    }
}

impl<T> From<(T, T, T)> for RollPitchYaw<T> {
    fn from(t: (T, T, T)) -> Self {
        RollPitchYaw {
            roll: t.0,
            pitch: t.1,
            yaw: t.2,
        }
    }
}

impl<T: Copy> Copy for RollPitchYaw<T> {}

#[macro_export]
macro_rules! payload_rpy {
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