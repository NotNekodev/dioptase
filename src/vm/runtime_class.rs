use crate::{class::class_file::ClassFile, vm::runtime_method::RuntimeMethod};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassRef(pub usize);

#[allow(dead_code)]
pub struct RuntimeClass {
    pub name: String,
    pub super_class: Option<ClassRef>,
    pub methods: Vec<RuntimeMethod>,
}

#[allow(dead_code)]
impl RuntimeClass {
    pub fn new(name: String, super_class: Option<ClassRef>) -> Self {
        Self {
            name,
            super_class,
            methods: Vec::new(),
        }
    }

    pub fn from_class_file(
        class_file: &ClassFile,
        super_class: Option<ClassRef>,
        class_ref: ClassRef,
    ) -> Result<Self, ClassRef> {
        let name = class_file
            .constant_pool
            .get_class_name(class_file.this_class)
            .expect(
                format!(
                    "Failed to get class name for class index {}",
                    class_file.this_class
                )
                .as_str(),
            );

        let mut runtime_class = RuntimeClass::new(name, super_class);

        for (i, method) in class_file.methods.iter().enumerate() {
            runtime_class.methods.push(
                RuntimeMethod::from_method_info(method, class_ref, &class_file.constant_pool)
                    .expect(
                        format!(
                            "Failed to call RuntimeMethod::from_method_info on method {}",
                            i
                        )
                        .as_str(),
                    ),
            );
        }

        Ok(runtime_class)
    }
}
