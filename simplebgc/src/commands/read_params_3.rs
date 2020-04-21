use crate::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use enumflags2::BitFlags;
use num_traits::FromPrimitive;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AxisParams {
    pid: (u8, u8, u8),
    power: u8,
    invert: bool,
    poles: u8,

    /// Units: degrees
    rc_min_angle: i16,
    /// Units: degrees
    rc_max_angle: i16,
    rc_mode: AxisRcMode,
    rc_lpf: u8,
    rc_speed: u8,

    /// ROLL, PITCH: this value specify follow rate for
    /// flight controller. YAW: if value != 0, “follow motor”
    /// mode is enabled.
    rc_follow: i8,
    rc_trim: i8,
    follow_offset: i8,

    /// Additional power to correct lost synchronization
    booster_power: u8,
    follow_speed: u8,

    /// Initial angle that is set at system start-up, in 14bit resolution
    /// Units: 0,02197265625 degree
    rc_memory: i16,
    follow_lpf: u8,
}

impl Payload for AxisParams {
    fn from_bytes(b: Bytes) -> Result<Self, PayloadParseError>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn to_bytes(&self) -> Bytes
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

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
    Level0,
    /// 57600
    Level1,
    /// 38400
    Level2,
    /// 19200
    Level3,
    /// 9600
    Level4,
    /// 256000
    Level5,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RcMix(RcMixRate, RcMixChannel);

impl FromPrimitive for RcMix {
    fn from_i64(n: i64) -> Option<Self> {
        FromPrimitive::from_u64(n as u64)
    }

