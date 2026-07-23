use crate::{
    error::RuntimeError,
    vm::{thread::Thread, value::Value},
};

pub struct Interpreter;

impl Interpreter {
    pub fn run(thread: &mut Thread, method: &[u8]) -> Result<Value, RuntimeError> {
        loop {
            let thread_id = thread.id;

            let frame = thread
                .current_frame()
                .ok_or(RuntimeError::NoCurrentFrame { thread_id })?;

            let opcode = method[frame.pc];
            frame.pc += 1;

            match opcode {
                0x10 => {
                    let value = method[frame.pc] as i8;
                    frame.pc += 1;

                    frame.operand_stack.push(Value::Int(value as i32));
                }

                0xac => {
                    return frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc });
                }

                _ => {
                    return Err(RuntimeError::InvalidOpcode { opcode: opcode });
                }
            }
        }
    }
}
