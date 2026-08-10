use std::{str::FromStr, sync::Arc};

use jdescriptor::{MethodDescriptor, TypeDescriptor};

use crate::{
    class::{
        attributes::Attribute,
        constant_pool::ConstantPool,
        method::{MethodAccessFlags, MethodInfo},
    },
    error::{InternalError, RuntimeError},
    vm::runtime_class::ClassRef,
};

#[derive(Clone, Debug)]
pub struct RuntimeExceptionHandler {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_class: Option<String>, // None = catch-all (used for `finally`)
}

#[derive(Clone, Debug)]
pub enum MethodBody {
    Bytecode(Arc<[u8]>),
    Native,
    Abstract,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RuntimeMethod {
    pub name: String,
    pub descriptor: String,
    pub class: ClassRef,
    pub access: MethodAccessFlags,

    pub max_stack: usize,
    pub max_locals: usize,

    pub body: MethodBody,

    pub exception_handlers: Vec<RuntimeExceptionHandler>,
}

impl RuntimeMethod {
    pub fn from_method_info(
        method: &MethodInfo,
        class: ClassRef,
        constant_pool: &ConstantPool,
    ) -> Result<Self, RuntimeError> {
        let method_str: String = constant_pool.get_utf8(method.name_index)?;
        let descriptor: String = constant_pool.get_utf8(method.descriptor_index)?;

        let mut max_stack = 0;
        let mut max_locals = 0;
        let mut body: MethodBody = MethodBody::Unknown;

        let mut found_code_attribute = false;

        for attribute in &method.attributes {
            if let Attribute::Code {
                max_stack: stack,
                max_locals: locals,
                code: bytecode,
                ..
            } = attribute
            {
                max_stack = *stack as usize;
                max_locals = *locals as usize;
                body = MethodBody::Bytecode(bytecode.clone().into());
                found_code_attribute = true;
            }
        }

        if !found_code_attribute {
            if method.access_flags.contains(MethodAccessFlags::NATIVE) {
                body = MethodBody::Native;
            } else if method.access_flags.contains(MethodAccessFlags::ABSTRACT) {
                body = MethodBody::Abstract;
            } else {
                return Err(RuntimeError::Internal(InternalError::NoCodeInMethod {
                    method: method_str.clone(),
                }));
            }
        }

        let mut exception_handlers = Vec::new();

        for attribute in &method.attributes {
            if let Attribute::Code {
                exception_table, ..
            } = attribute
            {
                for entry in exception_table {
                    let catch_class = if entry.catch_type == 0 {
                        None
                    } else {
                        Some(constant_pool.get_class_name(entry.catch_type)?)
                    };
                    exception_handlers.push(RuntimeExceptionHandler {
                        start_pc: entry.start_pc,
                        end_pc: entry.end_pc,
                        handler_pc: entry.handler_pc,
                        catch_class,
                    });
                }
            }
        }

        Ok(Self {
            name: method_str,
            descriptor: descriptor,
            class,
            access: method.access_flags,

            max_stack: max_stack,
            max_locals: max_locals,

            body: body,
            exception_handlers,
        })
    }

    pub fn param_slot_count_from_descriptor(descriptor: &str) -> Result<usize, RuntimeError> {
        let d =
            MethodDescriptor::from_str(descriptor).map_err(|_| InternalError::InvalidDescriptor)?;
        Ok(d.parameter_types()
            .iter()
            .map(|p| match p {
                TypeDescriptor::Long | TypeDescriptor::Double => 2,
                _ => 1,
            })
            .sum())
    }

    pub fn param_slot_count(&self) -> usize {
        Self::param_slot_count_from_descriptor(&self.descriptor).unwrap()
    }

    pub fn param_slot_count_with_receiver(&self) -> usize {
        let mut slots = self.param_slot_count();

        if !self.access.contains(MethodAccessFlags::STATIC) {
            slots += 1;
        }

        slots
    }
}
