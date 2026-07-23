use crate::vm::{
    runtime_class::ClassRef,
    value::{ObjectRef, Value},
};

#[allow(dead_code)]
pub struct Object {
    pub class: ClassRef,
    pub fields: Vec<Value>,
}

#[allow(dead_code)]
pub struct Heap {
    objects: Vec<Object>,
}

#[allow(dead_code)]
impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn allocate(&mut self, class: ClassRef, field_slot_count: usize) -> ObjectRef {
        let id = self.objects.len();
        self.objects.push(Object {
            class,
            fields: vec![Value::Empty; field_slot_count],
        });
        ObjectRef(id)
    }

    pub fn get(&self, obj_ref: ObjectRef) -> &Object {
        &self.objects[obj_ref.0]
    }

    pub fn get_mut(&mut self, obj_ref: ObjectRef) -> &mut Object {
        &mut self.objects[obj_ref.0]
    }
}
