use std::{
    ops::{BitAnd, Mul, Shl},
    str::FromStr,
};

use jdescriptor::MethodDescriptor;

use crate::{
    error::{InternalError, RuntimeError},
    native::{
        native_context::NativeContext,
        native_registry::{NativeMethodIdentifier, NativeMethodRegistry},
    },
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
                    frame.push_value(Value::Reference(Some(obj_ref)));
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

        let class_name = vm.get_class(frame_class)?.name.clone();
        let method_name = vm.get_method(frame_class, frame_method_idx)?.name.clone();

        macro_rules! invalid_type {
            ($pc:expr, $expected:expr, $found:expr) => {
                RuntimeError::Internal(InternalError::InvalidType {
                    class: class_name.clone(),
                    method: method_name.clone(),
                    pc: $pc,
                    expected: $expected.to_string(),
                    found: format!("{:?}", $found),
                })
            };
        }

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

                        frame.push_value(Value::Int(value as i32));
                    }

                    Opcode::Sipush => {
                        let byte1 = code[frame.pc];
                        frame.pc += 1;
                        let byte2 = code[frame.pc];
                        frame.pc += 1;
                        let value: i16 = i16::from_be_bytes([byte1, byte2]);

                        frame.push_value(Value::Int(value as i32));
                    }

                    Opcode::IConst0 => frame.push_value(Value::Int(0)),
                    Opcode::IConst1 => frame.push_value(Value::Int(1)),
                    Opcode::IConst2 => frame.push_value(Value::Int(2)),
                    Opcode::IConst3 => frame.push_value(Value::Int(3)),
                    Opcode::IConst4 => frame.push_value(Value::Int(4)),
                    Opcode::IConst5 => frame.push_value(Value::Int(5)),

                    Opcode::FConst0 => frame.push_value(Value::Float(0.0)),
                    Opcode::FConst1 => frame.push_value(Value::Float(1.0)),
                    Opcode::FConst2 => frame.push_value(Value::Float(2.0)),

                    Opcode::ILoad => {
                        let index = code[frame.pc] as usize;
                        frame.pc += 1;

                        frame.push_value(frame.locals[index].clone())
                    }

                    Opcode::ILoad0 => frame.push_value(frame.locals[0].clone()),
                    Opcode::ILoad1 => frame.push_value(frame.locals[1].clone()),
                    Opcode::ILoad2 => frame.push_value(frame.locals[2].clone()),
                    Opcode::ILoad3 => frame.push_value(frame.locals[3].clone()),

                    Opcode::FLoad0 => frame.push_value(frame.locals[0].clone()),
                    Opcode::FLoad1 => frame.push_value(frame.locals[1].clone()),
                    Opcode::FLoad2 => frame.push_value(frame.locals[2].clone()),
                    Opcode::FLoad3 => frame.push_value(frame.locals[3].clone()),

                    Opcode::ALoad0 => frame.push_value(frame.locals[0].clone()),
                    Opcode::ALoad1 => frame.push_value(frame.locals[1].clone()),
                    Opcode::ALoad2 => frame.push_value(frame.locals[2].clone()),
                    Opcode::ALoad3 => frame.push_value(frame.locals[3].clone()),

                    Opcode::AStore => {
                        let index = code[frame.pc] as usize;
                        frame.pc += 1;

                        frame.locals[index] = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }

                    Opcode::AStore0 => {
                        frame.locals[0] = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }
                    Opcode::AStore1 => {
                        frame.locals[1] = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }
                    Opcode::AStore2 => {
                        frame.locals[2] = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }
                    Opcode::AStore3 => {
                        frame.locals[3] = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }

                    Opcode::IStore => {
                        let index = code[frame.pc] as usize;
                        frame.pc += 1;

                        frame.locals[index] = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?
                    }

                    Opcode::IStore0 => {
                        frame.locals[0] = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?
                    }
                    Opcode::IStore1 => {
                        frame.locals[1] = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?
                    }
                    Opcode::IStore2 => {
                        frame.locals[2] = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?
                    }
                    Opcode::IStore3 => {
                        frame.locals[3] = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?
                    }

                    Opcode::Dup => {
                        let top = frame
                            .operand_stack
                            .last()
                            .cloned()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        frame.push_value(top);
                    }

                    Opcode::IAdd => {
                        let value2 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(x), Value::Int(y)) => {
                                frame.push_value(Value::Int(x.wrapping_add(y)));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Int, Int)", other));
                            }
                        }
                    }

                    Opcode::LAdd => {
                        let value2 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Long(x), Value::Long(y)) => {
                                frame.push_value(Value::Long(x.wrapping_add(y)));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Long, Long)", other));
                            }
                        }
                    }

                    Opcode::ISub => {
                        let value2 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(x), Value::Int(y)) => {
                                frame.push_value(Value::Int(x.wrapping_sub(y)));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Int, Int)", other));
                            }
                        }
                    }

                    Opcode::IMul => {
                        let value2 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(x), Value::Int(y)) => {
                                frame.push_value(Value::Int(x.wrapping_mul(y)));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Int, Int)", other));
                            }
                        }
                    }

                    Opcode::FMul => {
                        let value2 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Float(x), Value::Float(y)) => {
                                frame.push_value(Value::Float(x.mul(y)));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Float, Float)", other));
                            }
                        }
                    }

                    Opcode::IReturn => {
                        let ret = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        vm.get_thread(thread_ref)?.pop_frame();

                        match vm.get_thread(thread_ref)?.current_frame() {
                            Some(caller) => caller.push_value(ret),
                            None => return Ok(StepOutcome::Return(ret)),
                        }
                    }

                    Opcode::FReturn => {
                        let ret = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        vm.get_thread(thread_ref)?.pop_frame();

                        match vm.get_thread(thread_ref)?.current_frame() {
                            Some(caller) => caller.push_value(ret),
                            None => return Ok(StepOutcome::Return(ret)),
                        }
                    }

                    Opcode::DReturn => {
                        let ret = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        vm.get_thread(thread_ref)?.pop_frame();

                        match vm.get_thread(thread_ref)?.current_frame() {
                            Some(caller) => caller.push_value(ret),
                            None => return Ok(StepOutcome::Return(ret)),
                        }
                    }

                    Opcode::AReturn => {
                        let ret = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        if !matches!(ret, Value::Reference(_)) {
                            return Err(invalid_type!(frame.pc, "Reference", ret));
                        }

                        vm.get_thread(thread_ref)?.pop_frame();

                        match vm.get_thread(thread_ref)?.current_frame() {
                            Some(caller) => caller.push_value(ret),
                            None => return Ok(StepOutcome::Return(ret)),
                        }
                    }

                    Opcode::FCmpL => {
                        let value2 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Float(value1), Value::Float(value2)) => {
                                if value1 == value2 {
                                    frame.push_value(Value::Int(0));
                                }

                                if value1 > value2 {
                                    frame.push_value(Value::Int(1));
                                }

                                if value1 < value2 {
                                    frame.push_value(Value::Int(-1));
                                }
                            }

                            other => {
                                return Err(invalid_type!(frame.pc, "(Float, Float)", other));
                            }
                        }
                    }

                    Opcode::FCmpG => {
                        let value2 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Float(value1), Value::Float(value2)) => {
                                if value1 == value2 {
                                    frame.push_value(Value::Int(0));
                                }

                                if value1 > value2 {
                                    frame.push_value(Value::Int(1));
                                }

                                if value1 < value2 {
                                    frame.push_value(Value::Int(-1));
                                }
                            }

                            other => {
                                return Err(invalid_type!(frame.pc, "(Float, Float)", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 == value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Int, Int)", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 != value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Int, Int)", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 < value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Int, Int)", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 <= value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Int, Int)", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 > value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Int, Int)", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let value1 = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match (value1, value2) {
                            (Value::Int(value1), Value::Int(value2)) => {
                                if value1 >= value2 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "(Int, Int)", other));
                            }
                        }
                    }

                    Opcode::GetField => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let objectref = match frame.pop_value() {
                            Some(Value::Reference(Some(r))) => r,
                            other => {
                                return Err(invalid_type!(frame.pc, "Reference", other));
                            }
                        };

                        let (owner_name, field_name, _descriptor) = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_field_ref(index)?;
                        let owner_ref = vm.resolve_class(&owner_name)?;

                        let slot = vm
                            .find_instance_field(owner_ref, &field_name)?
                            .map(|(_, slot)| slot)
                            .ok_or(InternalError::InvalidSlot)?;

                        let value = vm.heap().get_object(objectref)?.fields[slot].clone();

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.push_value(value);
                    }

                    Opcode::PutField => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let value = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        let objectref = match frame.pop_value() {
                            Some(Value::Reference(Some(r))) => r,
                            other => {
                                return Err(invalid_type!(frame.pc, "Reference", other));
                            }
                        };

                        let (owner_name, field_name, _descriptor) = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_field_ref(index)?;
                        let owner_ref = vm.resolve_class(&owner_name)?;

                        let slot = vm
                            .find_instance_field(owner_ref, &field_name)?
                            .map(|(_, slot)| slot)
                            .ok_or(InternalError::InvalidSlot)?;

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

                        let param_types = MethodDescriptor::from_str(&descriptor);
                        for _ in 0..param_types.unwrap().parameter_types().len() {
                            args.push(
                                frame
                                    .pop_value()
                                    .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?,
                            );
                        }
                        args.reverse();

                        let objectref = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                        args.insert(0, objectref);

                        let mut new_frame =
                            Frame::new(max_locals, max_stack, target_class_ref, target_method_idx);

                        let mut slot = 0;
                        for arg in args {
                            let width =
                                matches!(arg, Value::Long(_) | Value::Double(_)) as usize + 1;
                            new_frame.locals[slot] = arg;
                            slot += width;
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

                        vm.ensure_class_initialized(target_class_ref)?;

                        let (target_method_idx, max_locals, max_stack, argument_count, body) = {
                            let target_class = vm.get_class(target_class_ref)?;
                            let idx = target_class.find_method(&method_name, &descriptor).ok_or(
                                InternalError::MethodNotFound {
                                    class: target_class.name.clone(),
                                    method: method_name.clone(),
                                },
                            )?;

                            let method = &target_class.methods[idx];

                            (
                                idx,
                                method.max_locals,
                                method.max_stack,
                                method.param_slot_count(),
                                method.body.clone(),
                            )
                        };

                        let args = {
                            let frame = vm.get_thread(thread_ref)?.current_frame().ok_or(
                                InternalError::NoCurrentFrame {
                                    thread_id: thread_ref.0,
                                },
                            )?;

                            let mut args = Vec::with_capacity(argument_count);

                            let param_types = MethodDescriptor::from_str(&descriptor);
                            for _ in 0..param_types.unwrap().parameter_types().len() {
                                args.push(frame.pop_value().ok_or(
                                    InternalError::OperandStackUnderflow { pc: frame.pc },
                                )?);
                            }

                            args.reverse();
                            args
                        };

                        match body {
                            MethodBody::Bytecode(_) => {
                                if max_locals < argument_count {
                                    return Err(RuntimeError::Internal(
                                        InternalError::InvalidMethodLocals {
                                            method: method_name,
                                            expected: argument_count,
                                            actual: max_locals,
                                        },
                                    ));
                                }

                                let mut new_frame = Frame::new(
                                    max_locals,
                                    max_stack,
                                    target_class_ref,
                                    target_method_idx,
                                );

                                let mut slot = 0;
                                for arg in args {
                                    let width = matches!(arg, Value::Long(_) | Value::Double(_))
                                        as usize
                                        + 1;
                                    new_frame.locals[slot] = arg;
                                    slot += width;
                                }

                                vm.get_thread(thread_ref)?.push_frame(new_frame);
                            }

                            MethodBody::Native => {
                                let mut new_frame = Frame::new(
                                    argument_count,
                                    0,
                                    target_class_ref,
                                    target_method_idx,
                                );

                                let mut slot = 0;
                                for arg in args {
                                    let width = matches!(arg, Value::Long(_) | Value::Double(_))
                                        as usize
                                        + 1;
                                    new_frame.locals[slot] = arg;
                                    slot += width;
                                }

                                vm.get_thread(thread_ref)?.push_frame(new_frame);
                            }

                            MethodBody::Abstract => {
                                return Err(RuntimeError::Internal(
                                    InternalError::AbstractMethod {
                                        class: target_class_name,
                                        method: method_name,
                                    },
                                ));
                            }

                            MethodBody::Unknown => {
                                return Err(RuntimeError::Internal(
                                    InternalError::NoCodeInMethod {
                                        method: method_name,
                                    },
                                ));
                            }
                        }
                    }

                    Opcode::New => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let class_name = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_class_name(index)?;
                        let target_class_ref = vm.resolve_class(&class_name)?;
                        vm.ensure_class_initialized(target_class_ref)?;
                        let defaults = vm.default_field_values(target_class_ref)?;
                        let obj_ref = vm
                            .heap_mut()
                            .allocate_object_typed(target_class_ref, &defaults);

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.push_value(Value::Reference(Some(obj_ref)));
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
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        }
                    }

                    Opcode::I2F => {
                        let value = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                frame.push_value(Value::Float(val as f32));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        }
                    }

                    Opcode::F2I => {
                        let value = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Float(val) => {
                                frame.push_value(Value::Int(val as i32));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Float", other));
                            }
                        }
                    }

                    Opcode::I2L => {
                        let value = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                frame.push_value(Value::Long(val as i64));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val == 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val != 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val < 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val <= 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val > 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match value {
                            Value::Int(val) => {
                                if val >= 0 {
                                    frame.pc = branch_ip as usize + opcode_pc
                                }
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        }
                    }

                    Opcode::AConstNull => {
                        frame.push_value(Value::Reference(None));
                    }

                    Opcode::IfNonNull => {
                        let opcode_pc = frame.pc - 1;
                        let branchbyte1 = code[frame.pc];
                        frame.pc += 1;
                        let branchbyte2 = code[frame.pc];
                        frame.pc += 1;
                        let branch_ip: i16 = i16::from_be_bytes([branchbyte1, branchbyte2]);

                        let reference_value = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match reference_value {
                            Value::Reference(reference) => match reference {
                                Some(_) => {
                                    frame.pc = branch_ip as usize + opcode_pc;
                                }

                                None => {}
                            },
                            other => {
                                return Err(invalid_type!(frame.pc, "Reference", other));
                            }
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
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        match reference_value {
                            Value::Reference(reference) => match reference {
                                Some(_) => {}

                                None => {
                                    frame.pc = branch_ip as usize + opcode_pc;
                                }
                            },
                            other => {
                                return Err(invalid_type!(frame.pc, "Reference", other));
                            }
                        }
                    }

                    Opcode::Pop => {
                        let _ = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;
                    }

                    Opcode::NewArray => {
                        let atype = code[frame.pc];
                        frame.pc += 1;

                        let length = match frame.pop_value() {
                            Some(Value::Int(n)) if n >= 0 => n as usize,
                            Some(Value::Int(_)) => {
                                return Err(vm.throw(
                                    "java/lang/NegativeArraySizeException",
                                    Some("Cannot create negative sized array"),
                                ));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let array_type = ArrayElementType::try_from(atype)
                            .map_err(|_| InternalError::InvalidArrayType { atype })?;

                        let array_ref = vm.heap_mut().allocate_array(array_type, length);

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.push_value(Value::Reference(Some(array_ref)));
                    }

                    Opcode::ANewArray => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let length = match frame.pop_value() {
                            Some(Value::Int(n)) if n >= 0 => n as usize,
                            Some(Value::Int(_)) => {
                                return Err(vm.throw(
                                    "java/lang/NegativeArraySizeException",
                                    Some("Cannot create negative sized array"),
                                ));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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
                        frame.push_value(Value::Reference(Some(array_ref)));
                    }

                    Opcode::CAStore => {
                        let value = match frame.pop_value() {
                            Some(Value::Int(n)) => n as i32,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let index = match frame.pop_value() {
                            Some(Value::Int(n)) => n as usize,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let arrayref = match frame.pop_value() {
                            Some(Value::Reference(aref)) => aref,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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

                    Opcode::IAStore => {
                        let value = match frame.pop_value() {
                            Some(Value::Int(n)) => n as i32,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let index = match frame.pop_value() {
                            Some(Value::Int(n)) => n as usize,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let arrayref = match frame.pop_value() {
                            Some(Value::Reference(aref)) => aref,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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
                        let index = match frame.pop_value() {
                            Some(Value::Int(n)) => n as usize,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let arrayref = match frame.pop_value() {
                            Some(Value::Reference(aref)) => aref,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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
                        frame.push_value(value);
                    }

                    Opcode::CALoad => {
                        let index = match frame.pop_value() {
                            Some(Value::Int(n)) => n as usize,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let arrayref = match frame.pop_value() {
                            Some(Value::Reference(aref)) => aref,
                            other => {
                                return Err(invalid_type!(frame.pc, "Reference", other));
                            }
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
                        frame.push_value(value);
                    }

                    Opcode::ArrayLength => {
                        let arrayref = match frame.pop_value() {
                            Some(Value::Reference(aref)) => aref,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
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
                        frame.push_value(Value::Int(length as i32));
                    }

                    Opcode::AAStore => {
                        let pc = frame.pc.clone();
                        let value = match frame.pop_value() {
                            Some(Value::Reference(aref)) => Value::Reference(aref),
                            other => {
                                return Err(invalid_type!(pc, "Reference", other));
                            }
                        };

                        let index = match frame.pop_value() {
                            Some(Value::Int(aref)) => aref,
                            other => {
                                return Err(invalid_type!(pc, "Reference", other));
                            }
                        };

                        let arrayref = match frame.pop_value() {
                            Some(Value::Reference(aref)) => aref,
                            other => {
                                return Err(invalid_type!(pc, "Reference", other));
                            }
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
                                other => {
                                    return Err(invalid_type!(pc, "Reference", other));
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

                            other => {
                                return Err(invalid_type!(pc, "Reference", other));
                            }
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
                        let index = match frame.pop_value() {
                            Some(Value::Int(aref)) => aref,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let arrayref = match frame.pop_value() {
                            Some(Value::Reference(aref)) => aref,
                            other => {
                                return Err(invalid_type!(frame.pc, "Reference", other));
                            }
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
                        frame.push_value(value);
                    }

                    Opcode::GetStatic => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let (owner_name, field_name, descriptor) = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_field_ref(index)?;

                        let owner_ref = vm.resolve_class(&owner_name)?;

                        vm.ensure_class_initialized(owner_ref)?;

                        let slot = vm
                            .get_class(owner_ref)?
                            .find_static_field(&field_name)
                            .ok_or(InternalError::InvalidSlot)?
                            .slot;
                        let storage_ref = vm.static_storage_ref(owner_ref)?;
                        let value = vm.heap().get_object(storage_ref)?.fields[slot].clone();

                        println!(
                            "GETSTATIC {}.{}:{} = {:?}",
                            owner_name, field_name, descriptor, value
                        );

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.push_value(value);
                    }

                    Opcode::PutStatic => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let value = frame
                            .pop_value()
                            .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?;

                        let (owner_name, field_name, _descriptor) = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_field_ref(index)?;

                        let owner_ref = vm.resolve_class(&owner_name)?;

                        vm.ensure_class_initialized(owner_ref)?;

                        let slot = vm
                            .get_class(owner_ref)?
                            .find_static_field(&field_name)
                            .ok_or(InternalError::InvalidSlot)?
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
                        let param_types = MethodDescriptor::from_str(&descriptor);
                        for _ in 0..param_types.unwrap().parameter_types().len() {
                            args.push(
                                frame
                                    .pop_value()
                                    .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?,
                            );
                        }
                        args.reverse();

                        let objectref = match frame.pop_value() {
                            Some(Value::Reference(Some(r))) => r,
                            Some(Value::Reference(None)) => {
                                return Err(vm.throw(
                                    "java/lang/NullPointerException",
                                    Some("objrectref on invokevirtual is `null`"),
                                ));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Reference", other));
                            }
                        };

                        let obj_class = vm.runtime_class_of(objectref)?;
                        let (resolved_class, method_idx) =
                            vm.resolve_virtual_method(obj_class, &method_name, &descriptor)?;
                        let (max_locals, max_stack) = {
                            let m = &vm.get_class(resolved_class)?.methods[method_idx];
                            (m.max_locals, m.max_stack)
                        };

                        let mut new_frame =
                            Frame::new(max_locals, max_stack, resolved_class, method_idx);
                        new_frame.locals[0] = Value::Reference(Some(objectref));
                        let mut slot = 1;
                        for arg in args {
                            let width =
                                matches!(arg, Value::Long(_) | Value::Double(_)) as usize + 1;
                            new_frame.locals[slot] = arg;
                            slot += width;
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
                            .ok_or(InternalError::InvalidConstantPoolEntry {
                                index,
                                expected: "Any constant pool data type",
                                found: "out of bounds".to_string(),
                            })?;

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
                                Value::Reference(Some(vm.intern_string(&s)?))
                            }
                            crate::class::constant_pool::ConstantPoolEntry::Class {
                                name_index,
                            } => {
                                let class_name = vm
                                    .get_class(frame_class)?
                                    .constant_pool
                                    .get_utf8(name_index)?;
                                let target_class_ref = vm.resolve_class(&class_name)?;
                                let class_obj_ref = vm.class_object_for(target_class_ref);
                                Value::Reference(Some(class_obj_ref))
                            }
                            other => {
                                return Err(RuntimeError::Internal(
                                    InternalError::InvalidConstantPoolEntry {
                                        index,
                                        expected: "Any data type",
                                        found: format!("{:?}", other),
                                    },
                                ));
                            }
                        };

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.push_value(value);
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
                            .ok_or(InternalError::InvalidConstantPoolEntry {
                                index,
                                expected: "Any constant pool data type",
                                found: "out of bounds".to_string(),
                            })?;

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
                                Value::Reference(Some(vm.intern_string(&s)?))
                            }
                            crate::class::constant_pool::ConstantPoolEntry::Class {
                                name_index,
                            } => {
                                let class_name = vm
                                    .get_class(frame_class)?
                                    .constant_pool
                                    .get_utf8(name_index)?;
                                let target_class_ref = vm.resolve_class(&class_name)?;
                                let class_obj_ref = vm.class_object_for(target_class_ref);
                                Value::Reference(Some(class_obj_ref))
                            }
                            other => {
                                return Err(RuntimeError::Internal(
                                    InternalError::InvalidConstantPoolEntry {
                                        index,
                                        expected: "Integer, Float, String or Class",
                                        found: format!("{:?}", other),
                                    },
                                ));
                            }
                        };

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        frame.push_value(value);
                    }

                    Opcode::Ldc2W => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let entry = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .entries
                            .get(index as usize)
                            .cloned()
                            .ok_or(InternalError::InvalidConstantPoolEntry {
                                index,
                                expected: "Any constant pool data type",
                                found: "out of bounds".to_string(),
                            })?;

                        let value = match entry {
                            crate::class::constant_pool::ConstantPoolEntry::Long(i) => {
                                Value::Long(i)
                            }
                            crate::class::constant_pool::ConstantPoolEntry::Double(f) => {
                                Value::Double(f)
                            }
                            /*crate::class::constant_pool::ConstantPoolEntry::String {
                                string_index,
                            } => {
                                let s = vm
                                    .get_class(frame_class)?
                                    .constant_pool
                                    .get_utf8(string_index)?;
                                Value::Reference(Some(vm.heap_mut().allocate_string(s)))
                            }
                            crate::class::constant_pool::ConstantPoolEntry::Class {
                                name_index,
                            } => {
                                let class_name = vm
                                    .get_class(frame_class)?
                                    .constant_pool
                                    .get_utf8(name_index)?;
                                let target_class_ref = vm.resolve_class(&class_name)?;
                                let class_obj_ref = vm.class_object_for(target_class_ref);
                                Value::Reference(Some(class_obj_ref))
                            }*/
                            other => {
                                return Err(RuntimeError::Internal(
                                    InternalError::InvalidConstantPoolEntry {
                                        index,
                                        expected: "Long or Double",
                                        found: format!("{:?}", other),
                                    },
                                ));
                            }
                        };

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                        match value {
                            Value::Long(x) => {
                                frame.push_value(Value::Long(x));
                            }
                            Value::Double(x) => {
                                frame.push_value(Value::Double(x));
                            }

                            other => {
                                return Err(RuntimeError::Internal(
                                    InternalError::InvalidConstantPoolEntry {
                                        index,
                                        expected: "Long or Double",
                                        found: format!("{:?}", other),
                                    },
                                ));
                            }
                        }
                    }

                    Opcode::IRem => {
                        let value2 = match frame.pop_value() {
                            Some(Value::Int(i)) => i,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let value1 = match frame.pop_value() {
                            Some(Value::Int(i)) => i,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        if value2 == 0 {
                            return Err(vm.throw(
                                "java/lang/ArithmeticException",
                                Some("Tried to device by 0 on `irem`"),
                            ));
                        }

                        let res = value1 - (value1 / value2) * value2;

                        frame.push_value(Value::Int(res));
                    }

                    Opcode::LShl => {
                        let value2 = match frame.pop_value() {
                            Some(Value::Int(i)) => i,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let value1 = match frame.pop_value() {
                            Some(Value::Long(i)) => i,
                            other => {
                                return Err(invalid_type!(frame.pc, "Double", other));
                            }
                        };

                        let res = value1.shl(value2);

                        frame.push_value(Value::Long(res));
                    }

                    Opcode::LAnd => {
                        let value2 = match frame.pop_value() {
                            Some(Value::Long(i)) => i,
                            other => {
                                return Err(invalid_type!(frame.pc, "Long", other));
                            }
                        };

                        let value1 = match frame.pop_value() {
                            Some(Value::Long(i)) => i,
                            other => {
                                return Err(invalid_type!(frame.pc, "Long", other));
                            }
                        };

                        let res = value1.bitand(value2);

                        frame.push_value(Value::Long(res));
                    }

                    Opcode::IAnd => {
                        let value2 = match frame.pop_value() {
                            Some(Value::Int(i)) => i,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let value1 = match frame.pop_value() {
                            Some(Value::Int(i)) => i,
                            other => {
                                return Err(invalid_type!(frame.pc, "Int", other));
                            }
                        };

                        let res = value1.bitand(value2);

                        frame.push_value(Value::Int(res));
                    }

                    Opcode::InvokeInterface => {
                        let index = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]);
                        frame.pc += 2;

                        let _count = code[frame.pc];
                        frame.pc += 1;

                        let _reserved = code[frame.pc];
                        frame.pc += 1;

                        let (_iface_name, method_name, descriptor) = vm
                            .get_class(frame_class)?
                            .constant_pool
                            .get_interface_method_ref(index)?;
                        let param_slots =
                               crate::vm::runtime_method::RuntimeMethod::param_slot_count_from_descriptor(&descriptor)?;

                        let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();

                        let mut args = Vec::with_capacity(param_slots);
                        let param_types = MethodDescriptor::from_str(&descriptor);
                        for _ in 0..param_types.unwrap().parameter_types().len() {
                            args.push(
                                frame
                                    .pop_value()
                                    .ok_or(InternalError::OperandStackUnderflow { pc: frame.pc })?,
                            );
                        }
                        args.reverse();

                        let objectref = match frame.pop_value() {
                            Some(Value::Reference(Some(r))) => r,
                            Some(Value::Reference(None)) => {
                                return Err(vm.throw(
                                    "java/lang/NullPointerException",
                                    Some("objectref on invokeinterface is null"),
                                ));
                            }
                            other => {
                                return Err(invalid_type!(frame.pc, "Reference", other));
                            }
                        };

                        let obj_class = vm.runtime_class_of(objectref)?;
                        let (resolved_class, method_idx) =
                            vm.resolve_virtual_method(obj_class, &method_name, &descriptor)?;
                        let (max_locals, max_stack) = {
                            let m = &vm.get_class(resolved_class)?.methods[method_idx];
                            (m.max_locals, m.max_stack)
                        };

                        let mut new_frame =
                            Frame::new(max_locals, max_stack, resolved_class, method_idx);
                        new_frame.locals[0] = Value::Reference(Some(objectref));
                        let mut slot = 1;
                        for arg in args {
                            let width =
                                matches!(arg, Value::Long(_) | Value::Double(_)) as usize + 1;
                            new_frame.locals[slot] = arg;
                            slot += width;
                        }
                        vm.get_thread(thread_ref)?.push_frame(new_frame);
                    }
                }

                return Ok(StepOutcome::Continue);
            }

            MethodBody::Native => {
                let (class_ref, method_index) = {
                    let frame = vm.get_thread(thread_ref)?.current_frame().ok_or(
                        InternalError::NoCurrentFrame {
                            thread_id: thread_ref.0,
                        },
                    )?;

                    (frame.class, frame.method_index)
                };

                let (method_name, descriptor, receiver_slots) = {
                    let method = vm.get_method(class_ref, method_index)?;

                    (
                        method.name.clone(),
                        method.descriptor.clone(),
                        method.param_slot_count_with_receiver(),
                    )
                };

                let class_name = vm.get_class(class_ref)?.name.clone();

                let id: NativeMethodIdentifier =
                    NativeMethodIdentifier::new(class_name, method_name, descriptor);

                let native = NativeMethodRegistry::from_inventory()
                    .get(&id)
                    .ok_or_else(|| {
                        vm.throw(
                            "java/lang/UnsatisfiedLinkError",
                            Some(&format!(
                                "{}.{}{}",
                                id.class_name, id.method_name, id.descriptor,
                            )),
                        )
                    })?;

                let args = {
                    let frame = vm.get_thread(thread_ref)?.current_frame().unwrap();
                    frame.locals[..receiver_slots].to_vec()
                };

                let mut ctx = NativeContext::new(vm, thread_ref);

                let result = native(&mut ctx, &args)?;

                ctx.vm_mut().get_thread(thread_ref)?.pop_frame();

                if let Some(caller) = ctx.vm_mut().get_thread(thread_ref)?.current_frame() {
                    if let Some(value) = result {
                        caller.push_value(value);
                    }

                    return Ok(StepOutcome::Continue);
                }

                return Ok(StepOutcome::Return(result.unwrap_or(Value::Empty)));
            }

            MethodBody::Abstract => {
                return Err(vm.throw(
                    "java/lang/Exception",
                    Some("Abstract Methods are not implemented"),
                ));
            }

            other => {
                return Err(invalid_type!(frame.pc, "Function Body", other));
            }
        }
    }
}
