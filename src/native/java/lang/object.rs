use crate::{error::RuntimeError, native::native_context::NativeContext, vm::value::Value};
use dioptase_native_macros::native;

#[native(
    class = "java/lang/Object",
    name = "registerNatives",
    descriptor = "()V"
)]
pub fn register_natives(
    _ctx: &mut NativeContext,
    _args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    Ok(None)
}
