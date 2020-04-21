use crate::{Payload, PayloadParseError};
use bytes::{Buf, Bytes};
use enumflags2::BitFlags;
use num_traits::FromPrimitive;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ControlMode {
    /// Mode is common for all axes
    Legacy(AxisControl),
    /// Mode is per-axis
    Extended(AxisControl, AxisControl, AxisControl),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AxisControl(AxisControlMode, BitFlags<AxisControlFlags>);

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum AxisControlMode {
    /// If this mode is set for all axes, finish serial control and
    /// restore normal RC control. If set for single axis, does
    /// not change its current control mode.
    NoControl = 0,

    /// Camera travels with the given speed in the Euler
    /// coordinates until the next CMD_CONTROL commands
    /// comes. Given angle is ignored.
    Speed = 1,

    /// Camera travels to the given Euler angle with the fixed
    /// speed. Speed is decreased near target to keep control
    /// smooth. Low-pass filter may be applied for the same
    /// reason.
    Angle = 2,

    /// Camera travels with the given speed. Additionally,
    /// controller keeps the given angle and fix accumulated
    /// error by the outer PI-loop. This mode allows the most
    /// precise type of control (see fig.1 for example), but it
    /// requires pretty fast update rate to keep it smooth, or
    /// apply low-pass filtering for speed and angle.
    SpeedAngle = 3,

    /// The ANGLE parameter is used as RC signal and
    /// overrides any other signal source, assigned to this
    /// axis. Normal working range is -500..500. A special
    /// value -10000 encodes a "signal lost" condition.
    /// The flag CONTROL_FLAG_AUTO_TASK can affect this
    /// mode (see below).
    /// Prior to 2.61 frw. ver., 'SPEED' parameter is ignored.
    Rc = 4,

    /// First, the neutral point of a camera relative to a frame is
    /// found in the Euler coordinates for a given axis. Than,
    /// the given angle (in ±360° range) is added to this point,
    /// and camera travels to it. Note that the given angle does
    /// not relate to a particular motor, it relates to global Euler
    /// angles!
    RelFrame = 5,

    /// The same as the MODE_RC, but the range of the
    /// ANGLE parameter has better resolution:
    /// -16384..16384. A special value -32768 encodes a
    /// "signal lost" condition.
    /// (frw. ver. 2.66b2+)
    HighRes = 6,
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum AxisControlFlags {
    /// If mode is one of the <MODE_ANGLE,
    /// MODE_ANGLE_REL_FRAME>, the task is processed with
    /// the speed and acceleration configured for automated
    /// tasks. If the SPEED parameter is provided, it's used
    /// instead. When all target angles are reached with the 1-
    /// degree tolerance, confirmation is sent:
    /// CMD_CONFIRM(CMD_CONTROL, 1).
    /// Use this flag to move gimbal to a certain position as
    /// fast as possible, and receive confirmation when the
    /// target is reached.
    /// (frw. ver. 2.62b7+)
    AutoTask = 1 << 6,

    // This is listed as 1 << 6 in the documentation,
    // but I am assuming that was a mistake.
    /// If mode is MODE_RC, this flag forces a control in the
    /// "SPEED" mode, with the dead-band, trimming and
    /// inversion settings are NOT applied to the provided RC
    /// signal, but the LPF, Expo curve and ACC limiter are still
    /// applied. Use this flag to control gimbal from remote
    /// applications, where signal is well-defined and you need
    /// to have a direction of rotation that does not depend on
    /// gimbal's "Inverse" and "Mode" parameters.
    /// (frw. ver. 2.62b7+)
    // ForceRcSpeed = 1 << 6,

    /// Speed units changed to 0.001 deg/sec for extremely
    /// slow motion (like timelapse shooting).
    /// (frw. ver. 2.60+)
    HighResSpeed = 1 << 7,
}

impl FromPrimitive for AxisControl {
    fn from_i64(n: i64) -> Option<Self> {
        Self::from_u8(n as u8)
    }

    fn from_u8(n: u8) -> Option<Self> {
        Some(AxisControl(
            FromPrimitive::from_u8(n)?,
            BitFlags::from_bits_truncate(n),
        ))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Self::from_u8(n as u8)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ControlAxisParams {
    /// Speed of rotation. Overrides the speed settings in the GUI and
    /// from the adjustable variables.
    /// Notes:
    /// - If the acceleration limiter is enabled in the RC settings,
    ///   the actual speed is filtered by it;
    /// - For the modes "MODE_ANGLE", "MODE_RC",
    ///   "MODE_ANGLE_REL_FRAME", the value may be
    ///   omitted (set to 0). if this case, speed is taken from the
    ///   RC settings; Also, in these modes, the actual speed is
    ///   decreased near target to prevent jerks when the
    ///   ANGLE parameter given with the high rate, changes
    ///   slowly;
    /// Units: 0,1220740379 deg./sec.
    /// (0.001 deg./sec., if the CONTROL_FLAG_HIGH_RES_SPEED
    /// is set)
    speed: i16,
    /// Depends on the MODE parameter:
    /// - MODE_ANGLE, MODE_SPEED_ANGLE: encodes the target angle
    /// - MODE_SPEED: ignored
    /// - MODE_RC: encodes RC signal in range -500..500
    /// - MODE_RC_HIGH_RES: encodes RC signal in range -16384..16384
    /// Units: 0,02197265625 degree.
    angle: i16,
}

impl Payload for ControlAxisParams {
    fn from_bytes(mut b: Bytes) -> Result<Self, PayloadParseError>
    where
        Self: Sized,
    {
        Ok(ControlAxisParams {
            speed: read_enum!(b, "SPEED", i16_le, i16)?,
            angle: read_enum!(b, "ANGLE", i16_le, i16)?,
        })
    }

    fn to_bytes(&self) -> Bytes
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ControlData {
    mode: ControlMode,
    axes: (ControlAxisParams, ControlAxisParams, ControlAxisParams),
}

impl Payload for ControlData {
    fn from_bytes(mut b: Bytes) -> Result<Self, PayloadParseError>
    where
        Self: Sized,
    {
        Ok(ControlData {
            mode: if b.remaining() == 13 {
                ControlMode::Legacy(read_enum!(b, "CONTROL_MODE", u8)?)
            } else {
                ControlMode::Extended(
                    read_enum!(b, "CONTROL_MODE[0]", u8)?,
                    read_enum!(b, "CONTROL_MODE[1]", u8)?,
                    read_enum!(b, "CONTROL_MODE[2]", u8)?,
                )
            },
            axes: (
                Payload::from_bytes(b.split_to(4))?,
                Payload::from_bytes(b.split_to(4))?,
                Payload::from_bytes(b.split_to(4))?,
            ),
        })
    }

    fn to_bytes(&self) -> Bytes
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
