use crate::{
    error::{
        InternalError,
        RuntimeError::{self, Internal},
    },
    native::native_context::NativeContext,
    vm::value::Value,
};
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

#[native(class = "java/lang/Object", name = "hashCode", descriptor = "()I")]
pub fn hash_code(ctx: &mut NativeContext, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let obj_ref = match args.first() {
        Some(Value::Reference(Some(r))) => *r,
        Some(Value::Reference(None)) => {
            return Err(ctx.vm_mut().throw(
                "java/lang/NullPointerException",
                Some("Object.hashCode() called on null"),
            ));
        }
        other => {
            return Err(Internal(InternalError::InvalidType {
                class: "java.lang.Object".to_string(),
                method: "hashCode".to_string(),
                pc: 0xDEADBEEF,
                expected: "Reference".to_string(),
                found: format!("{:?}", other),
            }));
        }
    };

    let hash = ctx.vm().heap().get_object(obj_ref)?.hash_code;

    Ok(Some(Value::Int(hash)))
}
