use crate::vm::{runtime_class::ClassRef, value::Value};

#[allow(dead_code)]
pub struct Frame {
    pub locals: Vec<Value>,
    pub operand_stack: Vec<Value>,

    pub pc: usize,

    pub class: ClassRef,
    pub method_index: usize,
}

#[allow(dead_code)]
impl Frame {
    pub fn new(max_locals: usize, max_stack: usize, class: ClassRef, method_index: usize) -> Self {
        Self {
            locals: vec![Value::Empty; max_locals],
            operand_stack: Vec::with_capacity(max_stack),
            pc: 0,
            class,
            method_index,
        }
    }

    pub fn push_value(&mut self, v: Value) {
        let is_wide = matches!(v, Value::Long(_) | Value::Double(_));
        if is_wide {
            self.operand_stack.push(Value::Empty);
        }
        self.operand_stack.push(v);
    }

    pub fn pop_value(&mut self) -> Option<Value> {
        let v = self.operand_stack.pop()?;
        if matches!(v, Value::Long(_) | Value::Double(_)) {
            self.operand_stack.pop();
        }
        Some(v)
    }
}
