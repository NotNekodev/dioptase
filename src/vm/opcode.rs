#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Opcode {
    AConstNull = 0x01,
    Bipush = 0x10,
    Sipush = 0x11,

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

    IALoad = 0x2e,

    AStore0 = 0x4b,
    AStore1 = 0x4c,
    AStore2 = 0x4d,
    AStore3 = 0x4e,

    IAStore = 0x4f,

    AAStore = 0x53,
    Pop = 0x57,
    Dup = 0x59,

    IAdd = 0x60,
    ISub = 0x64,
    IMul = 0x68,

    IInc = 0x86,

    AALoad = 0x32,

    IStore0 = 0x3b,
    IStore1 = 0x3c,
    IStore2 = 0x3d,
    IStore3 = 0x3e,

    IfEq = 0x99,
    IfNe = 0x9a,
    IfLt = 0x9b,
    IfGe = 0x9c,
    IfGt = 0x9d,
    IfLe = 0x9e,

    IfICmpEq = 0x9f,
    IfICmpNe = 0xa0,
    IfICmpLt = 0xa1,
    IfICmpGe = 0xa2,
    IfICmpGt = 0xa3,
    IfICmpLe = 0xa4,

    Goto = 0xa7,
    IReturn = 0xac,
    Return = 0xb1,

    GetField = 0xb4,
    PutField = 0xb5,

    InvokeSpecial = 0xb7,
    InvokeStatic = 0xb8,

    New = 0xbb,
    NewArray = 0xbc,
    ANewArray = 0xbd,
    ArrayLength = 0xbe,

    IfNull = 0xc6,
    IfNonNull = 0xc7,
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::AConstNull),

            0x10 => Ok(Self::Bipush),
            0x11 => Ok(Self::Sipush),

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

            0x2e => Ok(Self::IALoad),

            0x32 => Ok(Self::AALoad),

            0x3b => Ok(Self::IStore0),
            0x3c => Ok(Self::IStore1),
            0x3d => Ok(Self::IStore2),
            0x3e => Ok(Self::IStore3),

            0x4b => Ok(Self::AStore0),
            0x4c => Ok(Self::AStore1),
            0x4d => Ok(Self::AStore2),
            0x4e => Ok(Self::AStore3),
            0x4f => Ok(Self::IAStore),

            0x53 => Ok(Self::AAStore),
            0x57 => Ok(Self::Pop),
            0x59 => Ok(Self::Dup),

            0x60 => Ok(Self::IAdd),
            0x64 => Ok(Self::ISub),
            0x68 => Ok(Self::IMul),

            0x84 => Ok(Self::IInc),

            0x99 => Ok(Self::IfEq),
            0x9a => Ok(Self::IfNe),
            0x9b => Ok(Self::IfLt),
            0x9c => Ok(Self::IfGe),
            0x9d => Ok(Self::IfGt),
            0x9e => Ok(Self::IfLe),

            0x9f => Ok(Self::IfICmpEq),
            0xa0 => Ok(Self::IfICmpNe),
            0xa1 => Ok(Self::IfICmpLt),
            0xa2 => Ok(Self::IfICmpGe),
            0xa3 => Ok(Self::IfICmpGt),
            0xa4 => Ok(Self::IfICmpLe),

            0xa7 => Ok(Self::Goto),
            0xac => Ok(Self::IReturn),
            0xb1 => Ok(Self::Return),

            0xb4 => Ok(Self::GetField),
            0xb5 => Ok(Self::PutField),

            0xb7 => Ok(Self::InvokeSpecial),
            0xb8 => Ok(Self::InvokeStatic),

            0xbb => Ok(Self::New),
            0xbc => Ok(Self::NewArray),
            0xbd => Ok(Self::ANewArray),
            0xbe => Ok(Self::ArrayLength),

            0xc6 => Ok(Self::IfNull),
            0xc7 => Ok(Self::IfNonNull),

            _ => Err(()),
        }
    }
}
