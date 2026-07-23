use std::collections::HashMap;

use crate::{
    class::{class_file::ClassFile, method::MethodAccessFlags, reader::ClassReader},
    error::RuntimeError,
    vm::{
        classpath::ClassPath,
        frame::Frame,
        heap::Heap,
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
    classpath: ClassPath,
    heap: Heap,
}

#[allow(dead_code)]
impl VM {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            threads: Vec::new(),
            main_thread: ThreadRef(0),
            classes_by_name: HashMap::new(),
            classpath: ClassPath::empty(),
            heap: Heap::new(),
        }
    }

    pub fn set_classpath(&mut self, classpath: ClassPath) {
        self.classpath = classpath;
    }

    pub fn resolve_class(&mut self, binary_name: &str) -> Result<ClassRef, RuntimeError> {
        if let Some(existing) = self.classes_by_name.get(binary_name) {
            return Ok(*existing);
        }

        let path = self.classpath.find_class_file(binary_name).ok_or_else(|| {
            RuntimeError::ClassNotFound {
                class: binary_name.to_string(),
            }
        })?;

        let data = std::fs::read(&path).map_err(|e| RuntimeError::ClassLoadError {
            class: binary_name.to_string(),
            source_cp: e.to_string(),
        })?;

        let mut reader = ClassReader::new(data);
        let class_file =
            ClassFile::read(&mut reader).map_err(|e| RuntimeError::ClassLoadError {
                class: binary_name.to_string(),
                source_cp: e.to_string(),
            })?;

        self.load_class(class_file)
    }

    pub fn get_class(&self, class_index: ClassRef) -> Result<&RuntimeClass, RuntimeError> {
        let option = self.classes.get(class_index.0);

        match option {
            Some(class) => return Ok(class),
            None => {
                return Err(RuntimeError::ClassNotFound {
                    class: format!("(index {}", class_index.0),
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
                class: format!("(index {}", class_index.0),
            })?;

        let method =
            class
                .methods
                .get(method_index)
                .ok_or_else(|| RuntimeError::MethodNotFound {
                    class: class.name.clone(),
                    method: format!("(index {}", method_index),
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
        let super_class = if class_file.super_class == 0 {
            None
        } else {
            let super_name = class_file
                .constant_pool
                .get_class_name(class_file.super_class)?;
            Some(self.resolve_class(&super_name)?)
        };

        let id = self.classes.len();
        let class_ref = ClassRef(id);

        let runtime_class = RuntimeClass::from_class_file(&class_file, super_class, class_ref)?;

        self.classes_by_name
            .insert(runtime_class.name.clone(), ClassRef(id));
        self.classes.push(runtime_class);

        Ok(class_ref)
    }

    pub fn run_main(&mut self, main_class: &str) -> Result<Value, RuntimeError> {
        let main_thread = self.create_thread();

        let main_class: ClassRef = self.resolve_class(main_class)?;
        let mut main_method_idx: Option<usize> = None;

        for (j, method) in self.get_class(main_class)?.methods.iter().enumerate() {
            if method.name == "main"
                && method.descriptor == "([Ljava/lang/String;)I"
                && method.access.contains(MethodAccessFlags::STATIC)
                && method.access.contains(MethodAccessFlags::PUBLIC)
            {
                main_method_idx = Some(j);
            }
        }

        let method_idx = main_method_idx.unwrap();

        let method = self.get_method(main_class, method_idx)?;

        let max_locals = method.max_locals;
        let max_stack = method.max_stack;

        let frame = Frame::new(max_locals, max_stack, main_class, method_idx);
        self.get_thread(main_thread)?.push_frame(frame);
        let result = Interpreter::run(self, main_thread)?;
        Ok(result)
    }

    pub fn heap(&self) -> &Heap {
        &self.heap
    }

    pub fn heap_mut(&mut self) -> &mut Heap {
        &mut self.heap
    }

    // TODO: expand with interfaces
    pub fn is_assignable(&self, from: ClassRef, to: ClassRef) -> Result<bool, RuntimeError> {
        if from == to {
            return Ok(true);
        }

        let mut current = Some(from);

        while let Some(class) = current {
            if class == to {
                return Ok(true);
            }

            current = self.get_class(class)?.super_class;
        }

        Ok(false)
    }
}
