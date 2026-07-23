use crate::{
    class::{
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

    pub code: Vec<u8>,
}

impl RuntimeMethod {
    pub fn from_method_info(
        method: &MethodInfo,
        class: ClassRef,
        constant_pool: &ConstantPool,
    ) -> Result<Self, RuntimeError> {
        let method_str: String = constant_pool.get_utf8(method.name_index).expect(
            format!(
                "MethodNameIndex {} is not a CONSTANT_Utf8",
                method.name_index
            )
            .as_str(),
        );

        let descriptor: String = constant_pool.get_utf8(method.descriptor_index).expect(
            format!(
                "DescriptorIndex {} is not a CONSTANT_Utf8",
                method.descriptor_index
            )
            .as_str(),
        );

        Ok(Self {
            name: method_str,
            descriptor: descriptor,
            class,
            access: method.access_flags,

            max_stack: 0,
            max_locals: 0,

            code: Vec::new(),
        })
    }
}
