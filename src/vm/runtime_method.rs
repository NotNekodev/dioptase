use std::{rc::Rc, str::FromStr};

use jdescriptor::{MethodDescriptor, TypeDescriptor};

use crate::{
    class::{
        attributes::Attribute,
        constant_pool::ConstantPool,
        method::{MethodAccessFlags, MethodInfo},
    },
    error::RuntimeError,
    vm::runtime_class::ClassRef,
};

#[allow(dead_code)]
pub struct RuntimeMethod {
    pub name: String,
    pub descriptor: String,
    pub class: ClassRef,
    pub access: MethodAccessFlags,

    pub max_stack: usize,
    pub max_locals: usize,

    pub code: Rc<[u8]>,
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
        let mut code: Vec<u8> = Vec::new();

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
                code = bytecode.clone();
                found_code_attribute = true;
            }
        }

        if !found_code_attribute {
            if !method.access_flags.contains(MethodAccessFlags::ABSTRACT) {
                return Err(RuntimeError::NoCodeInMethod {
                    method: method_str.clone(),
                });
            }
        }

        Ok(Self {
            name: method_str,
            descriptor: descriptor,
            class,
            access: method.access_flags,

            max_stack: max_stack,
            max_locals: max_locals,

            code: code.into(),
        })
    }

    pub fn param_slot_count_from_descriptor(descriptor: &str) -> Result<usize, RuntimeError> {
        let d = MethodDescriptor::from_str(descriptor)
            .map_err(|_| RuntimeError::InvalidConstantPoolEntry)?;
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
}
