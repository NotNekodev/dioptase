use crate::{
    error::{InternalError, RuntimeError},
    vm::{
        frame::Frame,
        heap::ArrayElementType,
        opcode::Opcode,
        runtime_method::MethodBody,
        thread::ThreadRef,
        value::{ObjectRef, Value},
        vm::VM,
    },
};

pub enum StepOutcome {
    Continue,
    Return(Value),
}

pub struct Interpreter;

impl Interpreter {
    pub fn run(vm: &mut VM, thread_ref: ThreadRef) -> Result<Value, RuntimeError> {
        loop {
            match Self::step(vm, thread_ref) {
                Ok(StepOutcome::Continue) => continue,
                Ok(StepOutcome::Return(v)) => return Ok(v),
                Err(RuntimeError::Internal(e)) => return Err(RuntimeError::Internal(e)), // fatal, no catch
                Err(RuntimeError::Thrown(obj_ref)) => {
                    if Self::unwind_to_handler(vm, thread_ref, obj_ref)? {
                        continue;
                    } else {
                        return Err(RuntimeError::Thrown(obj_ref));
                    }
                }
            }
        }
    }

    fn unwind_to_handler(
        vm: &mut VM,
        thread_ref: ThreadRef,
        obj_ref: ObjectRef,
    ) -> Result<bool, RuntimeError> {
        let obj_class = vm.heap().get_object(obj_ref)?.class;

        loop {
            let (frame_class, frame_method_idx, pc) = {
                let thread = vm.get_thread(thread_ref)?;
                match thread.current_frame() {
                    Some(f) => (f.class, f.method_index, f.pc),
                    None => return Ok(false),
                }
            };

            let handlers = vm
                .get_method(frame_class, frame_method_idx)?
                .exception_handlers
                .clone();

            for h in &handlers {
                let in_range = (h.start_pc as usize) <= pc && pc < (h.end_pc as usize);
                if !in_range {
                    continue;
                }
                let matches = match &h.catch_class {
                    None => true,
                    Some(name) => {
                        let catch_ref = vm.resolve_class(name)?;
                        vm.is_assignable(obj_class, catch_ref)?
                    }
                };
                if matches {
                    let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                    frame.pc = h.handler_pc as usize;
                    frame.operand_stack.clear();
                    frame.operand_stack.push(Value::Reference(Some(obj_ref)));
                    return Ok(true);
                }
            }

            vm.get_thread(thread_ref)?.pop_frame();
        }
    }

