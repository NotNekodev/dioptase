#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Opcode {
    Bipush = 0x10,

    IConst0 = 0x03,
    IConst1 = 0x04,
    IConst2 = 0x05,
    IConst3 = 0x06,
    IConst4 = 0x07,
    IConst5 = 0x08,

    ILoad0 = 0x1a,
    ILoad1 = 0x1b,
    ILoad2 = 0x1c,
    ILoad3 = 0x1d,

    ALoad0 = 0x2a,
    ALoad1 = 0x2b,
    ALoad2 = 0x2c,
    ALoad3 = 0x2d,

    Dup = 0x59,

    IAdd = 0x60,
    ISub = 0x64,
    IMul = 0x68,

    IStore0 = 0x3b,
    IStore1 = 0x3c,
    IStore2 = 0x3d,
    IStore3 = 0x3e,

    IfICmpEq = 0x9f,
    IfICmpNe = 0xa0,
    IfICmpLt = 0xa1,
    IfICmpGe = 0xa2,
    IfICmpGt = 0xa3,
    IfICmpLe = 0xa4,

    IReturn = 0xac,

    InvokeSpecial = 0xb7,
    InvokeStatic = 0xb8,

    New = 0xbb,
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Self::Bipush),

            0x03 => Ok(Self::IConst0),
            0x04 => Ok(Self::IConst1),
            0x05 => Ok(Self::IConst2),
            0x06 => Ok(Self::IConst3),
            0x07 => Ok(Self::IConst4),
            0x08 => Ok(Self::IConst5),

            0x1a => Ok(Self::ILoad0),
            0x1b => Ok(Self::ILoad1),
            0x1c => Ok(Self::ILoad2),
            0x1d => Ok(Self::ILoad3),

            0x2a => Ok(Self::ALoad0),
            0x2b => Ok(Self::ALoad1),
            0x2c => Ok(Self::ALoad2),
            0x2d => Ok(Self::ALoad3),

            0x3b => Ok(Self::IStore0),
            0x3c => Ok(Self::IStore1),
            0x3d => Ok(Self::IStore2),
            0x3e => Ok(Self::IStore3),

            0x59 => Ok(Self::Dup),

            0x60 => Ok(Self::IAdd),
            0x64 => Ok(Self::ISub),
            0x68 => Ok(Self::IMul),

            0xac => Ok(Self::IReturn),

            0x9f => Ok(Self::IfICmpEq),
            0xa0 => Ok(Self::IfICmpNe),
            0xa1 => Ok(Self::IfICmpLt),
            0xa2 => Ok(Self::IfICmpGe),
            0xa3 => Ok(Self::IfICmpGt),
            0xa4 => Ok(Self::IfICmpLe),

            0xb7 => Ok(Self::InvokeSpecial),
            0xb8 => Ok(Self::InvokeStatic),

            0xbb => Ok(Self::New),

            _ => Err(()),
        }
    }
}
