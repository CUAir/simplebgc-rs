mod read_params_3;

use crate::command::RcVirtMode::Spektrum;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use enumflags2::BitFlags;
use num_traits::FromPrimitive;
use std::convert::{TryFrom, TryInto};

pub struct Version {
    major: u8,
    minor: u8,
    beta: u8,
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum StateFlags1 {
    DebugMode = 1 << 0,
    IsFrameInverted = 1 << 1,
    InitStep1Done = 1 << 2,
    InitStep2Done = 1 << 3,
    StartupAutoRoutineDone = 1 << 4,
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum BoardFeatures {
    ThreeAxis = 1 << 0,
    BatMonitoring = 1 << 1,
    Encoders = 1 << 2,
    BodeTest = 1 << 3,
    Scripting = 1 << 4,
    CurrentSensor = 1 << 5,
}

#[derive(BitFlags, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ConnectionFlag {
    USB = 1 << 0,
}

/// These are what the SimpleBGC spec calls an 'incoming command'.
pub enum Message {
    BoardInfo {
        board_version: Version,
        firmware_version: Version,
        state: StateFlags1,
        connection_flag: ConnectionFlag,
        frw_extra_id: u32,
        reserved: [u8; 7],
    },
    GetAngles {
        /// Imu angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        imu_angle: i32,

        /// Target angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        target_angle: i32,

        /// Target speed that gimbal should keep, over Euler axes
        /// Units: 0,1220740379 degree/sec
        target_speed: i32,
    },
    GetAnglesExt {
        /// Imu angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        imu_angle: i32,

        /// Target angles in 14-bit resolution per full turn
        /// Units: 0,02197265625 degree
        target_angle: i32,

        /// Relative angle for joints between two arms of gimbal structure,
        /// measured by encoder or 2nd Imu. Value 0 corresponds to
        /// normal position of a gimbal. This angle does not overflow after
        /// multiple turns.
        stator_rotor_angle: i64,

        reserved: [u8; 10],
    },
    ReadParams3(ReadParams3Data),
}