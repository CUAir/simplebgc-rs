use crate::{Payload, PayloadParseError};

#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum SyncToData {
    ImuAttitude = 1,
}

#[derive(BgcPayload, Copy, Clone, Debug, PartialEq)]
pub struct DataStreamInterval {
    #[kind(raw)]
    pub cmd_id: u8,

    #[kind(raw)]
    pub interval_ms: u16,

    #[kind(raw)]
    pub config: [u8; 8],

    #[kind(enumeration)]
    #[format(u8)]
    pub sync_to_data: SyncToData,

    #[kind(raw)]
    pub reserved: [u8; 9],
}
