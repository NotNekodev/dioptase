use std::rc::Rc;

use crate::{
    error::RuntimeError,
    vm::{frame::Frame, opcode::Opcode, thread::ThreadRef, value::Value, vm::VM},
};

pub struct Interpreter;

impl Interpreter {
    pub fn run(vm: &mut VM, thread_ref: ThreadRef) -> Result<Value, RuntimeError> {
        loop {
            let (frame_class, frame_method_idx) = {
                let thread = vm.get_thread(thread_ref)?;
                let f = thread.current_frame().ok_or(RuntimeError::NoCurrentFrame {
                    thread_id: thread_ref.0,
                })?;
                (f.class, f.method_index)
            };

            let code: Rc<[u8]> = vm.get_method(frame_class, frame_method_idx)?.code.clone();

            let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
            let op = code[frame.pc];
            frame.pc += 1;
            let opcode = Opcode::try_from(op).map_err(|_| RuntimeError::InvalidOpcode {
                opcode: op,
                pc: frame.pc - 1,
            })?;

            match opcode {
                Opcode::Bipush => {
                    let value = code[frame.pc] as i8;
                    frame.pc += 1;

                    frame.operand_stack.push(Value::Int(value as i32));
                }

                Opcode::IConst0 => frame.operand_stack.push(Value::Int(0)),
                Opcode::IConst1 => frame.operand_stack.push(Value::Int(1)),
                Opcode::IConst2 => frame.operand_stack.push(Value::Int(2)),
                Opcode::IConst3 => frame.operand_stack.push(Value::Int(3)),
                Opcode::IConst4 => frame.operand_stack.push(Value::Int(4)),
                Opcode::IConst5 => frame.operand_stack.push(Value::Int(5)),

                Opcode::ILoad0 => frame.operand_stack.push(frame.locals[0].clone()),
                Opcode::ILoad1 => frame.operand_stack.push(frame.locals[1].clone()),
                Opcode::ILoad2 => frame.operand_stack.push(frame.locals[2].clone()),
                Opcode::ILoad3 => frame.operand_stack.push(frame.locals[3].clone()),

                Opcode::ALoad0 => frame.operand_stack.push(frame.locals[0].clone()),
                Opcode::ALoad1 => frame.operand_stack.push(frame.locals[1].clone()),
                Opcode::ALoad2 => frame.operand_stack.push(frame.locals[2].clone()),
                Opcode::ALoad3 => frame.operand_stack.push(frame.locals[3].clone()),

                Opcode::AStore0 => {
                    frame.locals[0] = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                }
                Opcode::AStore1 => {
                    frame.locals[1] = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                }
                Opcode::AStore2 => {
                    frame.locals[2] = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                }
                Opcode::AStore3 => {
                    frame.locals[3] = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                }

                Opcode::IStore0 => {
                    frame.locals[0] = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?
                }
                Opcode::IStore1 => {
                    frame.locals[1] = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?
                }
                Opcode::IStore2 => {
                    frame.locals[2] = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?
                }
                Opcode::IStore3 => {
                    frame.locals[3] = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?
                }

                Opcode::Dup => {
                    let top = frame
                        .operand_stack
                        .last()
                        .cloned()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    frame.operand_stack.push(top);
                }

                Opcode::IAdd => {
                    let value2 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    let value1 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match (value1, value2) {
                        (Value::Int(x), Value::Int(y)) => {
                            frame.operand_stack.push(Value::Int(x.wrapping_add(y)));
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::ISub => {
                    let value2 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    let value1 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match (value1, value2) {
                        (Value::Int(x), Value::Int(y)) => {
                            frame.operand_stack.push(Value::Int(x.wrapping_sub(y)));
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IMul => {
                    let value2 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    let value1 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match (value1, value2) {
                        (Value::Int(x), Value::Int(y)) => {
                            frame.operand_stack.push(Value::Int(x.wrapping_mul(y)));
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IReturn => {
                    let ret = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    vm.get_thread(thread_ref)?.pop_frame();

                    match vm.get_thread(thread_ref)?.current_frame() {
                        Some(caller) => caller.operand_stack.push(ret),
                        None => return Ok(ret),
                    }
                }

                Opcode::Return => {
                    vm.get_thread(thread_ref)?.pop_frame();
                    if vm.get_thread(thread_ref)?.current_frame().is_none() {
                        return Ok(Value::Empty);
                    }
                }

                Opcode::IfICmpEq => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value2 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    let value1 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match (value1, value2) {
                        (Value::Int(value1), Value::Int(value2)) => {
                            if value1 == value2 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfICmpNe => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value2 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    let value1 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match (value1, value2) {
                        (Value::Int(value1), Value::Int(value2)) => {
                            if value1 != value2 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfICmpLt => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value2 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    let value1 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match (value1, value2) {
                        (Value::Int(value1), Value::Int(value2)) => {
                            if value1 < value2 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfICmpLe => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value2 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    let value1 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match (value1, value2) {
                        (Value::Int(value1), Value::Int(value2)) => {
                            if value1 <= value2 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfICmpGt => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value2 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    let value1 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match (value1, value2) {
                        (Value::Int(value1), Value::Int(value2)) => {
                            if value1 > value2 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfICmpGe => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value2 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    let value1 = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match (value1, value2) {
                        (Value::Int(value1), Value::Int(value2)) => {
                            if value1 >= value2 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::GetField => {
                    let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                    frame.pc += 2;

                    let objectref = match frame.operand_stack.pop() {
                        Some(Value::Reference(Some(r))) => r,
                        _ => return Err(RuntimeError::InvalidType),
                    };

                    let (owner_name, field_name, _descriptor) = vm
                        .get_class(frame_class)?
                        .constant_pool
                        .get_field_ref(index)?;
                    let owner_ref = vm.resolve_class(&owner_name)?;
                    let slot = vm
                        .get_class(owner_ref)?
                        .find_field(&field_name)
                        .ok_or(RuntimeError::InvalidConstantPoolEntry)?
                        .slot;

                    let value = vm.heap().get(objectref).fields[slot].clone();

                    let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                    frame.operand_stack.push(value);
                }

                Opcode::PutField => {
                    let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                    frame.pc += 2;

                    let value = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    let objectref = match frame.operand_stack.pop() {
                        Some(Value::Reference(Some(r))) => r,
                        _ => return Err(RuntimeError::InvalidType),
                    };

                    let (owner_name, field_name, _descriptor) = vm
                        .get_class(frame_class)?
                        .constant_pool
                        .get_field_ref(index)?;
                    let owner_ref = vm.resolve_class(&owner_name)?;
                    let slot = vm
                        .get_class(owner_ref)?
                        .find_field(&field_name)
                        .ok_or(RuntimeError::InvalidConstantPoolEntry)?
                        .slot;

                    vm.heap_mut().get_mut(objectref).fields[slot] = value;
                }

                Opcode::InvokeSpecial => {
                    let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                    frame.pc += 2;

                    let (target_class_name, method_name, descriptor) = vm
                        .get_class(frame_class)?
                        .constant_pool
                        .get_method_ref(index)?;

                    let target_class_ref = vm.resolve_class(&target_class_name)?;

                    let (target_method_idx, max_locals, max_stack, param_slots) = {
                        let target_class = vm.get_class(target_class_ref)?;

                        let idx = target_class.find_method(&method_name, &descriptor).ok_or(
                            RuntimeError::MethodNotFound {
                                class: target_class.name.clone(),
                                method: method_name.clone(),
                            },
                        )?;

                        let m = &target_class.methods[idx];
                        (idx, m.max_locals, m.max_stack, m.param_slot_count())
                    };

                    let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                    let mut args = Vec::with_capacity(param_slots + 1);

                    for _ in 0..param_slots {
                        args.push(
                            frame
                                .operand_stack
                                .pop()
                                .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?,
                        );
                    }

                    let objectref = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;
                    args.push(objectref);
                    args.reverse();

                    let mut new_frame =
                        Frame::new(max_locals, max_stack, target_class_ref, target_method_idx);

                    for (i, v) in args.into_iter().enumerate() {
                        new_frame.locals[i] = v;
                    }

                    vm.get_thread(thread_ref)?.push_frame(new_frame);
                }

                Opcode::InvokeStatic => {
                    let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                    frame.pc += 2;

                    let (target_class_name, method_name, descriptor) = vm
                        .get_class(frame_class)?
                        .constant_pool
                        .get_method_ref(index)?;

                    let target_class_ref = vm.resolve_class(&target_class_name)?;

                    let (target_method_idx, max_locals, max_stack, argument_count) = {
                        let target_class = vm.get_class(target_class_ref)?;
                        let idx = target_class.find_method(&method_name, &descriptor).ok_or(
                            RuntimeError::MethodNotFound {
                                class: target_class.name.clone(),
                                method: method_name.clone(),
                            },
                        )?;
                        let m = &target_class.methods[idx];
                        (idx, m.max_locals, m.max_stack, m.param_slot_count())
                    };

                    let frame = vm.get_thread(thread_ref)?.current_frame().unwrap(); // fresh borrow
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

                    let mut new_frame =
                        Frame::new(max_locals, max_stack, target_class_ref, target_method_idx);
                    for (i, arg) in args.into_iter().enumerate() {
                        new_frame.locals[i] = arg;
                    }
                    vm.get_thread(thread_ref)?.push_frame(new_frame);
                }

                Opcode::New => {
                    let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                    frame.pc += 2;

                    let class_name = vm
                        .get_class(frame_class)?
                        .constant_pool
                        .get_class_name(index)?;
                    let target_class_ref = vm.resolve_class(&class_name)?;
                    let slot_count = vm.get_class(target_class_ref)?.instance_slot_count();
                    let obj_ref = vm.heap_mut().allocate(target_class_ref, slot_count);

                    let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                    frame.operand_stack.push(Value::Reference(Some(obj_ref)));
                }

                Opcode::IInc => {
                    let index = code[frame.pc];
                    let amount = code[frame.pc + 1] as i8;
                    frame.pc += 2;

                    let local_var: &mut Value =
                        frame
                            .locals
                            .get_mut(index as usize)
                            .ok_or(RuntimeError::NoLocalVar {
                                index: index as usize,
                            })?;

                    match local_var {
                        Value::Int(val) => {
                            *val = val.wrapping_add(amount as i32);
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::Goto => {
                    let opcode_pc = frame.pc - 1;
                    let offset = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                    frame.pc += 2;

                    frame.pc = (opcode_pc as isize + offset as isize) as usize;
                }

                Opcode::IfEq => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match value {
                        Value::Int(val) => {
                            if val == 0 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfNe => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match value {
                        Value::Int(val) => {
                            if val != 0 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfLt => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match value {
                        Value::Int(val) => {
                            if val < 0 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfLe => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match value {
                        Value::Int(val) => {
                            if val <= 0 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfGt => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match value {
                        Value::Int(val) => {
                            if val > 0 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfGe => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let value = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match value {
                        Value::Int(val) => {
                            if val >= 0 {
                                frame.pc = branch_ip as usize + opcode_pc
                            }
                        }
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::AConstNull => {
                    frame.operand_stack.push(Value::Reference(None));
                }

                Opcode::IfNonNull => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let reference_value = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match reference_value {
                        Value::Reference(reference) => match reference {
                            Some(_) => {
                                frame.pc = branch_ip as usize + opcode_pc;
                            }

                            None => {}
                        },
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }

                Opcode::IfNull => {
                    let opcode_pc = frame.pc - 1;
                    let branchbyte1 = code[frame.pc];
                    frame.pc += 1;
                    let branchbyte2 = code[frame.pc];
                    frame.pc += 1;
                    let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                    let reference_value = frame
                        .operand_stack
                        .pop()
                        .ok_or(RuntimeError::OperandStackUnderflow { pc: frame.pc })?;

                    match reference_value {
                        Value::Reference(reference) => match reference {
                            Some(_) => {}

                            None => {
                                frame.pc = branch_ip as usize + opcode_pc;
                            }
                        },
                        _ => return Err(RuntimeError::InvalidType),
                    }
                }
            }
        }
    }
}
