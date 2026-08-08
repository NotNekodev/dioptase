use std::{collections::HashMap, str::FromStr};

use jdescriptor::TypeDescriptor;

use crate::{
    class::{class_file::ClassFile, method::MethodAccessFlags, reader::ClassReader},
    error::{InternalError, RuntimeError},
    vm::{
        classpath::ClassPath,
        frame::Frame,
        heap::{Heap, HeapEntry},
        interpreter::Interpreter,
        runtime_class::{ClassRef, RuntimeClass},
        runtime_method::RuntimeMethod,
        thread::{Thread, ThreadRef},
        value::{ObjectRef, Value},
    },
};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct PrimitiveClasses {
    pub boolean: ClassRef,
    pub byte: ClassRef,
    pub char: ClassRef,
    pub short: ClassRef,
    pub int: ClassRef,
    pub long: ClassRef,
    pub float: ClassRef,
    pub double: ClassRef,
    pub void: ClassRef,
}

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
    class_objects: HashMap<ClassRef, ObjectRef>,
    primitive_classes: PrimitiveClasses,
}

#[allow(dead_code)]
impl VM {
    pub fn new() -> Self {
        let mut vm = Self {
            classes: Vec::new(),
            threads: Vec::new(),
            main_thread: ThreadRef(0),
            classes_by_name: HashMap::new(),
            classpath: ClassPath::empty(),
            heap: Heap::new(),
            static_storage: HashMap::new(),
            exceptions_registered: false,
            class_objects: HashMap::new(),

            primitive_classes: PrimitiveClasses {
                boolean: ClassRef(usize::MAX),
                byte: ClassRef(usize::MAX),
                char: ClassRef(usize::MAX),
                short: ClassRef(usize::MAX),
                int: ClassRef(usize::MAX),
                long: ClassRef(usize::MAX),
                float: ClassRef(usize::MAX),
                double: ClassRef(usize::MAX),
                void: ClassRef(usize::MAX),
            },
        };

        vm.bootstrap_primitives();

        vm
    }

    fn register_primitive_class(&mut self, name: &str) -> ClassRef {
        let class_ref = ClassRef(self.classes.len());

        let class = RuntimeClass::primitive(name);

        self.classes.push(class);
        self.classes_by_name.insert(name.to_string(), class_ref);

        class_ref
    }

    fn bootstrap_primitives(&mut self) {
        let boolean = self.register_primitive_class("boolean");
        let byte = self.register_primitive_class("byte");
        let char = self.register_primitive_class("char");
        let short = self.register_primitive_class("short");
        let int = self.register_primitive_class("int");
        let long = self.register_primitive_class("long");
        let float = self.register_primitive_class("float");
        let double = self.register_primitive_class("double");
        let void = self.register_primitive_class("void");

        self.primitive_classes = PrimitiveClasses {
            boolean,
            byte,
            char,
            short,
            int,
            long,
            float,
            double,
            void,
        };
    }

    pub fn set_classpath(&mut self, classpath: ClassPath) {
        self.classpath = classpath;
    }

