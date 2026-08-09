use crate::{
    error::RuntimeError,
    native::native_context::NativeContext,
    vm::value::Value::{self},
};
use dioptase_native_macros::native;

#[native(class = "sun/misc/VM", name = "initialize", descriptor = "()V")]
pub fn initialize(
    _ctx: &mut NativeContext,
    _args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    Ok(None)
}
