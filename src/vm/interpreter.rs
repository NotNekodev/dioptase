use std::collections::HashMap;

use crate::{
    error::RuntimeError,
    vm::{
        frame::Frame,
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

                Opcode::IConst3 => frame.operand_stack.push(Value::Int(3)),
                Opcode::IConst5 => frame.operand_stack.push(Value::Int(5)),

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
                    let ret = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    thread.pop_frame();

                    match thread.current_frame() {
                        Some(caller) => caller.operand_stack.push(ret),
                        None => return Ok(ret),
                    }
                }

                Opcode::InvokeStatic => {
                    let indexbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let indexbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let index: u16 = u16::from_be_bytes([indexbyte1, indexbyte2]);

                    let (target_class_name, method_name, descriptor) =
                        class.constant_pool.get_method_ref(index)?;

                    println!(
                        "InvokeStatic {}.{}{}",
                        target_class_name, method_name, descriptor
                    );

                    let target_class_ref = classes_by_name
                        .get(&target_class_name)
                        .ok_or(RuntimeError::ClassNotFound { index: 0 })?;
                    let target_class = &classes[target_class_ref.0];

                    let target_method_idx = target_class
                        .find_method(method_name.as_str(), descriptor.as_str())
                        .ok_or(RuntimeError::MethodNotFound {
                            class: target_class.name.clone(),
                            index: 0,
                        })?;
                    let target_method = &target_class.methods[target_method_idx];

                    let argument_count = target_method.param_slot_count();
                    let mut args = Vec::with_capacity(argument_count);

                    for _ in 0..argument_count {
                        args.push(
                            frame
                                .operand_stack
                                .pop()
                                .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?,
                        );
                    }
                    args.reverse();

                    let mut new_frame = Frame::new(
                        target_method.max_locals,
                        target_method.max_stack,
                        *target_class_ref,
                        target_method_idx,
                    );

                    for (i, arg) in args.into_iter().enumerate() {
                        new_frame.locals[i] = arg;
                    }
                    thread.push_frame(new_frame);
                }
            }
        }
    }
}
