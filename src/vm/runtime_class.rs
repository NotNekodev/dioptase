use crate::{
    class::{class_file::ClassFile, constant_pool::ConstantPool},
    error::RuntimeError,
    vm::{runtime_field::RuntimeField, runtime_method::RuntimeMethod},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassRef(pub usize);

#[allow(dead_code)]
pub struct RuntimeClass {
    pub name: String,
    pub super_class: Option<ClassRef>,
    pub methods: Vec<RuntimeMethod>,
    pub constant_pool: ConstantPool,
    pub instance_fields: Vec<RuntimeField>,
    pub static_fields: Vec<RuntimeField>,
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
            instance_fields: Vec::new(),
            static_fields: Vec::new(),
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

        let mut runtime_class = RuntimeClass::new(name.clone(), super_class);

        runtime_class.constant_pool = class_file.constant_pool.clone();

        let (instance_fields, static_fields) =
            RuntimeField::partition_field_infos(&class_file.fields, &class_file.constant_pool)?;
        runtime_class.instance_fields = instance_fields;
        runtime_class.static_fields = static_fields;

        for method in class_file.methods.iter() {
            let name = name.clone();
            runtime_class.methods.push(RuntimeMethod::from_method_info(
                method,
                class_ref,
                &class_file.constant_pool,
                name.as_str(),
            )?);
        }

        Ok(runtime_class)
    }

    pub fn find_method(&self, name: &str, descriptor: &str) -> Option<usize> {
        self.methods
            .iter()
            .position(|m| m.name == name && m.descriptor == descriptor)
    }

    pub fn find_field(&self, name: &str) -> Option<&RuntimeField> {
        self.instance_fields.iter().find(|f| f.name == name)
    }

    pub fn find_static_field(&self, name: &str) -> Option<&RuntimeField> {
        self.static_fields.iter().find(|f| f.name == name)
    }

    pub fn instance_slot_count(&self) -> usize {
        self.instance_fields
            .iter()
            .map(|f| {
                if f.descriptor == "J" || f.descriptor == "D" {
                    2
                } else {
                    1
                }
            })
            .sum()
    }

    pub fn static_slot_count(&self) -> usize {
        self.static_fields
            .iter()
            .map(|f| {
                if f.descriptor == "J" || f.descriptor == "D" {
                    2
                } else {
                    1
                }
            })
            .sum()
    }
}
