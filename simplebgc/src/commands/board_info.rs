use crate::{Payload, PayloadParseError};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use enumflags2::BitFlags;

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

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct BoardInfo {
    #[bgc_raw("BOARD_VER")]
    board_version: u8,

    #[bgc_raw("FIRMWARE_VER")]
    firmware_version: u16,

    #[bgc_flags("STATE_FLAGS1")]
    #[repr(u8)]
    state: StateFlags1,

    #[bgc_flags("BOARD_FEATURES")]
    #[repr(u16)]
    board_features: BoardFeatures,

    #[bgc_flags("CONNECTION_FLAG")]
    #[repr(u8)]
    connection_flag: ConnectionFlag,

    #[bgc_raw("FRW_EXTRA_ID")]
    frw_extra_id: u32,

    #[bgc_raw("RESERVED")]
    reserved: [u8; 7],
}
