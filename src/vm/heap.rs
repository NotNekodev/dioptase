use crate::{
    error::RuntimeError,
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

            100 => Err(()),
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

#[allow(dead_code)]
#[derive(Debug)]
pub struct Heap {
    entries: Vec<HeapEntry>,
    next_identity_hash: i32,
}

#[allow(dead_code)]
impl Heap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_identity_hash: 1,
        }
    }

    pub fn allocate_object(&mut self, class: ClassRef, field_slot_count: usize) -> ObjectRef {
        let id = self.entries.len();

        self.entries.push(HeapEntry::Object(Object {
            class,
            fields: vec![Value::Empty; field_slot_count],
            class_object: None,
            hash_code: self.next_identity_hash,
        }));

        self.next_identity_hash = self.next_identity_hash.wrapping_add(1);

        ObjectRef(id)
    }

    pub fn allocate_object_typed(
        &mut self,
        class: ClassRef,
        field_defaults: &[Value],
    ) -> ObjectRef {
        let id = self.entries.len();

        self.entries.push(HeapEntry::Object(Object {
            class,
            fields: field_defaults.to_vec(),
            class_object: None,
            hash_code: self.next_identity_hash,
        }));

        self.next_identity_hash = self.next_identity_hash.wrapping_add(1);

        ObjectRef(id)
    }

    pub fn allocate_array(
        &mut self,
        class: ClassRef,
        element_type: ArrayElementType,
        length: usize,
    ) -> ObjectRef {
        let id = self.entries.len();
        let fill = match element_type {
            ArrayElementType::Boolean => Value::Int(0),
            ArrayElementType::Char => Value::Int(0),
            ArrayElementType::Float => Value::Float(0.0),
            ArrayElementType::Double => Value::Double(0.0),
            ArrayElementType::Byte => Value::Int(0),
            ArrayElementType::Short => Value::Int(0),
            ArrayElementType::Int => Value::Int(0),
            ArrayElementType::Long => Value::Long(0),
            ArrayElementType::Reference(_) => Value::Reference(None),
        };
        self.entries.push(HeapEntry::Array(ArrayObject {
            element_type,
            elements: vec![fill; length],
            class: class,
        }));
        ObjectRef(id)
    }

    pub fn get(&self, r: ObjectRef) -> &HeapEntry {
        &self.entries[r.0]
    }
    pub fn get_mut(&mut self, r: ObjectRef) -> &mut HeapEntry {
        &mut self.entries[r.0]
    }

    pub fn get_array(&self, r: ObjectRef) -> Result<&ArrayObject, crate::error::RuntimeError> {
        match &self.entries[r.0] {
            HeapEntry::Array(a) => Ok(a),
            other => Err(RuntimeError::Internal(
                crate::error::InternalError::InvalidHeapEntry {
                    expected: "HeapEntry::Array",
                    found: format!("{:?}", other),
                },
            )),
        }
    }

    pub fn get_object(&self, r: ObjectRef) -> Result<&Object, crate::error::RuntimeError> {
        match &self.entries[r.0] {
            HeapEntry::Object(o) => Ok(o),
            other => Err(RuntimeError::Internal(
                crate::error::InternalError::InvalidHeapEntry {
                    expected: "HeapEntry::Object",
                    found: format!("{:?}", other),
                },
            )),
        }
    }

    pub fn get_array_mut(
        &mut self,
        r: ObjectRef,
    ) -> Result<&mut ArrayObject, crate::error::RuntimeError> {
        match &mut self.entries[r.0] {
            HeapEntry::Array(a) => Ok(a),
            other => Err(RuntimeError::Internal(
                crate::error::InternalError::InvalidHeapEntry {
                    expected: "HeapEntry::Array",
                    found: format!("{:?}", other),
                },
            )),
        }
    }

    pub fn get_object_mut(
        &mut self,
        r: ObjectRef,
    ) -> Result<&mut Object, crate::error::RuntimeError> {
        match &mut self.entries[r.0] {
            HeapEntry::Object(o) => Ok(o),
            other => Err(RuntimeError::Internal(
                crate::error::InternalError::InvalidHeapEntry {
                    expected: "HeapEntry::Object",
                    found: format!("{:?}", other),
                },
            )),
        }
    }

    pub fn allocate_class_object(
        &mut self,
        class_class: ClassRef,
        represented_class: ClassRef,
        field_slot_count: usize,
    ) -> ObjectRef {
        let id = self.entries.len();

        self.entries.push(HeapEntry::Object(Object {
            class: class_class,
            fields: vec![Value::Empty; field_slot_count],
            class_object: Some(represented_class),
            hash_code: self.next_identity_hash,
        }));

        self.next_identity_hash = self.next_identity_hash.wrapping_add(1);

        ObjectRef(id)
    }

    pub fn get_class_object(&self, r: ObjectRef) -> Result<ClassRef, RuntimeError> {
        match self.entries[r.0].clone() {
            HeapEntry::Object(ref object) => object.class_object.ok_or_else(|| {
                RuntimeError::Internal(crate::error::InternalError::InvalidHeapEntry {
                    expected: "java.lang.Class object",
                    found: format!("{:?}", object),
                })
            }),

            other => Err(RuntimeError::Internal(
                crate::error::InternalError::InvalidHeapEntry {
                    expected: "java.lang.Class object",
                    found: format!("{:?}", other),
                },
            )),
        }
    }

    pub fn allocate_class_object_typed(
        &mut self,
        class_class: ClassRef,
        represented_class: ClassRef,
        field_defaults: &[Value],
    ) -> ObjectRef {
        let id = self.entries.len();
        self.entries.push(HeapEntry::Object(Object {
            class: class_class,
            fields: field_defaults.to_vec(),
            class_object: Some(represented_class),
            hash_code: self.next_identity_hash,
        }));

        self.next_identity_hash = self.next_identity_hash.wrapping_add(1);

        ObjectRef(id)
    }
}
