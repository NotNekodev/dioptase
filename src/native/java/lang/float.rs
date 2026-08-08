use dioptase_native_macros::native;

use crate::{
    error::{InternalError, RuntimeError},
    native::native_context::NativeContext,
    vm::value::Value,
};

#[native(
    class = "java/lang/Float",
    name = "floatToRawIntBits",
    descriptor = "(F)I"
)]
pub fn float_to_raw_int_bits(
    _ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let value = match args.first() {
        Some(Value::Float(value)) => *value,
        other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                class: "java/lang/Float".to_string(),
                method: "floatToRawIntBits".to_string(),
                pc: 0xDEADBEEF,
                expected: "Float".to_string(),
                found: format!("{:?}", other),
            }));
        }
    };

    Ok(Some(Value::Int(value.to_bits() as i32)))
}
