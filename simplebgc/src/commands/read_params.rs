use crate::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use enumflags2::BitFlags;
use num_traits::FromPrimitive;

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct AxisPidParams {
    #[bgc_raw]
    p: u8,
    #[bgc_raw]
    i: u8,
    #[bgc_raw]
    d: u8,
    #[bgc_raw]
    power: u8,
    #[bgc_raw]
    #[bgc_repr(u8)]
    invert: bool,
    #[bgc_raw]
    poles: u8,
}

roll_pitch_yaw!(AxisPidParams, 6);

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct AxisRcParams {
    /// Units: degrees
    #[bgc_raw]
    rc_min_angle: i16,
    /// Units: degrees
    #[bgc_raw]
    rc_max_angle: i16,
    #[bgc_flags]
    #[bgc_repr(u8)]
    rc_mode: BitFlags<AxisRcMode>,
    #[bgc_raw]
    rc_lpf: u8,
    #[bgc_raw]
    rc_speed: u8,

    /// ROLL, PITCH: this value specify follow rate for
    /// flight controller. YAW: if value != 0, “follow motor”
    /// mode is enabled.
    #[bgc_raw]
    rc_follow: i8,
}

roll_pitch_yaw!(AxisRcParams, 8);

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum AxisRcMode {
    Angle = 1 << 0,
    Speed = 1 << 1,
    Inverted = 1 << 2,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum PwmFrequency {
    Low = 0,
    High = 1,
    UltraHigh = 2,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum SerialSpeed {
    /// 115200
    B115200 = 0,
    /// 57600
    B57600,
    /// 38400
    B38400,
    /// 19200
    B19200,
    /// 9600
    B9600,
    /// 256000
    B25600,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum RcVirtMode {
    Normal = 0,
    CPPM,
    SBus,
    Spektrum,
    API = 10,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RcMap {
    PWM { source: RcMapPwmSource },
    Analog { channel: u8 },
    Serial { channel: u8 },
    Virtual { channel: u8 },
    Step { channel: u8 },
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum RcMapPwmSource {
    Roll,
    Pitch,
    ExtFcRoll,
    ExtFcPitch,
    Yaw,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum RcMapAnalogChannel {
    ADC1 = 1,
    ADC2 = 2,
    ADC3 = 4,
}

impl FromPrimitive for RcMap {
    fn from_i64(n: i64) -> Option<Self> {
        FromPrimitive::from_u8(n as u8)
    }

    fn from_u8(b: u8) -> Option<Self> {
        let chan = b & 0b11111;
        let kind = (b & 0b00000111) >> 5;

        Some(match kind {
            0 => RcMap::PWM {
                source: FromPrimitive::from_u8(chan)?,
            },
            1 => RcMap::Analog {
                channel: FromPrimitive::from_u8(chan)?,
            },
            2 => RcMap::Serial { channel: chan },
            4 => RcMap::Virtual { channel: chan },
            5 => RcMap::Step { channel: chan },
            _ => return None,
        })
    }

    fn from_u64(n: u64) -> Option<Self> {
        FromPrimitive::from_u8(n as u8)
    }
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum RcMixRate {
    FullRc = 0,
    HalfHalf = 32,
    FullFc = 63,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum RcMixChannel {
    None = 0,
    Roll,
    Pitch,
    Yaw,
}

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct RcMix(
    #[bgc_enum("")]
    #[bgc_repr(u8)]
    RcMixRate,
    #[bgc_enum("")]
    #[bgc_repr(u8)]
    RcMixChannel,
);

impl FromPrimitive for RcMix {
    fn from_i64(n: i64) -> Option<Self> {
        FromPrimitive::from_u8(n as u8)
    }

    fn from_u8(b: u8) -> Option<Self> {
        Some(RcMix(
            FromPrimitive::from_u8(b & 0b111111)?,
            FromPrimitive::from_u8(b >> 5)?,
        ))
    }

    fn from_u64(b: u64) -> Option<Self> {
        FromPrimitive::from_u8(b as u8)
    }
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum FollowMode {
    Disabled = 0,
    Fc,
    Pitch,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(i8)]
pub enum Orientation {
    PosX = 1,
    PosY,
    PosZ,
    NegX = -1,
    NegY = -2,
    NegZ = -3,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum FrameImuPos {
    Disabled = 0,
    BelowYaw,
    AboveYaw,
    BelowYawPIDSource,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum GyroCalibrationMode {
    /// do not skip
    NoSkip = 0,
    /// skip always
    Skip,
    /// try to calibrate but skip if motion is detected
    Attempt,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MotorOutput {
    Disabled = 0,
    Roll,
    Pitch,
    Yaw,
    I2CDrv1,
    I2CDrv2,
    I2CDrv3,
    I2CDrv4,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
enum BeeperMode {
    Calibrate = 1,
    Confirm = 2,
    Error = 4,
    Alarm = 8,
    Motors = 128,
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum AdaptivePid {
    Roll = 1,
    Pitch = 2,
    Yaw = 4,
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum GeneralFlags {
    RememberLastUsedProfile = 1 << 0,
    UpsideDownAuto = 1 << 1,
    SwapFrameMainImu = 1 << 2,
    BlinkProfile = 1 << 3,
    EmergencyStop = 1 << 4,
    MagnetometerPosFrame = 1 << 5,
    FrameImuFF = 1 << 6,
    OverheatStopMotors = 1 << 7,
    CenterYawAtStartup = 1 << 8,
    SwapRcSerialUartB = 1 << 9,
    UartBSerialApi = 1 << 10,
    BlinkBatLevel = 1 << 11,
    AdaptiveGyroTrust = 1 << 12,
    IsUpsideDown = 1 << 13,
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum ProfileFlags {
    Adc1AutoDetection = 1 << 0,
    Adc2AutoDetection = 1 << 1,
    Adc3AutoDetection = 1 << 2,
    FollowUseFrameImu = 1 << 4,
    BriefcaseAutoDetection = 1 << 5,
    UpsideDownAutoRotate = 1 << 6,
    FollowLockOffsetCorrection = 1 << 7,
    StartNeutralPosition = 1 << 8,
    MenuButtonDisableFollow = 1 << 9,
    TimelapseFrameFixed = 1 << 10,
    RcKeepMixRate = 1 << 11,
    RcKeepCurPosOnInit = 1 << 12,
    OuterMotorLimitFreeRotation = 1 << 13,
    EulerOrderAuto = 1 << 14,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum SpektrumModeDSM {
    DSM2 = 0,
    DSMX,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum SpektrumModeTime {
    /// 11ms
    Short = 0,
    /// 22ms
    Long,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum SpektrumModeBits {
    /// 10bits
    Short = 0,
    /// 11bits
    Long,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SpektrumMode {
    Auto,
    Mode(SpektrumModeDSM, SpektrumModeTime, SpektrumModeBits),
}

impl FromPrimitive for SpektrumMode {
    fn from_i64(n: i64) -> Option<Self> {
        FromPrimitive::from_u8(n as u8)
    }

    fn from_u8(b: u8) -> Option<Self> {
        if b == 0 {
            Some(SpektrumMode::Auto)
        } else {
            let value = b - 1;
            if value > 7 {
                // no valid values here
                None
            } else {
                Some(SpektrumMode::Mode(
                    if value & 4 == 4 {
                        SpektrumModeDSM::DSMX
                    } else {
                        SpektrumModeDSM::DSM2
                    },
                    if value & 2 == 2 {
                        SpektrumModeTime::Long
                    } else {
                        SpektrumModeTime::Short
                    },
                    if value & 1 == 1 {
                        SpektrumModeBits::Long
                    } else {
                        SpektrumModeBits::Short
                    },
                ))
            }
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        FromPrimitive::from_u8(n as u8)
    }
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum AxisOrder {
    PitchRollYaw = 0,
    YawRollPitch,
    RollYawPitch,
    RollPitchYaw,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum EulerOrder {
    PitchRollYaw = 0,
    RollPitchYaw,
    LocalRoll,
    RollLocal,
    YawRollPitch,
    YawPitchRoll,
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ImuType {
    Main = 1,
    Frame,
}

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct RcMixes {
    #[bgc_payload("RC_MIX_FC_ROLL")]
    #[bgc_size(2)]
    fc_roll: RcMix,

    #[bgc_payload("RC_MIX_FC_PITCH")]
    #[bgc_size(2)]
    fc_pitch: RcMix,
}

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct RcMaps {
    #[bgc_enum("RC_MAP_ROLL")]
    #[bgc_repr(u8)]
    roll: RcMap,

    #[bgc_enum("RC_MAP_PITCH")]
    #[bgc_repr(u8)]
    pitch: RcMap,

    #[bgc_enum("RC_MAP_YAW")]
    #[bgc_repr(u8)]
    yaw: RcMap,

    #[bgc_enum("RC_MAP_CMD")]
    #[bgc_repr(u8)]
    cmd: RcMap,

    #[bgc_enum("RC_MAP_FC_ROLL")]
    #[bgc_repr(u8)]
    fc_roll: RcMap,

    #[bgc_enum("RC_MAP_FC_PITCH")]
    #[bgc_repr(u8)]
    fc_pitch: RcMap,
}

#[derive(BgcPayload, Clone, Debug, PartialEq)]
pub struct Params3Data {
    /// profile ID to read or write. To access current (active) profile,
    /// specify 255. Possible values: 0..4
    #[bgc_raw("PROFILE_ID")]
    profile_id: u8,

    #[bgc_payload]
    #[bgc_size(18)]
    pid: RollPitchYaw<AxisPidParams>,

    /// Units: 5 degrees/sec^2 0 – disabled.
    /// (starting from ver. 2.60 is deprecated; replaced by the ACC_LIMITER3)
    #[bgc_raw]
    acc_limiter_all: u8,

    #[bgc_raw]
    ext_fc_gain: (i8, i8),

    #[bgc_payload]
    #[bgc_size(24)]
    rc: RollPitchYaw<AxisRcParams>,

    #[bgc_raw]
    gyro_trust: u8,

    #[bgc_raw]
    #[bgc_repr(u8)]
    use_model: bool,

    #[bgc_enum]
    #[bgc_repr(u8)]
    pwm_freq: PwmFrequency,

    #[bgc_enum]
    #[bgc_repr(u8)]
    serial_speed: SerialSpeed,

    #[bgc_payload]
    #[bgc_size(3)]
    rc_trim: RollPitchYaw<i8>,

    #[bgc_raw]
    rc_deadband: u8,

    #[bgc_raw]
    rc_expo_rate: u8,

    #[bgc_enum]
    #[bgc_repr(u8)]
    rc_virt_mode: RcVirtMode,

    #[bgc_payload]
    #[bgc_size(6)]
    rc_map: RcMaps,

    #[bgc_payload]
    #[bgc_size(2)]
    rc_mix: RcMixes,

    #[bgc_enum]
    #[bgc_repr(u8)]
    follow_mode: FollowMode,

    #[bgc_raw]
    follow_deadband: u8,

    #[bgc_raw]
    follow_expo_rate: u8,

    #[bgc_enum]
    #[bgc_repr(i8)]
    axis_top: Orientation,

    #[bgc_enum]
    #[bgc_repr(i8)]
    axis_right: Orientation,

    #[bgc_enum]
    #[bgc_repr(i8)]
    frame_axis_top: Orientation,

    #[bgc_enum]
    #[bgc_repr(i8)]
    frame_axis_right: Orientation,

    #[bgc_enum]
    #[bgc_repr(u8)]
    frame_imu_pos: FrameImuPos,

    #[bgc_raw]
    gyro_deadband: u8,

    #[bgc_raw]
    gyro_sens: u8,

    #[bgc_raw]
    #[bgc_repr(u8)]
    i2c_speed_fast: bool,

    #[bgc_enum]
    #[bgc_repr(u8)]
    skip_gyro_calib: GyroCalibrationMode,

    #[bgc_raw]
    rc_cmd: [u8; 9], // TODO: implement RC_CMD_LOW .. MENU_CMD_LONG, probably as a couple of structs

    #[bgc_payload]
    #[bgc_size(3)]
    motor_output: RollPitchYaw<u8>,

    /// Negative means means alarm is disabled.
    #[bgc_raw]
    bat_threshold_alarm: i16,
    /// Negative value means function is disabled.
    #[bgc_raw]
    bat_threshold_motors: i16,
    /// Negative value means compensation is disabled.
    #[bgc_raw]
    bat_comp_ref: i16,
    #[bgc_enum]
    #[bgc_repr(u8)]
    beeper_mode: BeeperMode,
    #[bgc_raw]
    #[bgc_repr(u8)]
    follow_roll_mix_start: u8,
    #[bgc_raw]
    #[bgc_repr(u8)]
    follow_roll_mix_range: u8,

    #[bgc_payload]
    #[bgc_size(3)]
    booster_power: RollPitchYaw<u8>,

    #[bgc_payload]
    #[bgc_size(3)]
    follow_speed: RollPitchYaw<u8>,

    #[bgc_raw]
    #[bgc_repr(u8)]
    frame_angle_from_motors: bool,
    /// Disabled = 0
    /// 1..32 - Virtual channel number as source of data to be output
    #[bgc_raw]
    servo_out: [u8; 4],
    /// PWM frequency, 10 Hz per unit.
    #[bgc_raw]
    servo_rate: u8,

    #[bgc_flags]
    #[bgc_repr(u8)]
    adaptive_pid_enabled: BitFlags<AdaptivePid>,

    #[bgc_raw]
    adaptive_pid_threshold: u8,

    #[bgc_raw]
    adaptive_pid_rate: u8,

    #[bgc_raw]
    adaptive_pid_recovery_factor: u8,

    #[bgc_flags]
    #[bgc_repr(u16)]
    general_flags: BitFlags<GeneralFlags>,

    #[bgc_flags]
    #[bgc_repr(u16)]
    profile_flags: BitFlags<ProfileFlags>,

    #[bgc_enum]
    #[bgc_repr(u8)]
    spektrum_mode: SpektrumMode,

    /// Order of hardware axes, counting from a camera. Implemented in
    /// special builds of firmware only.
    #[bgc_enum]
    #[bgc_repr(u8)]
    order_of_axes: AxisOrder,

    /// Order of Euler angles to represent the current orientation of a
    /// camera and the target of stabilization
    #[bgc_enum]
    #[bgc_repr(u8)]
    euler_order: EulerOrder,

    /// currently selected IMU
    #[bgc_enum]
    #[bgc_repr(u8)]
    cur_imu: ImuType,

    /// profile ID which is currently active in the controller, 0...4
    #[bgc_raw]
    cur_profile_id: u8,
}