    pub fn resolve_class(&mut self, binary_name: &str) -> Result<ClassRef, RuntimeError> {
        if let Some(existing) = self.classes_by_name.get(binary_name) {
            return Ok(*existing);
        }

        let data =
            self.classpath
                .find_class(binary_name)
                .ok_or_else(|| InternalError::ClassNotFound {
                    class: binary_name.to_string(),
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

    pub fn invoke_static_to_completion(
        &mut self,
        class_ref: ClassRef,
        name: &str,
        descriptor: &str,
    ) -> Result<Value, RuntimeError> {
        let method_idx = self
            .get_class(class_ref)?
            .find_method(name, descriptor)
            .ok_or_else(|| InternalError::MethodNotFound {
                class: self.get_class(class_ref).unwrap().name.clone(),
                method: name.to_string(),
            })?;

        let method = self.get_method(class_ref, method_idx)?;
        let (max_locals, max_stack) = (method.max_locals, method.max_stack);

        let thread = self.create_thread();
        let frame = Frame::new(max_locals, max_stack, class_ref, method_idx);
        self.get_thread(thread)?.push_frame(frame);
        Interpreter::run(self, thread)
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
        if let Some(existing) = self
            .classes_by_name
            .insert(class.name.clone(), ClassRef(id))
        {
            println!(
                "warning: class `{}` registered twice (old ref {:?}, new ref {:?})",
                class.name,
                existing,
                ClassRef(id)
            );
        }
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

    pub fn ensure_class_initialized(&mut self, class_ref: ClassRef) -> Result<(), RuntimeError> {
        if self.static_storage.contains_key(&class_ref) {
            return Ok(());
        }

        let defaults = self.default_static_field_values(class_ref)?;
        let storage_ref = self.heap_mut().allocate_object_typed(class_ref, &defaults);
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

        let mut interfaces = Vec::with_capacity(class_file.interfaces.len());
        for &iface_index in &class_file.interfaces {
            let iface_name = class_file.constant_pool.get_class_name(iface_index)?;
            interfaces.push(self.resolve_class(&iface_name)?);
        }

        let field_base_slot = match super_class {
            Some(sc) => self.get_class(sc)?.total_instance_slot_count(),
            None => 0,
        };

        let id = self.classes.len();
        let class_ref = ClassRef(id);
        let runtime_class = RuntimeClass::from_class_file(
            &class_file,
            super_class,
            class_ref,
            field_base_slot,
            interfaces,
        )?;

        self.classes_by_name
            .insert(runtime_class.name.clone(), class_ref);
        self.classes.push(runtime_class);

        Ok(class_ref)
    }

    pub fn run_main(&mut self, main_class: &str) -> Result<Value, RuntimeError> {
        let main_thread = self.create_thread();

        let system_class = self.resolve_class("java/lang/System")?;
        self.ensure_class_initialized(system_class)?;
        self.invoke_static_to_completion(system_class, "initializeSystemClass", "()V")?;

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

    pub fn primitive_classes(&self) -> &PrimitiveClasses {
        &self.primitive_classes
    }

    pub fn is_assignable(&self, from: ClassRef, to: ClassRef) -> Result<bool, RuntimeError> {
        if from == to {
            return Ok(true);
        }

        let mut current = Some(from);

        while let Some(class_ref) = current {
            if class_ref == to {
                return Ok(true);
            }
            let class = self.get_class(class_ref)?;
            for &iface in &class.interfaces {
                if self.interface_extends(iface, to)? {
                    return Ok(true);
                }
            }
            current = class.super_class;
        }

        Ok(false)
    }

    fn interface_extends(&self, iface: ClassRef, target: ClassRef) -> Result<bool, RuntimeError> {
        if iface == target {
            return Ok(true);
        }

        let class = self.get_class(iface)?;

        for &super_iface in &class.interfaces {
            if self.interface_extends(super_iface, target)? {
                return Ok(true);
            }
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

    pub fn throw(&mut self, class_name: &str, message: Option<&str>) -> RuntimeError {
        let class_ref = match self.resolve_class(class_name) {
            Ok(c) => c,
            Err(e) => return e,
        };

        #[allow(unused_must_use)]
        self.ensure_class_initialized(class_ref);

        let defaults = self.default_field_values(class_ref).unwrap();
        let obj_ref = self.heap_mut().allocate_object_typed(class_ref, &defaults);

        if let Some(msg) = message {
            if let Ok(Some((_, slot))) = self.find_instance_field(class_ref, "detailMessage") {
                let str_ref = self.heap_mut().allocate_string(msg.to_string());

                let class_name = self
                    .get_class(class_ref)
                    .map(|c| c.name.clone())
                    .unwrap_or_default();

                if let Ok(obj) = self.heap_mut().get_object_mut(obj_ref) {
                    let field_count = obj.fields.len();

                    match obj.fields.get_mut(slot) {
                        Some(field) => *field = Value::Reference(Some(str_ref)),
                        None => {
                            println!(
                                "warning: detailMessage slot {} out of bounds for {} (has {} fields) — \
                                 likely duplicate/stale ClassRef for this class name",
                                slot, class_name, field_count,
                            );
                        }
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

        let message = self
            .find_instance_field(obj.class, "detailMessage")
            .ok()
            .flatten()
            .and_then(|(_, slot)| match obj.fields.get(slot) {
                Some(Value::Reference(Some(r))) => {
                    self.heap().get_string(*r).ok().map(|s| s.to_string())
                }
                _ => None,
            });

        match message {
            Some(m) => format!("{}: {}", class_name.replace('/', "."), m),
            None => class_name.replace('/', "."),
        }
    }

    pub fn find_instance_field(
        &self,
        start: ClassRef,
        name: &str,
    ) -> Result<Option<(ClassRef, usize)>, RuntimeError> {
        let mut current = Some(start);
        while let Some(c) = current {
            let class = self.get_class(c)?;
            if let Some(f) = class.instance_fields.iter().find(|f| f.name == name) {
                return Ok(Some((c, f.slot)));
            }
            current = class.super_class;
        }
        Ok(None)
    }

    fn default_value_for_descriptor(&self, descriptor: &str) -> Result<Value, RuntimeError> {
        let type_desc =
            TypeDescriptor::from_str(descriptor).map_err(|_| InternalError::InvalidDescriptor)?;

        Ok(match type_desc {
            TypeDescriptor::Boolean
            | TypeDescriptor::Byte
            | TypeDescriptor::Char
            | TypeDescriptor::Short
            | TypeDescriptor::Integer => Value::Int(0),
            TypeDescriptor::Long => Value::Long(0),
            TypeDescriptor::Float => Value::Float(0.0),
            TypeDescriptor::Double => Value::Double(0.0),
            TypeDescriptor::Object(_) | TypeDescriptor::Array(_, _) => Value::Reference(None),
            TypeDescriptor::Void => Value::Empty,
        })
    }

    pub fn default_field_values(&self, class_ref: ClassRef) -> Result<Vec<Value>, RuntimeError> {
        let total = self.get_class(class_ref)?.total_instance_slot_count();
        let mut fields = vec![Value::Empty; total];
        let mut current = Some(class_ref);
        while let Some(c) = current {
            let class = self.get_class(c)?;
            for f in &class.instance_fields {
                fields[f.slot] = self.default_value_for_descriptor(&f.descriptor)?;
            }
            current = class.super_class;
        }
        Ok(fields)
    }

    pub fn default_static_field_values(
        &self,
        class_ref: ClassRef,
    ) -> Result<Vec<Value>, RuntimeError> {
        let class = self.get_class(class_ref)?;
        let mut fields = vec![Value::Empty; class.static_slot_count()];
        for f in &class.static_fields {
            fields[f.slot] = self.default_value_for_descriptor(&f.descriptor)?;
        }
        Ok(fields)
    }

    pub fn class_object_for(&mut self, class_ref: ClassRef) -> ObjectRef {
        if let Some(existing) = self.class_objects.get(&class_ref) {
            return *existing;
        }

        let class_class = self
            .classes_by_name
            .get("java/lang/Class")
            .copied()
            .expect("java/lang/Class must be loaded before creating Class objects");

        let defaults = self
            .default_field_values(class_class)
            .expect("java/lang/Class fields must be valid");

        let obj_ref =
            self.heap_mut()
                .allocate_class_object_typed(class_class, class_ref, &defaults);

        self.class_objects.insert(class_ref, obj_ref);

        obj_ref
    }

    pub fn runtime_class_of(&mut self, obj_ref: ObjectRef) -> Result<ClassRef, RuntimeError> {
        match self.heap().get(obj_ref) {
            HeapEntry::Object(o) => Ok(o.class),
            HeapEntry::Str(_) => self.resolve_class("java/lang/String"),
            HeapEntry::Array(_) => self.resolve_class("java/lang/Object"),
        }
    }
}
