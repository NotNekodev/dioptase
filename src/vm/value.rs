#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectRef(pub usize);

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),

    Reference(Option<ObjectRef>),
    ReturnAddress(usize),

    Empty,
}
