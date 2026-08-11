use crate::{
    error::{InternalError, RuntimeError},
    native::native_context::NativeContext,
    vm::{heap::ArrayElementType, value::Value},
};
use dioptase_native_macros::native;

#[native(
    class = "java/lang/Throwable",
    name = "fillInStackTrace",
    descriptor = "(I)Ljava/lang/Throwable;"
)]
pub fn fill_in_stack_trace(
    ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::Internal(InternalError::InvalidType {
            expected: "2 arguments to Throwable.fillInStackTrace".to_string(),
            found: format!("{} arguments", args.len()),
            class: "java/lang/Throwable".to_string(),
            method: "fillInStackTrace".to_string(),
            pc: 0xDEADBEEF,
        }));
    }

    let throwable_ref = match args.get(0).unwrap() {
        Value::Reference(Some(reference)) => reference,

        Value::Reference(None) => {
            return ctx.throw("java/lang/NullPointerException", Some("this"));
        }

        other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "java/lang/Throwable".to_string(),
                found: format!("{:?}", other),
                class: "java/lang/Throwable".to_string(),
                method: "fillInStackTrace".to_string(),
                pc: 0xDEADBEEF,
            }));
        }
    };

    let throwable_class = ctx.vm_mut().resolve_class("java/lang/Throwable")?;
    let actual_class = ctx.vm_mut().runtime_class_of(*throwable_ref)?;

    if !ctx.vm().is_assignable(actual_class, throwable_class)? {
        return Err(RuntimeError::Internal(InternalError::InvalidType {
            expected: "java/lang/Throwable".to_string(),
            found: ctx
                .vm()
                .get_class(actual_class)
                .map(|c| c.name.clone())
                .unwrap_or_else(|_| format!("class {}", actual_class.0)),
            class: "java/lang/Throwable".to_string(),
            method: "fillInStackTrace".to_string(),
            pc: 0xDEADBEEF,
        }));
    }

    let thread_ref = ctx.thread();

    let frame_info = {
        let thread = ctx.vm_mut().get_thread(thread_ref)?;

        thread
            .frames()
            .iter()
            .rev()
            .map(|frame| {
                let class_ref = frame.class;
                let method_index = frame.method_index;

                (class_ref, method_index)
            })
            .collect::<Vec<_>>()
    };

    let mut stack_frames = Vec::new();

    for (class_ref, method_index) in frame_info {
        let class_name = ctx.vm().get_class(class_ref)?.name.clone();

        if class_name == "java/lang/Throwable" {
            continue;
        }

        stack_frames.push((class_ref, method_index));
    }

    let stack_trace_element_class = ctx.vm_mut().resolve_class("java/lang/StackTraceElement")?;

    let stack_trace_array_class = ctx
        .vm_mut()
        .resolve_class("[Ljava/lang/StackTraceElement;")?;

    let array_ref = ctx.vm_mut().heap_mut().allocate_array(
        stack_trace_array_class,
        ArrayElementType::Reference(stack_trace_element_class),
        stack_frames.len(),
    );

    for (index, (class_ref, method_index)) in stack_frames.iter().enumerate() {
        let class = ctx.vm().get_class(*class_ref)?;
        let class_name = class.name.clone();

        let method = ctx.vm().get_method(*class_ref, *method_index)?;
        let method_name = method.name.clone();

        let declaring_class = class_name.replace('/', ".");

        let defaults = ctx.vm().default_field_values(stack_trace_element_class)?;

        let element_ref = ctx
            .vm_mut()
            .heap_mut()
            .allocate_object_typed(stack_trace_element_class, &defaults);

        let declaring_class_ref = ctx.vm_mut().allocate_string(&declaring_class)?;

        let method_name_ref = ctx.vm_mut().allocate_string(&method_name)?;

        if let Some((_, slot)) = ctx
            .vm()
            .find_instance_field(stack_trace_element_class, "declaringClass")?
        {
            ctx.vm_mut().heap_mut().with_object_mut(element_ref, |o| {
                o.fields[slot] = Value::Reference(Some(declaring_class_ref));
                Ok(())
            })?;
        }

        if let Some((_, slot)) = ctx
            .vm()
            .find_instance_field(stack_trace_element_class, "methodName")?
        {
            ctx.vm_mut().heap_mut().with_object_mut(element_ref, |o| {
                o.fields[slot] = Value::Reference(Some(method_name_ref));
                Ok(())
            })?;
        }

        if let Some((_, slot)) = ctx
            .vm()
            .find_instance_field(stack_trace_element_class, "fileName")?
        {
            ctx.vm_mut().heap_mut().with_object_mut(element_ref, |o| {
                o.fields[slot] = Value::Reference(None);
                Ok(())
            })?;
        }

        if let Some((_, slot)) = ctx
            .vm()
            .find_instance_field(stack_trace_element_class, "lineNumber")?
        {
            ctx.vm_mut().heap_mut().with_object_mut(element_ref, |o| {
                o.fields[slot] = Value::Int(-1);
                Ok(())
            })?;
        }

        ctx.vm_mut().heap_mut().with_array_mut(array_ref, |a| {
            a.elements[index] = Value::Reference(Some(element_ref));
            Ok(())
        })?;
    }

    let (_, stack_trace_slot) = ctx
        .vm()
        .find_instance_field(throwable_class, "stackTrace")?
        .ok_or_else(|| RuntimeError::Internal(InternalError::InvalidSlot))?;

    ctx.vm_mut()
        .heap_mut()
        .with_object_mut(*throwable_ref, |o| {
            o.fields[stack_trace_slot] = Value::Reference(Some(array_ref));
            Ok(())
        })?;

    Ok(Some(Value::Reference(Some(*throwable_ref))))
}
