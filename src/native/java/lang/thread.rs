use crate::{
    error::RuntimeError::{self},
    native::native_context::NativeContext,
    vm::{
        thread::ThreadState,
        value::Value::{self},
    },
};
use dioptase_native_macros::native;

#[native(
    class = "java/lang/Thread",
    name = "registerNatives",
    descriptor = "()V"
)]
pub fn register_natives(
    _ctx: &mut NativeContext,
    _args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    Ok(None)
}

#[native(
    class = "java/lang/Thread",
    name = "currentThread",
    descriptor = "()Ljava/lang/Thread;"
)]
pub fn current_thread(
    ctx: &mut NativeContext,
    _args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let thread_ref = ctx.thread();
    let obj_ref = ctx.vm_mut().thread_object_for(thread_ref)?;
    Ok(Some(Value::Reference(Some(obj_ref))))
}

#[native(class = "java/lang/Thread", name = "setPriority0", descriptor = "(I)V")]
pub fn set_priority0(
    ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let obj_ref = match args.first() {
        Some(Value::Reference(Some(r))) => *r,
        Some(Value::Reference(None)) => {
            return ctx.throw("java/lang/NullPointerException", None);
        }
        _ => {
            return ctx.throw("java/lang/InternalError", None);
        }
    };

    let priority = match args.get(1) {
        Some(Value::Int(priority)) => *priority,
        _ => return ctx.throw("java/lang/NullPointerException", None),
    };

    let thread_ref = ctx.vm_mut().thread_ref_from_object(obj_ref)?;

    ctx.vm_mut()
        .get_thread(thread_ref)?
        .set_priority(priority as usize);

    Ok(None)
}

#[native(class = "java/lang/Thread", name = "isAlive", descriptor = "()Z")]
pub fn is_alive(ctx: &mut NativeContext, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let obj_ref = match args.first() {
        Some(Value::Reference(Some(r))) => *r,
        Some(Value::Reference(None)) => {
            return ctx.throw("java/lang/NullPointerException", None);
        }
        _ => {
            return ctx.throw("java/lang/InternalError", None);
        }
    };

    let thread_ref = ctx.vm_mut().thread_ref_from_object(obj_ref)?;
    let thread = ctx.vm_mut().get_thread(thread_ref)?;

    let is_alive = match thread.state() {
        ThreadState::Terminated => 0,
        _ => 1,
    };

    Ok(Some(Value::Int(is_alive)))
}
