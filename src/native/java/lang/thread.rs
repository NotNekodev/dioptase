use crate::{
    error::RuntimeError,
    native::native_context::NativeContext,
    vm::value::Value::{self},
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
    let priority = match args.get(1) {
        Some(Value::Int(priority)) => *priority,
        _ => return ctx.throw("java/lang/NullPointerException", None),
    };

    let thread_ref = ctx.thread();

    ctx.vm_mut()
        .get_thread(thread_ref)?
        .set_priority(priority as usize);

    Ok(None)
}
