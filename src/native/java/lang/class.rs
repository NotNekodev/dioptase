use crate::{
    error::RuntimeError,
    native::native_context::NativeContext,
    vm::{
        runtime_class::ClassRef,
        value::Value::{self, Reference},
    },
};
use dioptase_native_macros::native;

#[native(
    class = "java/lang/Class",
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
    class = "java/lang/Class",
    name = "getPrimitiveClass",
    descriptor = "(Ljava/lang/String;)Ljava/lang/Class;"
)]
pub fn get_primitive_class(
    ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let mut class_name: String = String::new();

    match args[0].clone() {
        Reference(reference) => match reference {
            Some(objref) => {
                class_name = ctx.vm().java_string_to_rust(objref)?;
            }
            None => {
                ctx.vm_mut().throw(
                    "java/lang/NullPointerException",
                    Some("Called java.lang.Class#getPrimitiveClass() with a null pointer"),
                );
            }
        },

        other => {
            ctx.vm_mut().throw(
                "java/lang/InternalError",
                Some("argument 0 to java.lang.Class#getPrimitiveClass() is not a String"),
            );
            return Err(RuntimeError::Internal(
                crate::error::InternalError::InvalidType {
                    class: "<unknown>".to_string(),
                    method: "<unknown>".to_string(),
                    pc: 0xDEADBEEF,
                    expected: "java.class.String".to_string(),
                    found: format!("{:?}", other),
                },
            ));
        }
    }

    println!("getPrimitiveClass for {:?}", class_name);

    let primitive: ClassRef;

    match class_name.as_str() {
        "boolean" => {
            primitive = ctx.vm().primitive_classes().boolean;
        }

        "byte" => {
            primitive = ctx.vm().primitive_classes().boolean;
        }

        "char" => {
            primitive = ctx.vm().primitive_classes().char;
        }

        "short" => {
            primitive = ctx.vm().primitive_classes().short;
        }

        "int" => {
            primitive = ctx.vm().primitive_classes().int;
        }

        "long" => {
            primitive = ctx.vm().primitive_classes().long;
        }

        "float" => {
            primitive = ctx.vm().primitive_classes().float;
        }

        "double" => {
            primitive = ctx.vm().primitive_classes().double;
        }

        "void" => {
            primitive = ctx.vm().primitive_classes().void;
        }

        _ => {
            return Ok(Some(Reference(None)));
        }
    }

    let primitive_class_ref = ctx.vm_mut().class_object_for(primitive);

    Ok(Some(Reference(Some(primitive_class_ref))))
}

#[native(
    class = "java/lang/Class",
    name = "desiredAssertionStatus0",
    descriptor = "(Ljava/lang/Class;)Z"
)]
pub fn desired_assertion_status(
    _ctx: &mut NativeContext,
    _args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    Ok(Some(Value::Int(0)))
}
