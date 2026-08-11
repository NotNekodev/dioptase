use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use crate::{
    error::{InternalError, RuntimeError},
    vm::{
        runtime_class::ClassRef,
        value::{ObjectRef, Value},
    },
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Object {
    pub class: ClassRef,
    pub fields: Vec<Value>,

    pub class_object: Option<ClassRef>,
    pub hash_code: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ArrayElementType {
    // newarray
    Boolean,
    Char,
    Float,
    Double,
    Byte,
    Short,
    Int,
    Long,

    // anewarray
    Reference(ClassRef),
}

impl ArrayElementType {
    pub fn descriptor(&self) -> &'static str {
        match self {
            Self::Boolean => "Z",
            Self::Byte => "B",
            Self::Char => "C",
            Self::Short => "S",
            Self::Int => "I",
            Self::Long => "J",
            Self::Float => "F",
            Self::Double => "D",
            Self::Reference(_) => unreachable!(),
        }
    }
}

impl TryFrom<u8> for ArrayElementType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            4 => Ok(ArrayElementType::Boolean),
            5 => Ok(ArrayElementType::Char),
            6 => Ok(ArrayElementType::Float),
            7 => Ok(ArrayElementType::Double),
            8 => Ok(ArrayElementType::Byte),
            9 => Ok(ArrayElementType::Short),
            10 => Ok(ArrayElementType::Int),
            11 => Ok(ArrayElementType::Long),

            _ => Err(()),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArrayObject {
    pub element_type: ArrayElementType,
    pub elements: Vec<Value>,
    pub class: ClassRef,
}

#[derive(Debug, Clone)]
pub enum HeapEntry {
    Object(Object),
    Array(ArrayObject),
}

type Slot = Arc<Mutex<HeapEntry>>;

#[allow(dead_code)]
pub struct Heap {
    entries: RwLock<Vec<Slot>>,
    next_identity_hash: AtomicI32,
}

#[allow(dead_code)]
impl Heap {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            next_identity_hash: AtomicI32::new(1),
        }
    }

    fn next_hash(&self) -> i32 {
        self.next_identity_hash.fetch_add(1, Ordering::Relaxed)
    }

    fn push_entry(&self, entry: HeapEntry) -> ObjectRef {
        let slot = Arc::new(Mutex::new(entry));
        let mut entries = self.entries.write().unwrap();
        let id = entries.len();
        entries.push(slot);
        ObjectRef(id)
    }

    fn slot(&self, r: ObjectRef) -> Result<Slot, RuntimeError> {
        self.entries
            .read()
            .unwrap()
            .get(r.0)
            .cloned()
            .ok_or_else(|| {
                RuntimeError::Internal(InternalError::InvalidHeapEntry {
                    expected: "valid ObjectRef",
                    found: format!("out-of-bounds {:?}", r),
                })
            })
    }

    pub fn allocate_object(&self, class: ClassRef, field_slot_count: usize) -> ObjectRef {
        let hash_code = self.next_hash();
        self.push_entry(HeapEntry::Object(Object {
            class,
            fields: vec![Value::Empty; field_slot_count],
            class_object: None,
            hash_code,
        }))
    }

    pub fn allocate_object_typed(&self, class: ClassRef, field_defaults: &[Value]) -> ObjectRef {
        let hash_code = self.next_hash();
        self.push_entry(HeapEntry::Object(Object {
            class,
            fields: field_defaults.to_vec(),
            class_object: None,
            hash_code,
        }))
    }

    pub fn allocate_array(
        &self,
        class: ClassRef,
        element_type: ArrayElementType,
        length: usize,
    ) -> ObjectRef {
        let fill = match element_type {
            ArrayElementType::Boolean
            | ArrayElementType::Char
            | ArrayElementType::Byte
            | ArrayElementType::Short
            | ArrayElementType::Int => Value::Int(0),
            ArrayElementType::Float => Value::Float(0.0),
            ArrayElementType::Double => Value::Double(0.0),
            ArrayElementType::Long => Value::Long(0),
            ArrayElementType::Reference(_) => Value::Reference(None),
        };
        self.push_entry(HeapEntry::Array(ArrayObject {
            element_type,
            elements: vec![fill; length],
            class,
        }))
    }

    pub fn allocate_class_object(
        &self,
        class_class: ClassRef,
        represented_class: ClassRef,
        field_slot_count: usize,
    ) -> ObjectRef {
        let hash_code = self.next_hash();
        self.push_entry(HeapEntry::Object(Object {
            class: class_class,
            fields: vec![Value::Empty; field_slot_count],
            class_object: Some(represented_class),
            hash_code,
        }))
    }

    pub fn allocate_class_object_typed(
        &self,
        class_class: ClassRef,
        represented_class: ClassRef,
        field_defaults: &[Value],
    ) -> ObjectRef {
        let hash_code = self.next_hash();
        self.push_entry(HeapEntry::Object(Object {
            class: class_class,
            fields: field_defaults.to_vec(),
            class_object: Some(represented_class),
            hash_code,
        }))
    }

    pub fn with_object<T>(
        &self,
        r: ObjectRef,
        f: impl FnOnce(&Object) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let slot = self.slot(r)?;
        let guard = slot.lock().unwrap();
        match &*guard {
            HeapEntry::Object(o) => f(o),
            other => Err(RuntimeError::Internal(InternalError::InvalidHeapEntry {
                expected: "HeapEntry::Object",
                found: format!("{:?}", other),
            })),
        }
    }

    pub fn with_object_mut<T>(
        &self,
        r: ObjectRef,
        f: impl FnOnce(&mut Object) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let slot = self.slot(r)?;
        let mut guard = slot.lock().unwrap();
        match &mut *guard {
            HeapEntry::Object(o) => f(o),
            other => Err(RuntimeError::Internal(InternalError::InvalidHeapEntry {
                expected: "HeapEntry::Object",
                found: format!("{:?}", other),
            })),
        }
    }

    pub fn with_array<T>(
        &self,
        r: ObjectRef,
        f: impl FnOnce(&ArrayObject) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let slot = self.slot(r)?;
        let guard = slot.lock().unwrap();
        match &*guard {
            HeapEntry::Array(a) => f(a),
            other => Err(RuntimeError::Internal(InternalError::InvalidHeapEntry {
                expected: "HeapEntry::Array",
                found: format!("{:?}", other),
            })),
        }
    }

    pub fn with_array_mut<T>(
        &self,
        r: ObjectRef,
        f: impl FnOnce(&mut ArrayObject) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        let slot = self.slot(r)?;
        let mut guard = slot.lock().unwrap();
        match &mut *guard {
            HeapEntry::Array(a) => f(a),
            other => Err(RuntimeError::Internal(InternalError::InvalidHeapEntry {
                expected: "HeapEntry::Array",
                found: format!("{:?}", other),
            })),
        }
    }

    pub fn get_object(&self, r: ObjectRef) -> Result<Object, RuntimeError> {
        self.with_object(r, |o| Ok(o.clone()))
    }

    pub fn get_array(&self, r: ObjectRef) -> Result<ArrayObject, RuntimeError> {
        self.with_array(r, |a| Ok(a.clone()))
    }

    pub fn get_class_object(&self, r: ObjectRef) -> Result<ClassRef, RuntimeError> {
        self.with_object(r, |o| {
            o.class_object.ok_or_else(|| {
                RuntimeError::Internal(InternalError::InvalidHeapEntry {
                    expected: "java.lang.Class object",
                    found: format!("{:?}", o),
                })
            })
        })
    }

    pub fn class_of(&self, r: ObjectRef) -> Result<ClassRef, RuntimeError> {
        let slot = self.slot(r)?;
        let guard = slot.lock().unwrap();
        Ok(match &*guard {
            HeapEntry::Object(o) => o.class,
            HeapEntry::Array(a) => a.class,
        })
    }
}
