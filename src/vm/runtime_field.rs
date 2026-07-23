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
    pub fn from_field_infos(
        fields: &[FieldInfo],
        constant_pool: &ConstantPool,
    ) -> Result<Vec<Self>, RuntimeError> {
        let mut out = Vec::new();
        let mut slot = 0;
        for f in fields {
            if f.access_flags.contains(FieldAccessFlags::STATIC) {
                continue;
            }
            let name = constant_pool.get_utf8(f.name_index)?;
            let descriptor = constant_pool.get_utf8(f.descriptor_index)?;
            let width = if descriptor == "J" || descriptor == "D" {
                2
            } else {
                1
            };
            out.push(Self {
                name,
                descriptor,
                slot,
            });
            slot += width;
        }
        Ok(out)
    }
}
