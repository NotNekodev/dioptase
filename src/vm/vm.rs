use std::collections::HashMap;

use crate::{
    class::{class_file::ClassFile, method::MethodAccessFlags, reader::ClassReader},
    error::{InternalError, RuntimeError},
    vm::{
        classpath::ClassPath,
        frame::Frame,
        heap::Heap,
        interpreter::Interpreter,
        runtime_class::{ClassRef, RuntimeClass},
        runtime_field::RuntimeField,
        runtime_method::RuntimeMethod,
        thread::{Thread, ThreadRef},
        value::{ObjectRef, Value},
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
    static_storage: HashMap<ClassRef, ObjectRef>,
    exceptions_registered: bool,
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
            static_storage: HashMap::new(),
            exceptions_registered: false,
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
            InternalError::ClassNotFound {
                class: binary_name.to_string(),
            }
        })?;

        let data = std::fs::read(&path).map_err(|e| InternalError::ClassLoadError {
            class: binary_name.to_string(),
            source_cp: e.to_string(),
        })?;

        let mut reader = ClassReader::new(data);
        let class_file =
            ClassFile::read(&mut reader).map_err(|e| InternalError::ClassLoadError {
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
                return Err(RuntimeError::Internal(InternalError::ClassNotFound {
                    class: format!("(index {}", class_index.0),
                }));
            }
        }
    }

    pub fn get_method(
        &self,
        class_index: ClassRef,
        method_index: usize,
    ) -> Result<&RuntimeMethod, RuntimeError> {
        let class =
            self.classes
                .get(class_index.0)
                .ok_or_else(|| InternalError::ClassNotFound {
                    class: format!("(index {}", class_index.0),
                })?;

        let method =
            class
                .methods
                .get(method_index)
                .ok_or_else(|| InternalError::MethodNotFound {
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
            None => Err(RuntimeError::Internal(InternalError::ThreadNotFound {
                thread_id: thread_ref.0,
            })),
        }
    }

    fn ensure_class_initialized(&mut self, class_ref: ClassRef) -> Result<(), RuntimeError> {
        if self.static_storage.contains_key(&class_ref) {
            return Ok(());
        }

        let static_slot_count = self.get_class(class_ref)?.static_slot_count();
        let storage_ref = self
            .heap_mut()
            .allocate_object(class_ref, static_slot_count);
        self.static_storage.insert(class_ref, storage_ref);

        if let Some(clinit_idx) = self.get_class(class_ref)?.find_method("<clinit>", "()V") {
            let method = self.get_method(class_ref, clinit_idx)?;
            let (max_locals, max_stack) = (method.max_locals, method.max_stack);

            let clinit_thread = self.create_thread();
            let frame = Frame::new(max_locals, max_stack, class_ref, clinit_idx);
            self.get_thread(clinit_thread)?.push_frame(frame);
            Interpreter::run(self, clinit_thread)?;
        }

        Ok(())
    }

    pub fn static_storage_ref(&self, class_ref: ClassRef) -> Result<ObjectRef, RuntimeError> {
        self.static_storage.get(&class_ref).copied().ok_or_else(|| {
            RuntimeError::Internal(InternalError::ClassNotFound {
                class: format!("(statics uninitialized for class index {})", class_ref.0),
            })
        })
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
            .insert(runtime_class.name.clone(), class_ref);
        self.classes.push(runtime_class);

        self.ensure_class_initialized(class_ref)?;

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

    pub fn resolve_virtual_method(
        &self,
        start: ClassRef,
        name: &str,
        descriptor: &str,
    ) -> Result<(ClassRef, usize), RuntimeError> {
        let mut current = Some(start);

        while let Some(class_ref) = current {
            let class = self.get_class(class_ref)?;

            if let Some(index) = class.find_method(name, descriptor) {
                return Ok((class_ref, index));
            }

            current = class.super_class;
        }

        Err(RuntimeError::Internal(InternalError::MethodNotFound {
            class: self.get_class(start)?.name.clone(),
            method: name.to_string(),
        }))
    }

    fn register_synthetic_class(&mut self, name: &str, super_class: Option<ClassRef>) -> ClassRef {
        let mut class = RuntimeClass::new(name.to_string(), super_class);
        class.instance_fields.push(RuntimeField {
            name: "message".to_string(),
            descriptor: "Ljava/lang/String;".to_string(),
            slot: 0,
        });
        self.add_class(class)
    }

    fn ensure_exceptions_registered(&mut self) -> Result<(), RuntimeError> {
        if self.exceptions_registered {
            return Ok(());
        }
        let object_ref = self.resolve_class("java/lang/Object")?;

        let throwable = self.register_synthetic_class("java/lang/Throwable", Some(object_ref));
        let exception = self.register_synthetic_class("java/lang/Exception", Some(throwable));
        let runtime_exception =
            self.register_synthetic_class("java/lang/RuntimeException", Some(exception));
        let _error = self.register_synthetic_class("java/lang/Error", Some(throwable));

        for name in [
            "java/lang/NullPointerException",
            "java/lang/ArrayIndexOutOfBoundsException",
            "java/lang/ArrayStoreException",
            "java/lang/NegativeArraySizeException",
            "java/lang/ArithmeticException",
            "java/lang/ClassCastException",
        ] {
            self.register_synthetic_class(name, Some(runtime_exception));
        }

        self.exceptions_registered = true;
        Ok(())
    }

    pub fn throw(&mut self, class_name: &str, message: Option<&str>) -> RuntimeError {
        if let Err(e) = self.ensure_exceptions_registered() {
            return e;
        }
        let class_ref = match self.resolve_class(class_name) {
            Ok(c) => c,
            Err(e) => return e,
        };
        let slot_count = self
            .get_class(class_ref)
            .map(|c| c.instance_slot_count())
            .unwrap_or(1);
        let obj_ref = self.heap_mut().allocate_object(class_ref, slot_count);

        if let Some(msg) = message {
            if let Ok(class) = self.get_class(class_ref) {
                if let Some(field) = class.find_field("message") {
                    let slot = field.slot;
                    let str_ref = self.heap_mut().allocate_string(msg.to_string());
                    if let Ok(obj) = self.heap_mut().get_object_mut(obj_ref) {
                        obj.fields[slot] = Value::Reference(Some(str_ref));
                    }
                }
            }
        }

        RuntimeError::Thrown(obj_ref)
    }

    pub fn describe_exception(&self, obj_ref: ObjectRef) -> String {
        let Ok(obj) = self.heap().get_object(obj_ref) else {
            return format!("<non-Throwable object {:?}>", obj_ref);
        };
        let class_name = self
            .get_class(obj.class)
            .map(|c| c.name.clone())
            .unwrap_or_default();
        let message = obj.fields.first().and_then(|v| match v {
            Value::Reference(Some(r)) => self.heap().get_string(*r).ok().map(|s| s.to_string()),
            _ => None,
        });
        match message {
            Some(m) => format!("{}: {}", class_name.replace('/', "."), m),
            None => class_name.replace('/', "."),
        }
    }
}
