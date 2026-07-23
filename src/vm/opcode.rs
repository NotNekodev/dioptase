#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Opcode {
    Bipush = 0x10,

    ILoad1 = 0x1b,

    IStore1 = 0x3c,

    IReturn = 0xac,
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Self::Bipush),

            0x1b => Ok(Self::ILoad1),

            0x3c => Ok(Self::IStore1),

            0xac => Ok(Self::IReturn),

            _ => Err(()),
        }
    }
}
