use crate::*;
use bytes::{BufMut, Bytes, BytesMut};
use enumflags2::BitFlags;
use num_traits::*;

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct AxisPidParams {
    #[kind(raw)]
    pub p: u8,
    #[kind(raw)]
    pub i: u8,
    #[kind(raw)]
    pub d: u8,
    #[kind(raw)]
    pub power: u8,
    #[kind(raw)]
    #[format(u8)]
    pub invert: bool,
    #[kind(raw)]
    pub poles: u8,
}

payload_rpy!(AxisPidParams, 6);

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct AxisRcParams {
    /// Units: degrees
    #[kind(raw)]
    pub rc_min_angle: i16,
    /// Units: degrees
    #[kind(raw)]
    pub rc_max_angle: i16,
    #[kind(flags)]
    #[format(u8)]
    pub rc_mode: BitFlags<AxisRcMode>,
    #[kind(raw)]
    pub rc_lpf: u8,
    #[kind(raw)]
    pub rc_speed: u8,

    /// ROLL, PITCH: this value specify follow rate for
    /// flight controller. YAW: if value != 0, “follow motor”
    /// mode is enabled.
    #[kind(raw)]
    pub rc_follow: i8,
}

payload_rpy!(AxisRcParams, 8);

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum AxisRcMode {
    Angle = 1 << 0,
    Fixed = 1 << 1,
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
    None,
    PWM { source: RcMapPwmSource },
    Analog { channel: RcMapAnalogChannel },
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
        if b == 0 {
            return Some(RcMap::None);
        }

        let channel = b & 0b11111;
        let kind = (b & 0b00000111) >> 5;

        Some(match kind {
            0 => RcMap::PWM {
                source: FromPrimitive::from_u8(channel)?,
            },
            1 => RcMap::Analog {
                channel: FromPrimitive::from_u8(channel)?,
            },
            2 => RcMap::Serial { channel },
            4 => RcMap::Virtual { channel },
            5 => RcMap::Step { channel },
            _ => return None,
        })
    }

    fn from_u64(n: u64) -> Option<Self> {
        FromPrimitive::from_u8(n as u8)
    }
}

impl ToPrimitive for RcMap {
    fn to_i64(&self) -> Option<i64> {
        self.to_u8().map(|u| u as i64)
    }

    fn to_u8(&self) -> Option<u8> {
        Some(match self {
            RcMap::None => 0,
            RcMap::PWM { source } => ToPrimitive::to_u8(source)?,
            RcMap::Analog { channel } => 0b00100000 | ToPrimitive::to_u8(channel)?,
            RcMap::Serial { channel } => 0b01000000 | *channel,
            RcMap::Virtual { channel } => 0b10000000 | *channel,
            RcMap::Step { channel } => 0b10100000 | *channel,
        })
    }

