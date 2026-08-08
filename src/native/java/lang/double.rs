use dioptase_native_macros::native;

use crate::{
    error::{InternalError, RuntimeError},
    native::native_context::NativeContext,
    vm::value::Value,
};

#[native(
    class = "java/lang/Double",
    name = "doubleToRawLongBits",
    descriptor = "(D)J"
)]
pub fn double_to_raw_long_bits(
    _ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let value = match args.first() {
        Some(Value::Double(value)) => *value,
        other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                class: "java/lang/Double".to_string(),
                method: "doubleToRawLongBits".to_string(),
                pc: 0xDEADBEEF,
                expected: "Double".to_string(),
                found: format!("{:?}", other),
            }));
        }
    };

    Ok(Some(Value::Long(value.to_bits() as i64)))
}
