use crate::{
    error::RuntimeError,
    vm::{opcode::Opcode, thread::Thread, value::Value},
};

pub struct Interpreter;

impl Interpreter {
    pub fn run(thread: &mut Thread, method: &[u8]) -> Result<Value, RuntimeError> {
        loop {
            let thread_id = thread.id;

            let frame = thread
                .current_frame()
                .ok_or(RuntimeError::NoCurrentFrame { thread_id })?;

            let opcode_byte = method[frame.pc];
            frame.pc += 1;

            let opcode =
                Opcode::try_from(opcode_byte).map_err(|_| RuntimeError::InvalidOpcode {
                    opcode: opcode_byte,
                })?;

            match opcode {
                Opcode::Bipush => {
                    let value = method[frame.pc] as i8;
                    frame.pc += 1;

                    frame.operand_stack.push(Value::Int(value as i32));
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
