use crate::vm::value::Value;

#[allow(dead_code)]
pub struct Frame {
    pub locals: Vec<Value>,
    pub operand_stack: Vec<Value>,

    pub pc: usize,

    pub method_index: usize,
}

#[allow(dead_code)]
impl Frame {
    pub fn new(max_locals: usize, max_stack: usize, method_index: usize) -> Self {
        Self {
            locals: vec![Value::Empty; max_locals],
            operand_stack: Vec::with_capacity(max_stack),
            pc: 0,
            method_index,
        }
    }
}