    fn step(vm: &mut VM, thread_ref: ThreadRef) -> Result<StepOutcome, RuntimeError> {
        let (frame_class, frame_method_idx) = {
            let thread = vm.get_thread(thread_ref)?;
            let f = thread
                .current_frame()
                .ok_or(InternalError::NoCurrentFrame {
                    thread_id: thread_ref.0,
                })?;
            (f.class, f.method_index)
        };
        let body = vm.get_method(frame_class, frame_method_idx)?.body.clone();

        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
        match body {
            MethodBody::Bytecode(code) => {
                let op = code[frame.pc];
                frame.pc += 1;
                let opcode = Opcode::try_from(op).map_err(|_| InternalError::InvalidOpcode {
                    opcode: op,
                    pc: frame.pc - 1,
                })?;

                match opcode {
                    Opcode::Bipush => {
                        let value = code[frame.pc] as i8;
                        frame.pc += 1;

                        frame.operand_stack.push(Value::Int(value as i32));
                    }

                    Opcode::Sipush => {
                        let byte1 = code[frame.pc];
                        frame.pc += 1;
                        let byte2 = code[frame.pc];
                        frame.pc += 1;
                        let value: i16 = i16::from_be_bytes([byte1, byte2]);

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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }
                    Opcode::AStore1 => {
                        frame.locals[1] = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }
                    Opcode::AStore2 => {
                        frame.locals[2] = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }
                    Opcode::AStore3 => {
                        frame.locals[3] = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }

                    Opcode::IStore0 => {
                        frame.locals[0] = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?
                    }
                    Opcode::IStore1 => {
                        frame.locals[1] = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?
                    }
                    Opcode::IStore2 => {
                        frame.locals[2] = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?
                    }
                    Opcode::IStore3 => {
                        frame.locals[3] = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?
                    }

                    Opcode::Dup => {
                        let top = frame
                            .operand_stack
                            .last()
                            .cloned()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        frame.operand_stack.push(top);
                    }

                    Opcode::IAdd => {
                        let value2 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(x), Value::Int(y)) => {
                                frame.operand_stack.push(Value::Int(x.wrapping_add(y)));
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        }
                    }

                    Opcode::ISub => {
                        let value2 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(x), Value::Int(y)) => {
                                frame.operand_stack.push(Value::Int(x.wrapping_sub(y)));
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        }
                    }

                    Opcode::IMul => {
                        let value2 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(x), Value::Int(y)) => {
                                frame.operand_stack.push(Value::Int(x.wrapping_mul(y)));
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        }
                    }

                    Opcode::IReturn => {
                        let ret = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        vm.get_thread(thread_ref)?.pop_frame();

                        match vm.get_thread(thread_ref)?.current_frame() {
                            Some(caller) => caller.operand_stack.push(ret),
                            None => return Ok(StepOutcome::Return(ret)),
                        }
                    }

                    Opcode::Return => {
                        vm.get_thread(thread_ref)?.pop_frame();
                        if vm.get_thread(thread_ref)?.current_frame().is_none() {
                            return Ok(StepOutcome::Return(Value::Empty));
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 == value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 != value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 < value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 <= value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 > value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 >= value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        }
                    }

                    Opcode::GetField => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let objectref = match frame.operand_stack.pop() {
                            Some(Value::Reference(Some(r))) => r,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let (owner_name, field_name, _descriptor) = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_field_ref(index)?;
                        let owner_ref = vm.resolve_class(&owner_name)?;
                        let slot = vm
                            .get_class(owner_ref)?
                            .find_field(&field_name)
                            .ok_or(InternalError::InvalidConstantPoolEntry)?
                            .slot;

                        let value = vm.heap().get_object(objectref)?.fields[slot].clone();

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.operand_stack.push(value);
                    }

                    Opcode::PutField => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let value = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let objectref = match frame.operand_stack.pop() {
                            Some(Value::Reference(Some(r))) => r,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let (owner_name, field_name, _descriptor) = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_field_ref(index)?;
                        let owner_ref = vm.resolve_class(&owner_name)?;
                        let slot = vm
                            .get_class(owner_ref)?
                            .find_field(&field_name)
                            .ok_or(InternalError::InvalidConstantPoolEntry)?
                            .slot;

                        vm.heap_mut().get_object_mut(objectref)?.fields[slot] = value;
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
                                InternalError::MethodNotFound {
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
                                    .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?,
                            );
                        }

                        let objectref = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
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
                                InternalError::MethodNotFound {
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
                                    .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?,
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
                        let obj_ref = vm.heap_mut().allocate_object(target_class_ref, slot_count);

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.operand_stack.push(Value::Reference(Some(obj_ref)));
                    }

                    Opcode::IInc => {
                        let index = code[frame.pc];
                        let amount = code[frame.pc + 1] as i8;
                        frame.pc += 2;

                        let local_var: &mut Value = frame.locals.get_mut(index as usize).ok_or(
                            InternalError::NoLocalVar {
                                index: index as usize,
                            },
                        )?;

                        match local_var {
                            Value::Int(val) => {
                                *val = val.wrapping_add(amount as i32);
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val == 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val != 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val < 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val <= 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val > 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val >= 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match reference_value {
                            Value::Reference(reference) => match reference {
                                Some(_) => {
                                    frame.pc = branch_ip as usize + opcode_pc;
                                }

                                None => {}
                            },
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
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
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match reference_value {
                            Value::Reference(reference) => match reference {
                                Some(_) => {}

                                None => {
                                    frame.pc = branch_ip as usize + opcode_pc;
                                }
                            },
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        }
                    }

                    Opcode::Pop => {
                        let _ = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }

                    Opcode::NewArray => {
                        let atype = code[frame.pc];
                        frame.pc += 1;

                        let length = match frame.operand_stack.pop() {
                            Some(Value::Int(n)) if n >= 0 => n as usize,
                            Some(Value::Int(_)) => {
                                return Err(vm.throw(
                                    "java/lang/NegativeArraySizeException",
                                    Some("Cannot create negative sized array"),
                                ));
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let array_type = ArrayElementType::try_from(atype)
                            .map_err(|_| InternalError::InvalidArrayType { atype })?;

                        let array_ref = vm.heap_mut().allocate_array(array_type, length);

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.operand_stack.push(Value::Reference(Some(array_ref)));
                    }

                    Opcode::ANewArray => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let length = match frame.operand_stack.pop() {
                            Some(Value::Int(n)) if n >= 0 => n as usize,
                            Some(Value::Int(_)) => {
                                return Err(vm.throw(
                                    "java/lang/NegativeArraySizeException",
                                    Some("Cannot create negative sized array"),
                                ));
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let class_name = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_class_name(index)?;

                        let component = vm.resolve_class(&class_name)?;

                        let array_ref = vm
                            .heap_mut()
                            .allocate_array(ArrayElementType::Reference(component), length);

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.operand_stack.push(Value::Reference(Some(array_ref)));
                    }

                    Opcode::IAStore => {
                        let value = match frame.operand_stack.pop() {
                            Some(Value::Int(n)) => n as i32,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let index = match frame.operand_stack.pop() {
                            Some(Value::Int(n)) => n as usize,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let arrayref = match frame.operand_stack.pop() {
                            Some(Value::Reference(aref)) => aref,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let reference = match arrayref {
                            Some(value) => value,
                            None => {
                                return Err(vm.throw(
                                    "java/lang/NullPointerException",
                                    Some(&format!(
                                        "Tried to access index {} on a `null` array",
                                        index
                                    )),
                                ));
                            }
                        };

                        {
                            let len = {
                                let array = vm.heap_mut().get_array_mut(reference)?;
                                array.elements.len()
                            };

                            if index >= len {
                                return Err(vm.throw(
                                    "java/lang/ArrayIndexOutOfBoundsException",
                                    Some(&format!(
                                        "Index {} out of bounds for length {}",
                                        index, len
                                    )),
                                ));
                            }
                        }

                        let array = vm.heap_mut().get_array_mut(reference)?;
                        array.elements[index] = Value::Int(value);
                    }

                    Opcode::IALoad => {
                        let index = match frame.operand_stack.pop() {
                            Some(Value::Int(n)) => n as usize,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let arrayref = match frame.operand_stack.pop() {
                            Some(Value::Reference(aref)) => aref,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let reference = match arrayref {
                            Some(value) => value,
                            None => {
                                return Err(vm.throw(
                                    "java/lang/NullPointerException",
                                    Some(&format!(
                                        "Tried to access index {} on a `null` array",
                                        index
                                    )),
                                ));
                            }
                        };

                        let value = {
                            let result = {
                                let array = vm.heap_mut().get_array_mut(reference)?;

                                if let Some(v) = array.elements.get(index) {
                                    Ok(v.clone())
                                } else {
                                    Err(array.elements.len())
                                }
                            };

                            match result {
                                Ok(v) => v,
                                Err(len) => {
                                    return Err(vm.throw(
                                        "java/lang/ArrayIndexOutOfBoundsException",
                                        Some(&format!(
                                            "Index {} out of bounds for length {}",
                                            index, len
                                        )),
                                    ));
                                }
                            }
                        };

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.operand_stack.push(value);
                    }

                    Opcode::ArrayLength => {
                        let arrayref = match frame.operand_stack.pop() {
                            Some(Value::Reference(aref)) => aref,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let reference = match arrayref {
                            Some(value) => value,
                            None => {
                                return Err(vm.throw(
                                    "java/lang/NullPointerException",
                                    Some("Tried to get the length on a null array"),
                                ));
                            }
                        };

                        let array = vm.heap_mut().get_array_mut(reference)?;
                        let length = array.elements.len();

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.operand_stack.push(Value::Int(length as i32));
                    }

                    Opcode::AAStore => {
                        let value = match frame.operand_stack.pop() {
                            Some(Value::Reference(aref)) => Value::Reference(aref),
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let index = match frame.operand_stack.pop() {
                            Some(Value::Int(aref)) => aref,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let arrayref = match frame.operand_stack.pop() {
                            Some(Value::Reference(aref)) => aref,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let reference = match arrayref {
                            Some(value) => value,
                            None => {
                                return Err(vm.throw(
                                    "java/lang/NullPointerException",
                                    Some(&format!(
                                        "Tried to access index {} on a `null` array",
                                        index
                                    )),
                                ));
                            }
                        };

                        let component = {
                            let array = vm.heap().get_array(reference)?;

                            match array.element_type {
                                ArrayElementType::Reference(component) => component,
                                _ => {
                                    return Err(RuntimeError::Internal(InternalError::InvalidType));
                                }
                            }
                        };

                        match value {
                            Value::Reference(None) => {}

                            Value::Reference(Some(obj)) => {
                                let obj_class = vm.heap().get_object(obj)?.class;

                                let obj_class_name = vm.get_class(obj_class)?.name.clone();
                                let component_class_name = vm.get_class(component)?.name.clone();

                                if !vm.is_assignable(obj_class, component)? {
                                    return Err(vm.throw(
                                        "java/lang/ArrayStoreException",
                                        Some(&format!(
                                            "Cannot assign {} to {}",
                                            obj_class_name, component_class_name
                                        )),
                                    ));
                                }
                            }

                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        }

                        {
                            let len = {
                                let array = vm.heap_mut().get_array_mut(reference)?;
                                array.elements.len()
                            };

                            if index as usize >= len {
                                return Err(vm.throw(
                                    "java/lang/ArrayIndexOutOfBoundsException",
                                    Some(&format!(
                                        "Index {} out of bounds for length {}",
                                        index, len
                                    )),
                                ));
                            }
                        }

                        vm.heap_mut().get_array_mut(reference)?.elements[index as usize] = value;
                    }

                    Opcode::AALoad => {
                        let index = match frame.operand_stack.pop() {
                            Some(Value::Int(aref)) => aref,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let arrayref = match frame.operand_stack.pop() {
                            Some(Value::Reference(aref)) => aref,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let reference = match arrayref {
                            Some(value) => value,
                            None => {
                                return Err(vm.throw(
                                    "java/lang/NullPointerException",
                                    Some(&format!(
                                        "Tried to access index {} on a `null` array",
                                        index
                                    )),
                                ));
                            }
                        };

                        let value = {
                            let array = vm.heap().get_array(reference)?;

                            match array.elements.get(index as usize) {
                                Some(v) => v.clone(),
                                None => {
                                    return Err(vm.throw(
                                        "java/lang/ArrayIndexOutOfBoundsException",
                                        Some(&format!(
                                            "Index {} out of bounds for length {}",
                                            index,
                                            array.elements.len()
                                        )),
                                    ));
                                }
                            }
                        };

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.operand_stack.push(value);
                    }

                    Opcode::GetStatic => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let (owner_name, field_name, _descriptor) = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_field_ref(index)?;

                        let owner_ref = vm.resolve_class(&owner_name)?;

                        let slot = vm
                            .get_class(owner_ref)?
                            .find_static_field(&field_name)
                            .ok_or(InternalError::InvalidConstantPoolEntry)?
                            .slot;
                        let storage_ref = vm.static_storage_ref(owner_ref)?;
                        let value = vm.heap().get_object(storage_ref)?.fields[slot].clone();

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.operand_stack.push(value);
                    }

                    Opcode::PutStatic => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let value = frame
                            .operand_stack
                            .pop()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        let (owner_name, field_name, _descriptor) = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_field_ref(index)?;

                        let owner_ref = vm.resolve_class(&owner_name)?;

                        let slot = vm
                            .get_class(owner_ref)?
                            .find_static_field(&field_name)
                            .ok_or(InternalError::InvalidConstantPoolEntry)?
                            .slot;
                        let storage_ref = vm.static_storage_ref(owner_ref)?;
                        vm.heap_mut().get_object_mut(storage_ref)?.fields[slot] = value;
                    }

                    Opcode::InvokeVirtual => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let (_static_class_name, method_name, descriptor) = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_method_ref(index)?;
                        let param_slots =
                    crate::vm::runtime_method::RuntimeMethod::param_slot_count_from_descriptor(
                        &descriptor,
                    )?;

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();

                        let mut args = Vec::with_capacity(param_slots);
                        for _ in 0..param_slots {
                            args.push(
                                frame
                                    .operand_stack
                                    .pop()
                                    .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?,
                            );
                        }
                        args.reverse();

                        let objectref = match frame.operand_stack.pop() {
                            Some(Value::Reference(Some(r))) => r,
                            Some(Value::Reference(None)) => {
                                return Err(vm.throw(
                                    "java/lang/NullPointerException",
                                    Some("objrectref on invokevirtual is `null`"),
                                ));
                            }
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let obj_class = vm.heap().get_object(objectref)?.class;
                        let (resolved_class, method_idx) =
                            vm.resolve_virtual_method(obj_class, &method_name, &descriptor)?;
                        let (max_locals, max_stack) = {
                            let m = &vm.get_class(resolved_class)?.methods[method_idx];
                            (m.max_locals, m.max_stack)
                        };

                        let mut new_frame =
                            Frame::new(max_locals, max_stack, resolved_class, method_idx);
                        new_frame.locals[0] = Value::Reference(Some(objectref));
                        for (i, arg) in args.into_iter().enumerate() {
                            new_frame.locals[i + 1] = arg;
                        }
                        vm.get_thread(thread_ref)?.push_frame(new_frame);
                    }

                    Opcode::Ldc => {
                        let index = code[frame.pc] as u16;
                        frame.pc += 1;

                        let entry = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .entries
                            .get(index as usize)
                            .cloned()
                            .ok_or(InternalError::InvalidConstantPoolEntry)?;

                        let value = match entry {
                            crate::class::constant_pool::ConstantPoolEntry::Integer(i) => {
                                Value::Int(i)
                            }
                            crate::class::constant_pool::ConstantPoolEntry::Float(f) => {
                                Value::Float(f)
                            }
                            crate::class::constant_pool::ConstantPoolEntry::String {
                                string_index,
                            } => {
                                let s = vm
                                    .get_class(frame_class)?
                                    .constant_pool
                                    .get_utf8(string_index)?;
                                Value::Reference(Some(vm.heap_mut().allocate_string(s)))
                            }
                            _ => {
                                return Err(RuntimeError::Internal(
                                    InternalError::InvalidConstantPoolEntry,
                                ));
                            }
                        };

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.operand_stack.push(value);
                    }

                    Opcode::LdcW => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let entry = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .entries
                            .get(index as usize)
                            .cloned()
                            .ok_or(InternalError::InvalidConstantPoolEntry)?;

                        let value = match entry {
                            crate::class::constant_pool::ConstantPoolEntry::Integer(i) => {
                                Value::Int(i)
                            }
                            crate::class::constant_pool::ConstantPoolEntry::Float(f) => {
                                Value::Float(f)
                            }
                            crate::class::constant_pool::ConstantPoolEntry::String {
                                string_index,
                            } => {
                                let s = vm
                                    .get_class(frame_class)?
                                    .constant_pool
                                    .get_utf8(string_index)?;
                                Value::Reference(Some(vm.heap_mut().allocate_string(s)))
                            }
                            _ => {
                                return Err(RuntimeError::Internal(
                                    InternalError::InvalidConstantPoolEntry,
                                ));
                            }
                        };

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.operand_stack.push(value);
                    }

                    Opcode::IRem => {
                        let value2 = match frame.operand_stack.pop() {
                            Some(Value::Int(i)) => i,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        let value1 = match frame.operand_stack.pop() {
                            Some(Value::Int(i)) => i,
                            _ => return Err(RuntimeError::Internal(InternalError::InvalidType)),
                        };

                        if value2 == 0 {
                            return Err(vm.throw(
                                "java/lang/ArithmeticException",
                                Some("Tried to device by 0 on `irem`"),
                            ));
                        }

                        let res = value1 - (value1 / value2) * value2;

                        frame.operand_stack.push(Value::Int(res));
                    }
                }

                return Ok(StepOutcome::Continue);
            }

            MethodBody::Native(_func) => return Ok(StepOutcome::Return(Value::Int(0))),

            MethodBody::Abstract => {
                return Err(vm.throw(
                    "java/lang/Exception",
                    Some("Abstract Methods are not implemented"),
                ));
            }

            _ => {
                return Err(RuntimeError::Internal(InternalError::InvalidType));
            }
        }
    }
}
