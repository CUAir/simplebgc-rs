#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MotorsOffMode {
    /// turn motors off leaving driver in a high impedance
    Normal = 0,
    /// turn motors off leaving driver in a low impedance
    Break,
    /// reduce power and wait while all motors stop rotating, then power
    /// off completely
    SafeStop,
}
