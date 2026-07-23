use crate::{
    class::class_file::ClassFile,
    error::RuntimeError,
    vm::{
        runtime_class::{ClassRef, RuntimeClass},
        runtime_method::RuntimeMethod,
    },
};

#[allow(dead_code)]
pub struct VM {
    classes: Vec<RuntimeClass>,
}

#[allow(dead_code)]
impl VM {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
        }
    }

    pub fn get_class(&self, class_index: ClassRef) -> Result<&RuntimeClass, RuntimeError> {
        let option = self.classes.get(class_index.0);

        match option {
            Some(class) => return Ok(class),
            None => {
                return Err(RuntimeError::ClassNotFound {
                    index: class_index.0,
                });
            }
        }
    }

    pub fn get_method(
        &self,
        class_index: ClassRef,
        method_index: usize,
    ) -> Result<&RuntimeMethod, RuntimeError> {
        let class = self
            .classes
            .get(class_index.0)
            .ok_or_else(|| RuntimeError::ClassNotFound {
                index: class_index.0,
            })?;

        let method =
            class
                .methods
                .get(method_index)
                .ok_or_else(|| RuntimeError::MethodNotFound {
                    class: class.name.clone(),
                    index: method_index,
                })?;

        Ok(method)
    }

    pub fn add_class(&mut self, class: RuntimeClass) -> ClassRef {
        let id = self.classes.len();
        self.classes.push(class);

        ClassRef(id)
    }

    pub fn load_class(&mut self, class_file: ClassFile) -> Result<ClassRef, RuntimeError> {
        let id = self.classes.len();
        let class_ref = ClassRef(id);

        let super_class = if class_file.super_class == 0 {
            None
        } else {
            None
        };

        let runtime_class = RuntimeClass::from_class_file(&class_file, super_class, class_ref)?;

        self.classes.push(runtime_class);

        Ok(class_ref)
    }
}
