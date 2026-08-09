use crate::{error::RuntimeError, native::native_context::NativeContext, vm::value::Value};
use dioptase_native_macros::native;

#[native(
    class = "java/io/FileOutputStream",
    name = "initIDs",
    descriptor = "()V"
)]
pub fn init_ids(_ctx: &mut NativeContext, _args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    Ok(None)
}
