use std::collections::HashMap;

use crate::{
    error::RuntimeError,
    vm::{
        opcode::Opcode,
        runtime_class::{ClassRef, RuntimeClass},
        thread::Thread,
        value::Value,
    },
};

pub struct Interpreter;

impl Interpreter {
    pub fn run(
        classes: &[RuntimeClass],
        classes_by_name: &HashMap<String, ClassRef>,
        thread: &mut Thread,
    ) -> Result<Value, RuntimeError> {
        loop {
            let thread_id = thread.id;
            let (frame_class, frame_method_idx) = {
                let f = thread
                    .current_frame()
                    .ok_or(RuntimeError::NoCurrentFrame { thread_id })?;
                (f.class, f.method_index)
            };

            let class = &classes[frame_class.0];
            let method = &class.methods[frame_method_idx];
            let code = &method.code;

            let frame = thread.current_frame().unwrap();
            let op = code[frame.pc];
            frame.pc += 1;

            let opcode =
                Opcode::try_from(op).map_err(|_| RuntimeError::InvalidOpcode { opcode: op })?;

            match opcode {
                Opcode::Bipush => {
                    let value = code[frame.pc] as i8;
                    frame.pc += 1;

                    frame.operand_stack.push(Value::Int(value as i32));
                }

                Opcode::ILoad1 => {
                    let value = frame.locals[1].clone();

                    frame.operand_stack.push(value);
                }

                Opcode::IStore1 => {
                    let value = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    frame.locals[1] = value;
                }

                Opcode::IReturn => {
                    return frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc });
                }
            }
        }
    }
}
