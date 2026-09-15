use std::{
    collections::HashMap,
    str::FromStr,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicUsize, Ordering},
    },
};

use jdescriptor::TypeDescriptor;

use crate::{
    class::{
        class_file::ClassFile, constant_pool::ConstantPool, method::MethodAccessFlags,
        reader::ClassReader,
    },
    error::{InternalError, RuntimeError},
    vm::{
        classpath::ClassPath,
        frame::Frame,
        heap::{ArrayElementType, Heap},
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

#[derive(Debug, Clone, Copy)]
pub struct ResolvedMethod {
    pub class: ClassRef,
    pub method: usize,
}

#[allow(dead_code)]
pub struct VM {
    classes: RwLock<Vec<Arc<RuntimeClass>>>,
    classes_by_name: RwLock<HashMap<String, ClassRef>>,

    threads: RwLock<HashMap<ThreadRef, Arc<Thread>>>,
    next_thread_id: AtomicUsize,
    main_thread: RwLock<ThreadRef>,

    classpath: Mutex<ClassPath>,
    heap: Heap,

    static_storage: RwLock<HashMap<ClassRef, ObjectRef>>,
    class_objects: RwLock<HashMap<ClassRef, ObjectRef>>,
    primitive_classes: PrimitiveClasses,
    string_pool: RwLock<HashMap<String, ObjectRef>>,
    virtual_method_cache: RwLock<HashMap<(ClassRef, String, String), ResolvedMethod>>,

    thread_objects: RwLock<HashMap<ThreadRef, ObjectRef>>,
    thread_by_object: RwLock<HashMap<ObjectRef, ThreadRef>>,

    main_thread_group: RwLock<Option<ObjectRef>>,
}

#[allow(dead_code)]
impl VM {
    pub fn new() -> Self {
        let mut classes: Vec<Arc<RuntimeClass>> = Vec::new();
        let mut classes_by_name: HashMap<String, ClassRef> = HashMap::new();

        let register = |classes: &mut Vec<Arc<RuntimeClass>>,
                        classes_by_name: &mut HashMap<String, ClassRef>,
                        name: &str|
         -> ClassRef {
            let class_ref = ClassRef(classes.len());
            classes.push(Arc::new(RuntimeClass::primitive(name)));
            classes_by_name.insert(name.to_string(), class_ref);
            class_ref
        };

        let boolean = register(&mut classes, &mut classes_by_name, "boolean");
        let byte = register(&mut classes, &mut classes_by_name, "byte");
        let char = register(&mut classes, &mut classes_by_name, "char");
        let short = register(&mut classes, &mut classes_by_name, "short");
        let int = register(&mut classes, &mut classes_by_name, "int");
        let long = register(&mut classes, &mut classes_by_name, "long");
        let float = register(&mut classes, &mut classes_by_name, "float");
        let double = register(&mut classes, &mut classes_by_name, "double");
        let void = register(&mut classes, &mut classes_by_name, "void");

        Self {
            classes: RwLock::new(classes),
            classes_by_name: RwLock::new(classes_by_name),
            threads: RwLock::new(HashMap::new()),
            next_thread_id: AtomicUsize::new(0),
            main_thread: RwLock::new(ThreadRef(0)),
            classpath: Mutex::new(ClassPath::empty()),
            heap: Heap::new(),
            static_storage: RwLock::new(HashMap::new()),
            class_objects: RwLock::new(HashMap::new()),
            primitive_classes: PrimitiveClasses {
                boolean,
                byte,
                char,
                short,
                int,
                long,
                float,
                double,
                void,
            },
            string_pool: RwLock::new(HashMap::new()),
            virtual_method_cache: RwLock::new(HashMap::new()),
            thread_objects: RwLock::new(HashMap::new()),
            thread_by_object: RwLock::new(HashMap::new()),
            main_thread_group: RwLock::new(None),
        }
    }

    pub fn main_thread(&self) -> ThreadRef {
        *self.main_thread.read().unwrap()
    }

    pub fn main_thread_group(&self) -> Result<ObjectRef, RuntimeError> {
        if let Some(existing) = *self.main_thread_group.read().unwrap() {
            return Ok(existing);
        }

        let group_class = self.resolve_class("java/lang/ThreadGroup")?;
        self.ensure_class_initialized(group_class)?;

        let defaults = self.default_field_values(group_class)?;
        let obj_ref = self.heap().allocate_object_typed(group_class, &defaults);

        if let Some((_, slot)) = self.find_instance_field(group_class, "name")? {
            let name_ref = self.allocate_string("main")?;
            self.heap().with_object_mut(obj_ref, |o| {
                o.fields[slot] = Value::Reference(Some(name_ref));
                Ok(())
            })?;
        }
        if let Some((_, slot)) = self.find_instance_field(group_class, "maxPriority")? {
            self.heap().with_object_mut(obj_ref, |o| {
                o.fields[slot] = Value::Int(10);
                Ok(())
            })?;
        }

        let mut guard = self.main_thread_group.write().unwrap();
        if let Some(existing) = *guard {
            return Ok(existing);
        }
        *guard = Some(obj_ref);
        Ok(obj_ref)
    }

    pub fn thread_object_for(&self, thread_ref: ThreadRef) -> Result<ObjectRef, RuntimeError> {
        if let Some(existing) = self.thread_objects.read().unwrap().get(&thread_ref) {
            return Ok(*existing);
        }

        let thread_class = self.resolve_class("java/lang/Thread")?;
        self.ensure_class_initialized(thread_class)?;

        let defaults = self.default_field_values(thread_class)?;
        let obj_ref = self.heap().allocate_object_typed(thread_class, &defaults);

        let thread_handle = self.get_thread(thread_ref)?;

        if let Some((_, slot)) = self.find_instance_field(thread_class, "name")? {
            let name_ref = self.allocate_string(thread_handle.name())?;
            self.heap().with_object_mut(obj_ref, |o| {
                o.fields[slot] = Value::Reference(Some(name_ref));
                Ok(())
            })?;
        }

        if let Some((_, slot)) = self.find_instance_field(thread_class, "priority")? {
            let priority = thread_handle.priority();
            self.heap().with_object_mut(obj_ref, |o| {
                o.fields[slot] = Value::Int(priority as i32);
                Ok(())
            })?;
        }

        let group_ref = self.main_thread_group()?;
        if let Some((_, slot)) = self.find_instance_field(thread_class, "group")? {
            self.heap().with_object_mut(obj_ref, |o| {
                o.fields[slot] = Value::Reference(Some(group_ref));
                Ok(())
            })?;
        }

        let mut objects = self.thread_objects.write().unwrap();
        let mut by_object = self.thread_by_object.write().unwrap();
        if let Some(existing) = objects.get(&thread_ref) {
            return Ok(*existing); // lost the race
        }
        objects.insert(thread_ref, obj_ref);
        by_object.insert(obj_ref, thread_ref);

        Ok(obj_ref)
    }

    pub fn thread_ref_from_object(&self, object_ref: ObjectRef) -> Result<ThreadRef, RuntimeError> {
        if let Some(thread_ref) = self.thread_by_object.read().unwrap().get(&object_ref) {
            return Ok(*thread_ref);
        }

        let class_ref = self.runtime_class_of(object_ref)?;
        let thread_class = self.resolve_class("java/lang/Thread")?;

        if !self.is_assignable(class_ref, thread_class)? {
            let class_name = self.get_class(class_ref)?.name.clone();

            return Err(RuntimeError::Internal(InternalError::InvalidHeapEntry {
                expected: "java/lang/Thread",
                found: class_name,
            }));
        }

        let fallback_name = format!("Thread-{}", self.threads.read().unwrap().len());
        let name = if let Some((_, slot)) = self.find_instance_field(class_ref, "name")? {
            let value = self
                .heap()
                .with_object(object_ref, |o| Ok(o.fields.get(slot).cloned()))?;

            match value {
                Some(Value::Reference(Some(name_ref))) => self.java_string_to_rust(name_ref)?,
                _ => fallback_name,
            }
        } else {
            fallback_name
        };

        let thread_ref = self.create_thread(name);

        let mut objects = self.thread_objects.write().unwrap();
        let mut by_object = self.thread_by_object.write().unwrap();
        if let Some(existing) = by_object.get(&object_ref) {
            return Ok(*existing);
        }
        objects.insert(thread_ref, object_ref);
        by_object.insert(object_ref, thread_ref);

        Ok(thread_ref)
    }

    pub fn allocate_string(&self, s: &str) -> Result<ObjectRef, RuntimeError> {
        let string_class = self.resolve_class("java/lang/String")?;
        self.ensure_class_initialized(string_class)?;

        let units: Vec<u16> = s.encode_utf16().collect();

        let array_ref =
            self.heap()
                .allocate_array(string_class, ArrayElementType::Char, units.len());
        self.heap().with_array_mut(array_ref, |array| {
            for (slot, unit) in array.elements.iter_mut().zip(units.iter()) {
                *slot = Value::Int(*unit as i32);
            }
            Ok(())
        })?;

        let defaults = self.default_field_values(string_class)?;
        let obj_ref = self.heap().allocate_object_typed(string_class, &defaults);

        if let Some((_, slot)) = self.find_instance_field(string_class, "value")? {
            self.heap().with_object_mut(obj_ref, |o| {
                o.fields[slot] = Value::Reference(Some(array_ref));
                Ok(())
            })?;
        }

        Ok(obj_ref)
    }

    pub fn intern_string(&self, s: &str) -> Result<ObjectRef, RuntimeError> {
        if let Some(existing) = self.string_pool.read().unwrap().get(s) {
            return Ok(*existing);
        }
        let obj_ref = self.allocate_string(s)?;
        let mut pool = self.string_pool.write().unwrap();
        if let Some(existing) = pool.get(s) {
            return Ok(*existing);
        }
        pool.insert(s.to_string(), obj_ref);
        Ok(obj_ref)
    }

    pub fn java_string_to_rust(&self, r: ObjectRef) -> Result<String, RuntimeError> {
        let string_class = self
            .classes_by_name
            .read()
            .unwrap()
            .get("java/lang/String")
            .copied()
            .ok_or_else(|| {
                RuntimeError::Internal(InternalError::ClassNotFound {
                    class: "java/lang/String".to_string(),
                })
            })?;

        let obj_class = self.heap().class_of(r)?;
        if !self.is_assignable(obj_class, string_class)? {
            let found = self
                .get_class(obj_class)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            return Err(RuntimeError::Internal(InternalError::InvalidHeapEntry {
                expected: "java.lang.String",
                found,
            }));
        }

        let (_, slot) = self
            .find_instance_field(string_class, "value")?
            .ok_or(InternalError::InvalidSlot)?;

        let value = self.heap().with_object(r, |o| Ok(o.fields[slot].clone()))?;

        let array_ref = match value {
            Value::Reference(Some(r)) => r,
            Value::Reference(None) => return Ok(String::new()),
            other => {
                return Err(RuntimeError::Internal(InternalError::InvalidHeapEntry {
                    expected: "char[]",
                    found: format!("{:?}", other),
                }));
            }
        };

        let units: Vec<u16> = self.heap().with_array(array_ref, |array| {
            Ok(array
                .elements
                .iter()
                .map(|v| match v {
                    Value::Int(i) => *i as u16,
                    _ => 0,
                })
                .collect())
        })?;

        Ok(String::from_utf16_lossy(&units))
    }

    pub fn set_classpath(&self, classpath: ClassPath) {
        *self.classpath.lock().unwrap() = classpath;
    }

    pub fn resolve_class(&self, binary_name: &str) -> Result<ClassRef, RuntimeError> {
        if let Some(existing) = self.classes_by_name.read().unwrap().get(binary_name) {
            return Ok(*existing);
        }

        if binary_name.starts_with('[') {
            return self.resolve_array_class(binary_name);
        }

        let data = {
            let mut cp = self.classpath.lock().unwrap();
            cp.find_class(binary_name)
        }
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

    fn resolve_array_class(&self, descriptor: &str) -> Result<ClassRef, RuntimeError> {
        if let Some(existing) = self.classes_by_name.read().unwrap().get(descriptor) {
            return Ok(*existing);
        }

        let object_class = self.resolve_class("java/lang/Object")?;

        let mut classes = self.classes.write().unwrap();
        let mut classes_by_name = self.classes_by_name.write().unwrap();
        if let Some(existing) = classes_by_name.get(descriptor) {
            return Ok(*existing);
        }

        let class_ref = ClassRef(classes.len());
        let class = RuntimeClass {
            name: descriptor.to_string(),
            super_class: Some(object_class),
            methods: Vec::new(),
            constant_pool: ConstantPool {
                entries: Vec::new(),
            },
            instance_fields: Vec::new(),
            static_fields: Vec::new(),
            field_base_slot: 0,
            interfaces: Vec::new(),
        };

        classes.push(Arc::new(class));
        classes_by_name.insert(descriptor.to_string(), class_ref);

        Ok(class_ref)
    }

    pub fn get_class(&self, class_index: ClassRef) -> Result<Arc<RuntimeClass>, RuntimeError> {
        self.classes
            .read()
            .unwrap()
            .get(class_index.0)
            .cloned()
            .ok_or_else(|| {
                RuntimeError::Internal(InternalError::ClassNotFound {
                    class: format!("(index {})", class_index.0),
                })
            })
    }

    pub fn invoke_static_to_completion(
        &self,
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

        let thread = self.create_thread("static-invoke");
        self.get_thread(thread)?.start();
        let frame = Frame::new(max_locals, max_stack, class_ref, method_idx);
        self.get_thread(thread)?.push_frame(frame);
        Interpreter::run(self, thread)
    }

    pub fn get_method(
        &self,
        class_index: ClassRef,
        method_index: usize,
    ) -> Result<RuntimeMethod, RuntimeError> {
        let class = self.get_class(class_index)?;
        class.methods.get(method_index).cloned().ok_or_else(|| {
            RuntimeError::Internal(InternalError::MethodNotFound {
                class: class.name.clone(),
                method: format!("(index {})", method_index),
            })
        })
    }

    pub fn add_class(&self, class: RuntimeClass) -> ClassRef {
        let mut classes = self.classes.write().unwrap();
        let mut classes_by_name = self.classes_by_name.write().unwrap();

        let id = classes.len();
        if let Some(existing) = classes_by_name.insert(class.name.clone(), ClassRef(id)) {
            eprintln!(
                "warning: class `{}` registered twice (old ref {:?}, new ref {:?})",
                class.name,
                existing,
                ClassRef(id)
            );
        }
        classes.push(Arc::new(class));
        ClassRef(id)
    }

    pub fn create_thread(&self, name: impl Into<String>) -> ThreadRef {
        let id = self.next_thread_id.fetch_add(1, Ordering::Relaxed);
        let thread_ref = ThreadRef(id);
        let thread = Arc::new(Thread::new(thread_ref, name));
        self.threads.write().unwrap().insert(thread_ref, thread);
        thread_ref
    }

    pub fn get_thread(&self, thread_ref: ThreadRef) -> Result<Arc<Thread>, RuntimeError> {
        self.threads
            .read()
            .unwrap()
            .get(&thread_ref)
            .cloned()
            .ok_or(RuntimeError::Internal(InternalError::ThreadNotFound {
                thread_id: thread_ref.0,
            }))
    }

    // todo: JLS specifies that all other threads block until <clinit> completes
    pub fn ensure_class_initialized(&self, class_ref: ClassRef) -> Result<(), RuntimeError> {
        if self.static_storage.read().unwrap().contains_key(&class_ref) {
            return Ok(());
        }

        let defaults = self.default_static_field_values(class_ref)?;
        let storage_ref = self.heap().allocate_object_typed(class_ref, &defaults);

        {
            let mut storage = self.static_storage.write().unwrap();
            if storage.contains_key(&class_ref) {
                return Ok(());
            }
            storage.insert(class_ref, storage_ref);
        }

        if let Some(clinit_idx) = self.get_class(class_ref)?.find_method("<clinit>", "()V") {
            let method = self.get_method(class_ref, clinit_idx)?;
            let (max_locals, max_stack) = (method.max_locals, method.max_stack);

            let clinit_thread = self.create_thread("<clinit>");
            self.get_thread(clinit_thread)?.start();
            let frame = Frame::new(max_locals, max_stack, class_ref, clinit_idx);
            self.get_thread(clinit_thread)?.push_frame(frame);
            Interpreter::run(self, clinit_thread)?;
        }

        Ok(())
    }

    pub fn static_storage_ref(&self, class_ref: ClassRef) -> Result<ObjectRef, RuntimeError> {
        self.static_storage
            .read()
            .unwrap()
            .get(&class_ref)
            .copied()
            .ok_or_else(|| {
                RuntimeError::Internal(InternalError::ClassNotFound {
                    class: format!("(statics uninitialized for class index {})", class_ref.0),
                })
            })
    }

    pub fn load_class(&self, class_file: ClassFile) -> Result<ClassRef, RuntimeError> {
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

        let name = class_file
            .constant_pool
            .get_class_name(class_file.this_class)?;

        if let Some(existing) = self.classes_by_name.read().unwrap().get(&name) {
            return Ok(*existing);
        }

        let mut classes = self.classes.write().unwrap();
        let mut classes_by_name = self.classes_by_name.write().unwrap();
        if let Some(existing) = classes_by_name.get(&name) {
            return Ok(*existing); // lost the race
        }

        let id = classes.len();
        let class_ref = ClassRef(id);
        let runtime_class = RuntimeClass::from_class_file(
            &class_file,
            super_class,
            class_ref,
            field_base_slot,
            interfaces,
        )?;

        classes_by_name.insert(runtime_class.name.clone(), class_ref);
        classes.push(Arc::new(runtime_class));

        Ok(class_ref)
    }

    pub fn run_main(&self, main_class: &str) -> Result<Value, RuntimeError> {
        let main_thread = self.create_thread("main");
        *self.main_thread.write().unwrap() = main_thread;
        self.get_thread(main_thread)?.start();

        let system_class = self.resolve_class("java/lang/System")?;
        self.ensure_class_initialized(system_class)?;
        self.invoke_static_to_completion(system_class, "initializeSystemClass", "()V")?;

        let main_class_ref: ClassRef = self.resolve_class(main_class)?;
        let mut main_method_idx: Option<usize> = None;

        for (j, method) in self.get_class(main_class_ref)?.methods.iter().enumerate() {
            if method.name == "main"
                && method.descriptor == "([Ljava/lang/String;)I"
                && method.access.contains(MethodAccessFlags::STATIC)
                && method.access.contains(MethodAccessFlags::PUBLIC)
            {
                main_method_idx = Some(j);
            }
        }

        let method_idx = main_method_idx.unwrap();
        let method = self.get_method(main_class_ref, method_idx)?;
        let (max_locals, max_stack) = (method.max_locals, method.max_stack);

        let frame = Frame::new(max_locals, max_stack, main_class_ref, method_idx);
        self.get_thread(main_thread)?.push_frame(frame);
        Interpreter::run(self, main_thread)
    }

    pub fn heap(&self) -> &Heap {
        &self.heap
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
        let key = (start, name.to_string(), descriptor.to_string());

        if let Some(resolved) = self.virtual_method_cache.read().unwrap().get(&key) {
            return Ok((resolved.class, resolved.method));
        }

        let mut current = Some(start);
        let mut result = None;
        while let Some(class_ref) = current {
            let class = self.get_class(class_ref)?;
            if let Some(index) = class.find_method(name, descriptor) {
                result = Some(ResolvedMethod {
                    class: class_ref,
                    method: index,
                });
                break;
            }
            current = class.super_class;
        }

        let resolved = result.ok_or_else(|| {
            RuntimeError::Internal(InternalError::MethodNotFound {
                class: self
                    .get_class(start)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|_| format!("class {}", start.0)),
                method: name.to_string(),
            })
        })?;

        self.virtual_method_cache
            .write()
            .unwrap()
            .insert(key, resolved);
        Ok((resolved.class, resolved.method))
    }

    pub fn throw(&self, class_name: &str, message: Option<&str>) -> RuntimeError {
        let class_ref = match self.resolve_class(class_name) {
            Ok(c) => c,
            Err(e) => return e,
        };

        #[allow(unused_must_use)]
        self.ensure_class_initialized(class_ref);

        let defaults = self.default_field_values(class_ref).unwrap();
        let obj_ref = self.heap().allocate_object_typed(class_ref, &defaults);

        if let Some(msg) = message {
            if let Ok(Some((_, slot))) = self.find_instance_field(class_ref, "detailMessage") {
                let str_ref = match self.allocate_string(msg) {
                    Ok(r) => r,
                    Err(e) => return e,
                };

                let class_name = self
                    .get_class(class_ref)
                    .map(|c| c.name.clone())
                    .unwrap_or_default();

                let field_count = self
                    .heap()
                    .with_object(obj_ref, |o| Ok(o.fields.len()))
                    .unwrap_or(0);

                let set = self
                    .heap()
                    .with_object_mut(obj_ref, |o| match o.fields.get_mut(slot) {
                        Some(field) => {
                            *field = Value::Reference(Some(str_ref));
                            Ok(true)
                        }
                        None => Ok(false),
                    });

                if matches!(set, Ok(false)) {
                    println!(
                        "warning: detailMessage slot {} out of bounds for {} (has {} fields) — \
                         likely duplicate/stale ClassRef for this class name",
                        slot, class_name, field_count,
                    );
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
                Some(Value::Reference(Some(r))) => self.java_string_to_rust(*r).ok(),
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

    pub fn class_object_for(&self, class_ref: ClassRef) -> ObjectRef {
        if let Some(existing) = self.class_objects.read().unwrap().get(&class_ref) {
            return *existing;
        }

        let class_class = self
            .classes_by_name
            .read()
            .unwrap()
            .get("java/lang/Class")
            .copied()
            .expect("java/lang/Class must be loaded before creating Class objects");

        let defaults = self
            .default_field_values(class_class)
            .expect("java/lang/Class fields must be valid");

        let obj_ref = self
            .heap()
            .allocate_class_object_typed(class_class, class_ref, &defaults);

        let mut guard = self.class_objects.write().unwrap();
        if let Some(existing) = guard.get(&class_ref) {
            return *existing; // lost the race
        }
        guard.insert(class_ref, obj_ref);
        obj_ref
    }

    pub fn runtime_class_of(&self, obj_ref: ObjectRef) -> Result<ClassRef, RuntimeError> {
        self.heap().class_of(obj_ref)
    }

    pub fn invoke_virtual_to_completion(
        &self,
        receiver: ObjectRef,
        method_name: &str,
        descriptor: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let obj_class = self.runtime_class_of(receiver)?;
        let (resolved_class, method_idx) =
            self.resolve_virtual_method(obj_class, method_name, descriptor)?;
        let method = self.get_method(resolved_class, method_idx)?;
        let (max_locals, max_stack) = (method.max_locals, method.max_stack);

        let thread = self.create_thread("virtual-invoke");
        self.get_thread(thread)?.start();
        let mut frame = Frame::new(max_locals, max_stack, resolved_class, method_idx);
        frame.locals[0] = Value::Reference(Some(receiver));

        let mut slot = 1;
        for arg in args {
            let width = matches!(arg, Value::Long(_) | Value::Double(_)) as usize + 1;
            frame.locals[slot] = arg;
            slot += width;
        }

        self.get_thread(thread)?.push_frame(frame);
        Interpreter::run(self, thread)
    }
}
