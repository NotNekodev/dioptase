#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Opcode {
    AConstNull = 0x01,
    Bipush = 0x10,
    Sipush = 0x11,
    Ldc = 0x12,
    LdcW = 0x13,
    Ldc2W = 0x14,

    IConst0 = 0x03,
    IConst1 = 0x04,
    IConst2 = 0x05,
    IConst3 = 0x06,
    IConst4 = 0x07,
    IConst5 = 0x08,

    FConst0 = 0x0b,
    FConst1 = 0x0c,
    FConst2 = 0x0d,

    ILoad = 0x15,
    ILoad0 = 0x1a,
    ILoad1 = 0x1b,
    ILoad2 = 0x1c,
    ILoad3 = 0x1d,

    FLoad0 = 0x22,
    FLoad1 = 0x23,
    FLoad2 = 0x24,
    FLoad3 = 0x25,

    ALoad0 = 0x2a,
    ALoad1 = 0x2b,
    ALoad2 = 0x2c,
    ALoad3 = 0x2d,

    IALoad = 0x2e,
    CALoad = 0x34,

    AStore = 0x3a,
    AStore0 = 0x4b,
    AStore1 = 0x4c,
    AStore2 = 0x4d,
    AStore3 = 0x4e,

    IAStore = 0x4f,
    AAStore = 0x53,
    CAStore = 0x55,

    Pop = 0x57,
    Dup = 0x59,

    IAdd = 0x60,
    LAdd = 0x61,
    ISub = 0x64,
    IMul = 0x68,
    FMul = 0x6a,

    IRem = 0x70,
    LShl = 0x79,
    IAnd = 0x7e,
    LAnd = 0x7f,

    IInc = 0x84,

    I2L = 0x85,
    I2F = 0x86,
    F2I = 0x8b,

    AALoad = 0x32,

    IStore = 0x36,
    IStore0 = 0x3b,
    IStore1 = 0x3c,
    IStore2 = 0x3d,
    IStore3 = 0x3e,

    FCmpL = 0x95,
    FCmpG = 0x96,

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
    FReturn = 0xae,
    DReturn = 0xaf,
    AReturn = 0xb0,
    Return = 0xb1,
    GetStatic = 0xb2,
    PutStatic = 0xb3,
    GetField = 0xb4,
    PutField = 0xb5,

    InvokeVirtual = 0xb6,
    InvokeSpecial = 0xb7,
    InvokeStatic = 0xb8,
    InvokeInterface = 0xb9,

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
            0x12 => Ok(Self::Ldc),
            0x13 => Ok(Self::LdcW),
            0x14 => Ok(Self::Ldc2W),

            0x03 => Ok(Self::IConst0),
            0x04 => Ok(Self::IConst1),
            0x05 => Ok(Self::IConst2),
            0x06 => Ok(Self::IConst3),
            0x07 => Ok(Self::IConst4),
            0x08 => Ok(Self::IConst5),

            0x0b => Ok(Self::FConst0),
            0x0c => Ok(Self::FConst1),
            0x0d => Ok(Self::FConst2),

            0x15 => Ok(Self::ILoad),
            0x1a => Ok(Self::ILoad0),
            0x1b => Ok(Self::ILoad1),
            0x1c => Ok(Self::ILoad2),
            0x1d => Ok(Self::ILoad3),

            0x22 => Ok(Self::FLoad0),
            0x23 => Ok(Self::FLoad1),
            0x24 => Ok(Self::FLoad2),
            0x25 => Ok(Self::FLoad3),

            0x2a => Ok(Self::ALoad0),
            0x2b => Ok(Self::ALoad1),
            0x2c => Ok(Self::ALoad2),
            0x2d => Ok(Self::ALoad3),

            0x2e => Ok(Self::IALoad),

            0x32 => Ok(Self::AALoad),
            0x34 => Ok(Self::CALoad),

            0x36 => Ok(Self::IStore),
            0x3b => Ok(Self::IStore0),
            0x3c => Ok(Self::IStore1),
            0x3d => Ok(Self::IStore2),
            0x3e => Ok(Self::IStore3),

            0x3a => Ok(Self::AStore),
            0x4b => Ok(Self::AStore0),
            0x4c => Ok(Self::AStore1),
            0x4d => Ok(Self::AStore2),
            0x4e => Ok(Self::AStore3),

            0x4f => Ok(Self::IAStore),
            0x53 => Ok(Self::AAStore),
            0x55 => Ok(Self::CAStore),

            0x57 => Ok(Self::Pop),
            0x59 => Ok(Self::Dup),

            0x60 => Ok(Self::IAdd),
            0x61 => Ok(Self::LAdd),
            0x64 => Ok(Self::ISub),
            0x68 => Ok(Self::IMul),
            0x6a => Ok(Self::FMul),

            0x70 => Ok(Self::IRem),
            0x79 => Ok(Self::LShl),
            0x7e => Ok(Self::IAnd),
            0x7f => Ok(Self::LAnd),

            0x84 => Ok(Self::IInc),

            0x85 => Ok(Self::I2L),
            0x86 => Ok(Self::I2F),
            0x8b => Ok(Self::F2I),

            0x95 => Ok(Self::FCmpL),
            0x96 => Ok(Self::FCmpG),

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
            0xae => Ok(Self::FReturn),
            0xaf => Ok(Self::DReturn),
            0xb0 => Ok(Self::AReturn),
            0xb1 => Ok(Self::Return),
            0xb2 => Ok(Self::GetStatic),
            0xb3 => Ok(Self::PutStatic),
            0xb4 => Ok(Self::GetField),
            0xb5 => Ok(Self::PutField),

            0xb6 => Ok(Self::InvokeVirtual),
            0xb7 => Ok(Self::InvokeSpecial),
            0xb8 => Ok(Self::InvokeStatic),
            0xb9 => Ok(Self::InvokeInterface),

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
