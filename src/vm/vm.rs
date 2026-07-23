use std::collections::HashMap;

use crate::{
    class::{class_file::ClassFile, method::MethodAccessFlags},
    error::RuntimeError,
    vm::{
        frame::Frame,
        interpreter::Interpreter,
        runtime_class::{ClassRef, RuntimeClass},
        runtime_method::RuntimeMethod,
        thread::{Thread, ThreadRef},
        value::Value,
    },
};

#[allow(dead_code)]
pub struct VM {
    classes: Vec<RuntimeClass>,
    classes_by_name: HashMap<String, ClassRef>,
    threads: Vec<Thread>,
    main_thread: ThreadRef,
}

#[allow(dead_code)]
impl VM {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            threads: Vec::new(),
            main_thread: ThreadRef(0),
            classes_by_name: HashMap::new(),
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
        self.classes_by_name
            .insert(class.name.clone(), ClassRef(id));
        self.classes.push(class);

        ClassRef(id)
    }

    pub fn create_thread(&mut self) -> ThreadRef {
        let id = self.threads.len();
        self.threads.push(Thread::new(id));
        ThreadRef(id)
    }

    pub fn get_thread(&mut self, thread_ref: ThreadRef) -> Result<&mut Thread, RuntimeError> {
        let option = self.threads.get_mut(thread_ref.0);

        match option {
            Some(thread) => Ok(thread),
            None => Err(RuntimeError::ThreadNotFound {
                thread_id: thread_ref.0,
            }),
        }
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

        self.classes_by_name
            .insert(runtime_class.name.clone(), ClassRef(id));
        self.classes.push(runtime_class);

        Ok(class_ref)
    }

    pub fn run_main(&mut self) -> Result<Value, RuntimeError> {
        let main_thread = self.create_thread();

        let mut main_class: Option<ClassRef> = None;
        let mut main_method_idx: Option<usize> = None;

        'outer: for (i, class) in self.classes.iter().enumerate() {
            for (j, method) in class.methods.iter().enumerate() {
                if method.name == "main"
                    && method.descriptor == "([Ljava/lang/String;)I"
                    && method.access.contains(MethodAccessFlags::STATIC)
                    && method.access.contains(MethodAccessFlags::PUBLIC)
                {
                    main_class = Some(ClassRef(i));
                    main_method_idx = Some(j);
                    break 'outer;
                }
            }
        }

        let class_ref = main_class.ok_or(RuntimeError::MethodNotFound {
            class: "main".into(),
            index: 0,
        })?;

        let method_idx = main_method_idx.unwrap();

        let method = self.get_method(class_ref, method_idx)?;

        let max_locals = method.max_locals;
        let max_stack = method.max_stack;

        let frame = Frame::new(max_locals, max_stack, class_ref, method_idx);
        self.threads[main_thread.0].push_frame(frame);
        let result = Interpreter::run(
            &self.classes,
            &self.classes_by_name,
            &mut self.threads[main_thread.0],
        )?;
        Ok(result)
    }
}
