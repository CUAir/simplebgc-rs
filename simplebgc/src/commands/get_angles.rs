use crate::{Payload, PayloadParseError, RollPitchYaw};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use enumflags2::BitFlags;

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct AngleInfo {
    /// Imu angles in 14-bit resolution per full turn
    /// Units: 0,02197265625 degree
    #[bgc_raw("IMU_ANGLE")]
    imu_angle: i32,

    /// Target angles in 14-bit resolution per full turn
    /// Units: 0,02197265625 degree
    #[bgc_raw("TARGET_ANGLE")]
    target_angle: i32,

    /// Target speed that gimbal should keep, over Euler axes
    /// Units: 0,1220740379 degree/sec
    #[bgc_raw("TARGET_SPEED")]
    target_speed: i32,
}

roll_pitch_yaw!(AngleInfo, 4);