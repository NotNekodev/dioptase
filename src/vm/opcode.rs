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

    IAdd = 0x60,

    IStore1 = 0x3c,

    IReturn = 0xac,

    InvokeStatic = 0xb8,
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

            0x3c => Ok(Self::IStore1),

            0x60 => Ok(Self::IAdd),

            0xac => Ok(Self::IReturn),

            0xb8 => Ok(Self::InvokeStatic),

            _ => Err(()),
        }
    }
}
