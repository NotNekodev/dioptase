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

#[native(
    class = "java/lang/Double",
    name = "longBitsToDouble",
    descriptor = "(J)D"
)]
pub fn long_bits_to_double(
    _ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let value = match args.first() {
        Some(Value::Long(value)) => *value,
        other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                class: "java/lang/Double".to_string(),
                method: "longBitsToDouble".to_string(),
                pc: 0xDEADBEEF,
                expected: "Long".to_string(),
                found: format!("{:?}", other),
            }));
        }
    };

    Ok(Some(Value::Double(f64::from_bits(value as u64))))
}
