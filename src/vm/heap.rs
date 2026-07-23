use crate::vm::{
    runtime_class::ClassRef,
    value::{ObjectRef, Value},
};

#[allow(dead_code)]
pub struct Object {
    pub class: ClassRef,
    pub fields: Vec<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum ArrayElementType {
    // newarray
    Boolean = 4,
    Char = 5,
    Float = 6,
    Double = 7,
    Byte = 8,
    Short = 9,
    Int = 10,
    Long = 11,

    // anewarray
    Reference = 100,
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

            100 => Ok(ArrayElementType::Reference),
            _ => Err(()),
        }
    }
}

#[allow(dead_code)]
pub struct ArrayObject {
    pub element_type: ArrayElementType,
    pub elements: Vec<Value>,
}

pub enum HeapEntry {
    Object(Object),
    Array(ArrayObject),
}

#[allow(dead_code)]
pub struct Heap {
    entries: Vec<HeapEntry>,
}

#[allow(dead_code)]
impl Heap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn allocate_object(&mut self, class: ClassRef, field_slot_count: usize) -> ObjectRef {
        let id = self.entries.len();
        self.entries.push(HeapEntry::Object(Object {
            class,
            fields: vec![Value::Empty; field_slot_count],
        }));
        ObjectRef(id)
    }

    pub fn allocate_array(&mut self, element_type: ArrayElementType, length: usize) -> ObjectRef {
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
            ArrayElementType::Reference => Value::Reference(None),
        };
        self.entries.push(HeapEntry::Array(ArrayObject {
            element_type,
            elements: vec![fill; length],
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
            _ => Err(crate::error::RuntimeError::InvalidType),
        }
    }

    pub fn get_object(&self, r: ObjectRef) -> Result<&Object, crate::error::RuntimeError> {
        match &self.entries[r.0] {
            HeapEntry::Object(o) => Ok(o),
            _ => Err(crate::error::RuntimeError::InvalidType),
        }
    }

    pub fn get_array_mut(
        &mut self,
        r: ObjectRef,
    ) -> Result<&mut ArrayObject, crate::error::RuntimeError> {
        match &mut self.entries[r.0] {
            HeapEntry::Array(a) => Ok(a),
            _ => Err(crate::error::RuntimeError::InvalidType),
        }
    }

    pub fn get_object_mut(
        &mut self,
        r: ObjectRef,
    ) -> Result<&mut Object, crate::error::RuntimeError> {
        match &mut self.entries[r.0] {
            HeapEntry::Object(o) => Ok(o),
            _ => Err(crate::error::RuntimeError::InvalidType),
        }
    }
}
