use crate::class::{
    constant_pool::ConstantPool,
    field::{FieldAccessFlags, FieldInfo},
};
use crate::error::RuntimeError;

#[allow(dead_code)]
pub struct RuntimeField {
    pub name: String,
    pub descriptor: String,
    pub slot: usize,
}

impl RuntimeField {
    pub fn partition_field_infos(
        fields: &[FieldInfo],
        constant_pool: &ConstantPool,
    ) -> Result<(Vec<Self>, Vec<Self>), RuntimeError> {
        let mut instance_fields = Vec::new();
        let mut static_fields = Vec::new();
        let mut instance_slot = 0;
        let mut static_slot = 0;

        for f in fields {
            let name = constant_pool.get_utf8(f.name_index)?;
            let descriptor = constant_pool.get_utf8(f.descriptor_index)?;
            let width = if descriptor == "J" || descriptor == "D" {
                2
            } else {
                1
            };

            if f.access_flags.contains(FieldAccessFlags::STATIC) {
                static_fields.push(Self {
                    name,
                    descriptor,
                    slot: static_slot,
                });
                static_slot += width;
            } else {
                instance_fields.push(Self {
                    name,
                    descriptor,
                    slot: instance_slot,
                });
                instance_slot += width;
            }
        }

        Ok((instance_fields, static_fields))
    }
}
