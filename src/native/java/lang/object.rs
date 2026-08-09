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

#[native(
    class = "java/lang/Object",
    name = "getClass",
    descriptor = "()Ljava/lang/Class;"
)]
pub fn get_class(ctx: &mut NativeContext, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::Internal(InternalError::InvalidType {
            expected: "1 argument to Object.getClass".to_string(),
            found: format!("{} arguments", args.len()),
            class: "java/lang/Object".to_string(),
            method: "getClass".to_string(),
            pc: 0xDEADBEEF,
        }));
    }

    let object_ref = match args[0] {
        Value::Reference(Some(object_ref)) => object_ref,

        Value::Reference(None) => {
            return ctx.throw(
                "java/lang/NullPointerException",
                Some("Cannot invoke Object.getClass() on null"),
            );
        }

        ref other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "object reference".to_string(),
                found: format!("{:?}", other),
                class: "java/lang/Object".to_string(),
                method: "getClass".to_string(),
                pc: 0xDEADBEEF,
            }));
        }
    };

    let runtime_class = ctx.vm_mut().runtime_class_of(object_ref)?;
    let class_object = ctx.vm_mut().class_object_for(runtime_class);

    Ok(Some(Value::Reference(Some(class_object))))
}
