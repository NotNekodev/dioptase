use crate::{error::RuntimeError, native::native_context::NativeContext, vm::value::Value};
use dioptase_native_macros::native;

#[native(
    class = "sun/reflect/Reflection",
    name = "getCallerClass",
    descriptor = "()Ljava/lang/Class;"
)]
pub fn get_caller_class(
    ctx: &mut NativeContext,
    _args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let caller_class = {
        let tid = ctx.thread();
        let thread = ctx.vm_mut().get_thread(tid)?;

        let frames = thread.frames();

        if frames.len() < 2 {
            return Ok(Some(Value::Reference(None)));
        }

        frames[frames.len() - 2].class
    };

    let class_object = ctx.vm_mut().class_object_for(caller_class);

    Ok(Some(Value::Reference(Some(class_object))))
}