    fn to_u64(&self) -> Option<u64> {
        self.to_u8().map(|u| u as u64)
    }
}

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum RcMixChannel {
    None = 0,
    Roll,
    Pitch,
    Yaw,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RcMix {
    pub rc_mix_rate: u8,
    pub rc_mix_channel: RcMixChannel,
}

impl Payload for RcMix {
    fn from_bytes(b: Bytes) -> Result<Self, PayloadParseError>
    where
            Self: Sized {
        // We expect one byte
        if b.len() < 1 {
            return Err(PayloadParseError::InvalidFlags{ name: "RcMixRate".into() });
        }
        let byte = b[0];
        let rc_mix_rate = 0b00011111 & byte;
        let rc_mix_channel = RcMixChannel::from_u8(byte >> 5).unwrap();
        Ok(RcMix {
            rc_mix_rate,
            rc_mix_channel,
        })
    }

    fn to_bytes(&self) -> Bytes
    where
            Self: Sized {
        let mut b = BytesMut::new();
        let byte = self.rc_mix_rate & (self.rc_mix_channel.to_u8().unwrap() << 5);
        b.put_u8(byte);
        b.freeze()
    }
}

// impl FromPrimitive for RcMix {
//     fn from_i64(n: i64) -> Option<Self> {
//         FromPrimitive::from_u8(n as u8)
//     }

//     fn from_u8(b: u8) -> Option<Self> {
//         Some(RcMix(
//             FromPrimitive::from_u8(b & 0b111111)?,
//             FromPrimitive::from_u8(b >> 5)?,
//         ))
//     }

//     fn from_u64(b: u64) -> Option<Self> {
//         FromPrimitive::from_u8(b as u8)
//     }
// }

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
pub enum BeeperMode {
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

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
pub enum SpektrumMode {
    Auto = 0,
    DSM2Short10,
    DSM2Short11,
    DSM2Long10,
    DSM2Long11,
    DSMXShort10,
    DSMXShort11,
    DSMXLong10,
    DSMXLong11,
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
    #[kind(payload)]
    #[name("RC_MIX_FC_ROLL")]
    #[size(1)]
    pub fc_roll: RcMix,

    #[kind(payload)]
    #[name("RC_MIX_FC_PITCH")]
    #[size(1)]
    pub fc_pitch: RcMix,
}

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct RcMaps {
    #[kind(enumeration)]
    #[name("RC_MAP_ROLL")]
    #[format(u8)]
    pub roll: RcMap,

    #[kind(enumeration)]
    #[name("RC_MAP_PITCH")]
    #[format(u8)]
    pub pitch: RcMap,

    #[kind(enumeration)]
    #[name("RC_MAP_YAW")]
    #[format(u8)]
    pub yaw: RcMap,

    #[kind(enumeration)]
    #[name("RC_MAP_CMD")]
    #[format(u8)]
    pub cmd: RcMap,

    #[kind(enumeration)]
    #[name("RC_MAP_FC_ROLL")]
    #[format(u8)]
    pub fc_roll: RcMap,

    #[kind(enumeration)]
    #[name("RC_MAP_FC_PITCH")]
    #[format(u8)]
    pub fc_pitch: RcMap,
}

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct ParamsQuery {
    #[kind(raw)]
    pub profile_id: u8,
}

#[derive(BgcPayload, Clone, Debug, PartialEq)]
pub struct Params3Data {
    /// profile ID to read or write. To access current (active) profile,
    /// specify 255. Possible values: 0..4
    #[kind(raw)]
    pub profile_id: u8,

    #[kind(payload)]
    #[size(18)]
    pub pid: RollPitchYaw<AxisPidParams>,

    /// Units: 5 degrees/sec^2 0 – disabled.
    /// (starting from ver. 2.60 is deprecated; replaced by the ACC_LIMITER3)
    #[kind(raw)]
    pub acc_limiter_all: u8,

    #[kind(raw)]
    pub ext_fc_gain: (i8, i8),

    #[kind(payload)]
    #[size(24)]
    pub rc: RollPitchYaw<AxisRcParams>,

    #[kind(raw)]
    pub gyro_trust: u8,

    #[kind(raw)]
    #[format(u8)]
    pub use_model: bool,

    #[kind(enumeration)]
    #[format(u8)]
    pub pwm_freq: PwmFrequency,

    #[kind(enumeration)]
    #[format(u8)]
    pub serial_speed: SerialSpeed,

    #[kind(payload)]
    #[size(3)]
    pub rc_trim: RollPitchYaw<i8>,

    #[kind(raw)]
    pub rc_deadband: u8,

    #[kind(raw)]
    pub rc_expo_rate: u8,

    #[kind(enumeration)]
    #[format(u8)]
    pub rc_virt_mode: RcVirtMode,

    #[kind(payload)]
    #[size(6)]
    pub rc_map: RcMaps,

    #[kind(payload)]
    #[size(2)]
    pub rc_mix: RcMixes,

    #[kind(enumeration)]
    #[format(u8)]
    pub follow_mode: FollowMode,

    #[kind(raw)]
    pub follow_deadband: u8,

    #[kind(raw)]
    pub follow_expo_rate: u8,

    #[kind(enumeration)]
    #[format(i8)]
    pub axis_top: Orientation,

    #[kind(enumeration)]
    #[format(i8)]
    pub axis_right: Orientation,

    #[kind(enumeration)]
    #[format(i8)]
    pub frame_axis_top: Orientation,

    #[kind(enumeration)]
    #[format(i8)]
    pub frame_axis_right: Orientation,

    #[kind(enumeration)]
    #[format(u8)]
    pub frame_imu_pos: FrameImuPos,

    #[kind(raw)]
    pub gyro_deadband: u8,

    #[kind(raw)]
    pub gyro_sens: u8,

    #[kind(raw)]
    #[format(u8)]
    pub i2c_speed_fast: bool,

    #[kind(enumeration)]
    #[format(u8)]
    pub skip_gyro_calib: GyroCalibrationMode,

    #[kind(raw)]
    pub rc_cmd: [u8; 9], // TODO: implement RC_CMD_LOW .. MENU_CMD_LONG, probably as a couple of structs

    #[kind(payload)]
    #[size(3)]
    pub motor_output: RollPitchYaw<u8>,

    /// Negative means means alarm is disabled.
    #[kind(raw)]
    pub bat_threshold_alarm: i16,
    /// Negative value means function is disabled.
    #[kind(raw)]
    pub bat_threshold_motors: i16,
    /// Negative value means compensation is disabled.
    #[kind(raw)]
    pub bat_comp_ref: i16,

    #[kind(enumeration)]
    #[format(u8)]
    pub beeper_mode: BeeperMode,

    #[kind(raw)]
    #[format(u8)]
    pub follow_roll_mix_start: u8,
    #[kind(raw)]
    #[format(u8)]
    pub follow_roll_mix_range: u8,

    #[kind(payload)]
    #[size(3)]
    pub booster_power: RollPitchYaw<u8>,

    #[kind(payload)]
    #[size(3)]
    pub follow_speed: RollPitchYaw<u8>,

    #[kind(raw)]
    #[format(u8)]
    pub frame_angle_from_motors: bool,
    /// Disabled = 0
    /// 1..32 - Virtual channel number as source of data to be output
    #[kind(raw)]
    pub servo_out: [u8; 4],
    /// PWM frequency, 10 Hz per unit.
    #[kind(raw)]
    pub servo_rate: u8,

    #[kind(flags)]
    #[format(u8)]
    pub adaptive_pid_enabled: BitFlags<AdaptivePid>,

    #[kind(raw)]
    pub adaptive_pid_threshold: u8,

    #[kind(raw)]
    pub adaptive_pid_rate: u8,

    #[kind(raw)]
    pub adaptive_pid_recovery_factor: u8,

    #[kind(flags)]
    #[format(u16)]
    pub general_flags: BitFlags<GeneralFlags>,

    #[kind(flags)]
    #[format(u16)]
    pub profile_flags: BitFlags<ProfileFlags>,

    #[kind(enumeration)]
    #[format(u8)]
    pub spektrum_mode: SpektrumMode,

    /// Order of hardware axes, counting from a camera. Implemented in
    /// special builds of firmware only.
    #[kind(enumeration)]
    #[format(u8)]
    pub order_of_axes: AxisOrder,

    /// Order of Euler angles to represent the current orientation of a
    /// camera and the target of stabilization
    #[kind(enumeration)]
    #[format(u8)]
    pub euler_order: EulerOrder,

    /// currently selected IMU
    #[kind(enumeration)]
    #[format(u8)]
    pub cur_imu: ImuType,

    /// profile ID which is currently active in the controller, 0...4
    #[kind(raw)]
    pub cur_profile_id: u8,
}