    fn from_u64(b: u64) -> Option<Self> {
        let b = b as u8;
        Some(RcMix(
            FromPrimitive::from_u8(b & 0b111111)?,
            FromPrimitive::from_u8(b >> 5)?,
        ))
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RcMixes {
    fc_roll: RcMix,
    fc_pitch: RcMix,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RcMaps {
    roll: RcMap,
    pitch: RcMap,
    yaw: RcMap,
    cmd: RcMap,
    fc_roll: RcMap,
    fc_pitch: RcMap,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Params3Data {
    /// profile ID to read or write. To access current (active) profile,
    /// specify 255. Possible values: 0..4
    profile_id: u8,
    axes: (AxisParams, AxisParams, AxisParams),
    /// Units: 5 degrees/sec^2 0 – disabled.
    /// (starting from ver. 2.60 is deprecated; replaced by the ACC_LIMITER3)
    acc_limiter_all: u8,
    ext_fc_gain: [i8; 2],
    gyro_trust: u8,
    use_model: bool,
    pwm_freq: PwmFrequency,
    serial_speed: SerialSpeed,
    rc_deadband: u8,
    rc_expo_rate: u8,
    rc_virt_mode: RcVirtMode,
    rc_map: RcMaps,
    rc_mix: RcMixes,
    follow_mode: FollowMode,
    follow_deadband: u8,
    follow_expo_rate: u8,
    axis_top: Orientation,
    axis_right: Orientation,
    frame_axis_top: Orientation,
    frame_axis_right: Orientation,
    frame_imu_pos: FrameImuPos,
    gyro_deadband: u8,
    gyro_sens: u8,
    i2c_speed_fast: bool,
    skip_gyro_calib: GyroCalibrationMode,
    /// Negative means means alarm is disabled.
    bat_threshold_alarm: i16,
    /// Negative value means function is disabled.
    bat_threshold_motors: i16,
    /// Negative value means compensation is disabled.
    bat_comp_ref: i16,
    beeper_mode: BeeperMode,
    follow_roll_mix_start: u8,
    follow_roll_mix_range: u8,
    frame_angle_from_motors: bool,
    /// Disabled = 0
    /// 1..32 - Virtual channel number as source of data to be output
    servo_out: [u8; 4],
    /// PWM frequency, 10 Hz per unit.
    servo_rate: u8,
    adaptive_pid_enabled: BitFlags<AdaptivePid>,
    adaptive_pid_threshold: u8,
    adaptive_pid_rate: u8,
    adaptive_pid_recovery_factor: u8,
    general_flags: BitFlags<GeneralFlags>,
    profile_flags: BitFlags<ProfileFlags>,
    spektrum_mode: SpektrumMode,
    /// Order of hardware axes, counting from a camera. Implemented in
    /// special builds of firmware only.
    order_of_axes: AxisOrder,
    /// Order of Euler angles to represent the current orientation of a
    /// camera and the target of stabilization
    euler_order: EulerOrder,
    /// currently selected IMU
    cur_imu: ImuType,
    /// profile ID which is currently active in the controller, 0...4
    cur_profile_id: u8,
}

impl Payload for Params3Data {
    fn from_bytes(mut b: Bytes) -> Result<Self, PayloadParseError> {
        let profile_id = b.get_u8();

        // start w/ PID data
        let mut axis_data = [
            BytesMut::with_capacity(15),
            BytesMut::with_capacity(15),
            BytesMut::with_capacity(15),
        ];

        for axis_buf in axis_data.iter_mut() {
            let mut tmp = Vec::with_capacity(6);
            b.copy_to_slice(&mut tmp[..]);
            axis_buf.put(&tmp[..]); // P..POLES
        }

        let acc_limiter_all = b.get_u8();
        let ext_fc_gain = [b.get_i8(), b.get_i8()];

        for axis_buf in axis_data.iter_mut() {
            let mut tmp = Vec::with_capacity(8);
            b.copy_to_slice(&mut tmp[..]);
            axis_buf.put(&tmp[..]); // RC_MIN_ANGLE..RC_FOLLOW
        }

        let gyro_trust = b.get_u8();
        let use_model = b.get_u8() != 0;
        let pwm_freq: PwmFrequency = read_enum!(b, "PWM_FREQUENCY", u8)?;
        let serial_speed: SerialSpeed = read_enum!(b, "SERIAL_SPEED", u8)?;

        for axis_buf in axis_data.iter_mut() {
            axis_buf.put_i8(b.get_i8()); // RC_TRIM
        }

        let rc_deadband = b.get_u8();
        let rc_expo_rate = b.get_u8();
        let rc_virt_mode: RcVirtMode = read_enum!(b, "RC_VIRT_MODE", u8)?;

        let rc_map = RcMaps {
            roll: read_enum!(b, "RC_MAP_ROLL", u8)?,
            pitch: read_enum!(b, "RC_MAP_PITCH", u8)?,
            yaw: read_enum!(b, "RC_MAP_YAW", u8)?,
            cmd: read_enum!(b, "RC_MAP_CMD", u8)?,
            fc_roll: read_enum!(b, "RC_MAP_FC_ROLL", u8)?,
            fc_pitch: read_enum!(b, "RC_MAP_FC_PITCH", u8)?,
        };

        let rc_mix = RcMixes {
            fc_roll: read_enum!(b, "RC_MIX_FC_ROLL", u8)?,
            fc_pitch: read_enum!(b, "RC_MIX_FC_PITCH", u8)?,
        };

        let follow_mode: FollowMode = read_enum!(b, "FOLLOW_MODE", u8)?;
        let follow_deadband = b.get_u8();
        let follow_expo_rate = b.get_u8();

        for axis_buf in axis_data.iter_mut() {
            axis_buf.put_i8(b.get_i8()); // FOLLOW_OFFSET
        }

        let axis_top: Orientation = read_enum!(b, "AXIS_TOP", i8)?;
        let axis_right: Orientation = read_enum!(b, "AXIS_RIGHT", i8)?;
        let frame_axis_top: Orientation = read_enum!(b, "FRAME_AXIS_TOP", i8)?;
        let frame_axis_right: Orientation = read_enum!(b, "FRAME_AXIS_RIGHT", i8)?;

        let frame_imu_pos: FrameImuPos = read_enum!(b, "FRAME_IMU_POS", u8)?;
        let gyro_deadband = b.get_u8();
        let gyro_sens = b.get_u8();
        let i2c_speed_fast = b.get_u8() != 0;
        let skip_gyro_calib: GyroCalibrationMode = read_enum!(b, "SKIP_GYRO_CALIB", u8)?;

        // RC_CMD_LOW..MENU_CMD_LONG
        b.advance(9);

        for axis_buf in axis_data.iter_mut() {
            axis_buf.put_u8(b.get_u8()); // MOTOR_OUTPUT
        }

        let bat_threshold_alarm = b.get_i16_le();
        let bat_threshold_motors = b.get_i16_le();
        let bat_comp_ref = b.get_i16_le();
        let beeper_mode: BeeperMode = read_enum!(b, "BEEPER_MODE", u8)?;

        let follow_roll_mix_start = b.get_u8();
        let follow_roll_mix_range = b.get_u8();

        for axis_buf in axis_data.iter_mut() {
            axis_buf.put_u8(b.get_u8()); // BOOSTER_POWER
        }

        for axis_buf in axis_data.iter_mut() {
            axis_buf.put_u8(b.get_u8()); // FOLLOW_SPEED
        }

        let frame_angle_from_motors = b.get_u8() != 0;

        for axis_buf in axis_data.iter_mut() {
            axis_buf.put_i16_le(b.get_i16_le()); // RC_MEMORY
        }

        let servo_out = [b.get_u8(), b.get_u8(), b.get_u8(), b.get_u8()];
        let servo_rate = b.get_u8();
        let adaptive_pid_enabled: BitFlags<AdaptivePid> =
            read_flags!(b, "ADAPTIVE_PID_ENABLED", u8)?;
        let adaptive_pid_threshold = b.get_u8();
        let adaptive_pid_rate = b.get_u8();
        let adaptive_pid_recovery_factor = b.get_u8();
        let follow_lpf = [b.get_u8(), b.get_u8(), b.get_u8()];

        let general_flags: BitFlags<GeneralFlags> = read_flags!(b, "ADAPTIVE_PID_ENABLED", u16_le)?;
        let profile_flags: BitFlags<ProfileFlags> = read_flags!(b, "ADAPTIVE_PID_ENABLED", u16_le)?;

        let spektrum_mode: SpektrumMode = read_enum!(b, "SPEKTRUM_MODE", u8)?;

        let order_of_axes: AxisOrder = read_enum!(b, "ORDER_OF_AXES", u8)?;
        let euler_order: EulerOrder = read_enum!(b, "EULER_ORDER", u8)?;

        let cur_imu: ImuType = read_enum!(b, "CUR_IMU", u8)?;
        let cur_profile_id = b.get_u8();

        let [mut axis_data_roll, mut axis_data_yaw, mut axis_data_pitch] = axis_data;

        Ok(Params3Data {
            profile_id,
            axes: (
                AxisParams::from_bytes(axis_data_roll.freeze())?,
                AxisParams::from_bytes(axis_data_yaw.freeze())?,
                AxisParams::from_bytes(axis_data_pitch.freeze())?,
            ),
            acc_limiter_all,
            ext_fc_gain,
            gyro_trust,
            use_model,
            pwm_freq,
            serial_speed,
            rc_deadband,
            rc_expo_rate,
            rc_virt_mode,
            rc_map,
            rc_mix,
            follow_mode,
            follow_deadband,
            follow_expo_rate,
            axis_top,
            axis_right,
            frame_axis_top,
            frame_axis_right,
            frame_imu_pos,
            gyro_deadband,
            gyro_sens,
            i2c_speed_fast,
            skip_gyro_calib,
            // TODO: RC_CMD_LOW..MENU_CMD_LONG
            bat_threshold_alarm,
            bat_threshold_motors,
            bat_comp_ref,
            beeper_mode,
            follow_roll_mix_start,
            follow_roll_mix_range,
            frame_angle_from_motors,
            servo_out,
            servo_rate,
            adaptive_pid_enabled,
            adaptive_pid_threshold,
            adaptive_pid_rate,
            adaptive_pid_recovery_factor,
            general_flags,
            profile_flags,
            spektrum_mode,
            order_of_axes,
            euler_order,
            cur_imu,
            cur_profile_id,
        })
    }

    fn to_bytes(&self) -> Bytes
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
