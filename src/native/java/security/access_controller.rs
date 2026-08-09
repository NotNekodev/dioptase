use crate::{
    error::{InternalError, RuntimeError},
    native::native_context::NativeContext,
    vm::value::{
        ObjectRef,
        Value::{self, Reference},
    },
};
use dioptase_native_macros::native;

#[native(
    class = "java/security/AccessController",
    name = "doPrivileged",
    descriptor = "(Ljava/security/PrivilegedExceptionAction;)Ljava/lang/Object;"
)]
pub fn do_priviledged(
    ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let action = match args.get(0) {
        Some(Value::Reference(Some(r))) => *r,
        Some(Value::Reference(None)) => {
            ctx.vm_mut().throw("java/lang/NullPointerException", None);
            return Err(RuntimeError::Thrown(ObjectRef(0)));
        }
        other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                class: "java/security/AccessController".to_string(),
                method: "doPrivileged".to_string(),
                pc: 0xDEADBEEF,
                expected: "Reference".to_string(),
                found: format!("{:?}", other),
            }));
        }
    };

    let result = ctx.vm_mut().invoke_virtual_to_completion(
        action,
        "run",
        "()Ljava/lang/Object;",
        Vec::new(),
    )?;

    Ok(Some(result))
}

#[native(
    class = "java/security/AccessController",
    name = "doPrivileged",
    descriptor = "(Ljava/security/PrivilegedAction;)Ljava/lang/Object;"
)]
pub fn do_privileged_action(
    ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let action = match args.get(0) {
        Some(Value::Reference(Some(r))) => *r,
        Some(Value::Reference(None)) => {
            ctx.vm_mut().throw("java/lang/NullPointerException", None);
            return Err(RuntimeError::Thrown(ObjectRef(0)));
        }
        other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                class: "java/security/AccessController".to_string(),
                method: "doPrivileged".to_string(),
                pc: 0xDEADBEEF,
                expected: "Reference".to_string(),
                found: format!("{:?}", other),
            }));
        }
    };

    let result = ctx.vm_mut().invoke_virtual_to_completion(
        action,
        "run",
        "()Ljava/lang/Object;",
        Vec::new(),
    )?;

    Ok(Some(result))
}

#[native(
    class = "java/security/AccessController",
    name = "getStackAccessControlContext",
    descriptor = "()Ljava/security/AccessControlContext;"
)]
pub fn get_stack_access_control_context(
    _ctx: &mut NativeContext,
    _args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    Ok(Some(Reference(None)))
}
