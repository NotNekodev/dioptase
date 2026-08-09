use crate::{
    error::{InternalError, RuntimeError},
    native::native_context::NativeContext,
    vm::value::{ObjectRef, Value},
};
use dioptase_native_macros::native;

pub const ARRAY_BASE_OFFSET: i32 = 16;

#[native(
    class = "sun/misc/Unsafe",
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
    class = "sun/misc/Unsafe",
    name = "arrayBaseOffset",
    descriptor = "(Ljava/lang/Class;)I"
)]
pub fn array_base_offset(
    ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let class_object = match args.get(1) {
        Some(Value::Reference(Some(object_ref))) => *object_ref,

        other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                class: "sun/misc/Unsafe".to_string(),
                method: "arrayBaseOffset".to_string(),
                pc: 0xDEADBEEF,
                expected: "Reference".to_string(),
                found: format!("{:?}", other),
            }));
        }
    };

    let class_ref = ctx.vm().heap().get_class_object(class_object)?;

    let class = ctx.vm().get_class(class_ref)?;

    if !class.name.starts_with('[') {
        ctx.vm_mut().throw(
            "java/lang/IllegalArgumentException",
            Some("arrayBaseOffset called on an object that isnt an array"),
        );

        return Err(RuntimeError::Thrown(ObjectRef(0)));
    }

    Ok(Some(Value::Int(ARRAY_BASE_OFFSET)))
}

#[native(
    class = "sun/misc/Unsafe",
    name = "arrayIndexScale",
    descriptor = "(Ljava/lang/Class;)I"
)]
fn array_index_scale(
    ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let class_obj = match args.get(1) {
        Some(Value::Reference(Some(r))) => *r,
        other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                class: "sun/misc/Unsafe".to_string(),
                method: "arrayBaseOffset".to_string(),
                pc: 0xDEADBEEF,
                expected: "Reference".to_string(),
                found: format!("{:?}", other),
            }));
        }
    };

    let class_ref = ctx.vm().heap().get_class_object(class_obj)?;
    let class = ctx.vm().get_class(class_ref)?;

    let scale = match class.name.as_str() {
        "[Z" | "[B" => 1,
        "[C" | "[S" => 2,
        "[I" | "[F" => 4,
        "[J" | "[D" => 8,

        // Reference arrays
        name if name.starts_with("[L") || name.starts_with("[[") => 8,

        other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                class: class.name.clone(),
                method: "Unsafe.arrayIndexScale".to_string(),
                pc: 0xDEADBEEF,
                expected: "Descritpor".to_string(),
                found: format!("{:?}", other),
            }));
        }
    };

    Ok(Some(Value::Int(scale)))
}

#[native(class = "sun/misc/Unsafe", name = "addressSize", descriptor = "()I")]
fn address_size(_ctx: &mut NativeContext, _args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    Ok(Some(Value::Int(std::mem::size_of::<usize>() as i32)))
}
