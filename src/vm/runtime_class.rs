use crate::{
    class::{class_file::ClassFile, constant_pool::ConstantPool},
    error::RuntimeError,
    vm::runtime_method::RuntimeMethod,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassRef(pub usize);

#[allow(dead_code)]
pub struct RuntimeClass {
    pub name: String,
    pub super_class: Option<ClassRef>,
    pub methods: Vec<RuntimeMethod>,
    pub constant_pool: ConstantPool,
}

#[allow(dead_code)]
impl RuntimeClass {
    pub fn new(name: String, super_class: Option<ClassRef>) -> Self {
        Self {
            name,
            super_class,
            methods: Vec::new(),
            constant_pool: ConstantPool {
                entries: Vec::new(),
            },
        }
    }

    pub fn from_class_file(
        class_file: &ClassFile,
        super_class: Option<ClassRef>,
        class_ref: ClassRef,
    ) -> Result<Self, RuntimeError> {
        let name = class_file
            .constant_pool
            .get_class_name(class_file.this_class)?;

        let mut runtime_class = RuntimeClass::new(name, super_class);

        runtime_class.constant_pool = class_file.constant_pool.clone();

        for (_, method) in class_file.methods.iter().enumerate() {
            runtime_class.methods.push(RuntimeMethod::from_method_info(
                method,
                class_ref,
                &class_file.constant_pool,
            )?);
        }

        Ok(runtime_class)
    }
}
