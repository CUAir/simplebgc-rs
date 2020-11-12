use crate::*;
use bytes::{BufMut, Bytes, BytesMut};
use enumflags2::BitFlags;

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct AccGyroData {
    #[kind(raw)]
    pub acc_data: i16,

    #[kind(raw)]
    pub gyro_data: i16,
}

payload_rpy!(AccGyroData, 4);

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct RcRPY (
    #[kind(raw)]
    #[name("")]
    pub i16,
);

payload_rpy!(RcRPY, 2);

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct ImuAngle (
    // units: 0.02197265625 degrees (360 / 2^14)
    #[kind(raw)]
    #[name("")]
    pub i16,
);

payload_rpy!(ImuAngle, 2);

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct FrameImuAngle (
    // units: 0.02197265625 degrees (360 / 2^14)
    #[kind(raw)]
    #[name("")]
    pub i16,
);

payload_rpy!(FrameImuAngle, 2);

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct TargetAngle (
    // units: 0.02197265625 degrees (360 / 2^14)
    #[kind(raw)]
    #[name("")]
    pub i16,
);

payload_rpy!(TargetAngle, 2);

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum RTDataFlags {
    MotorsOn = 1 << 0,
}

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct MotorPower (
    #[kind(raw)]
    #[name("")]
    pub u8,
);

payload_rpy!(MotorPower, 1);

#[derive(BgcPayload, Clone, Debug, PartialEq)]
pub struct RealtimeData3 {
    #[kind(payload)]
    #[size(12)]
    pub acc_gyro_data: RollPitchYaw<AccGyroData>,

    #[kind(raw)]
    pub serial_err_cnt: u16,

    #[kind(raw)]
    pub system_error: u16,

    #[kind(raw)]
    pub system_sub_error: u8,

    #[kind(raw)]
    pub reserved: [u8; 3],

    #[kind(payload)]
    #[size(6)]
    pub rc_rpy: RollPitchYaw<RcRPY>,

    #[kind(raw)]
    pub rc_cmd: i16,

    #[kind(raw)]
    pub ext_fc_roll: i16,

    #[kind(raw)]
    pub ext_fc_pitch: i16,

    #[kind(payload)]
    #[size(6)]
    pub imu_angle: RollPitchYaw<ImuAngle>,

    #[kind(payload)]
    #[size(6)]
    pub frame_imu_angle: RollPitchYaw<FrameImuAngle>,

    #[kind(payload)]
    #[size(6)]
    pub target_angle: RollPitchYaw<TargetAngle>,

    #[kind(raw)]
    pub cycle_time: u16,

    #[kind(raw)]
    pub i2c_error_count: u16,

    #[kind(raw)]
    pub error_code: u8,

    #[kind(raw)]
    pub bat_level: u16,

    #[kind(flags)]
    #[format(u8)]
    pub rt_data_flags: BitFlags<RTDataFlags>,

    #[kind(raw)]
    pub cur_imu: u8,

    #[kind(raw)]
    pub cur_profile: u8,

    #[kind(payload)]
    #[size(3)]
    pub motor_power: RollPitchYaw<MotorPower>,
}