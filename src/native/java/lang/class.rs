use crate::{
    error::{InternalError, RuntimeError},
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
                ctx.vm().throw(
                    "java/lang/NullPointerException",
                    Some("Called java.lang.Class#getPrimitiveClass() with a null pointer"),
                );
            }
        },

        other => {
            ctx.vm().throw(
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

    let primitive_class_ref = ctx.vm().class_object_for(primitive);

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

#[native(
    class = "java/lang/Class",
    name = "getName0",
    descriptor = "()Ljava/lang/String;"
)]
pub fn get_name(ctx: &mut NativeContext, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let this = match args.first() {
        Some(Value::Reference(Some(reference))) => *reference,
        _ => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "java/lang/Class".to_string(),
                found: "null or non-reference".to_string(),
                class: "...".to_string(),
                method: "java/lang/Class.getName0".to_string(),
                pc: 0xDEADBEEF,
            }));
        }
    };

    let class_ref = ctx.vm().heap().get_class_object(this)?;

    let name = ctx.vm().get_class(class_ref)?.name.clone();

    let string_ref = ctx.vm().allocate_string(&name)?;

    Ok(Some(Value::Reference(Some(string_ref))))
}

#[native(
    class = "java/lang/Class",
    name = "forName0",
    descriptor = "(Ljava/lang/String;ZLjava/lang/ClassLoader;Ljava/lang/Class;)Ljava/lang/Class;"
)]
pub fn for_name0(ctx: &mut NativeContext, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    if args.len() != 4 {
        return Err(RuntimeError::Internal(InternalError::InvalidType {
            expected: "4 arguments to Class.forName0".to_string(),
            found: format!("{} arguments", args.len()),
            class: "java/lang/Class".to_string(),
            method: "forName0".to_string(),
            pc: 0xDEADBEEF,
        }));
    }

    let name_ref = match args[0] {
        Value::Reference(Some(reference)) => reference,

        Value::Reference(None) => {
            return ctx.throw("java/lang/NullPointerException", Some("class name is null"));
        }

        _ => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "java/lang/String".to_string(),
                found: format!("{:?}", args[0]),
                class: "java/lang/Class".to_string(),
                method: "forName0".to_string(),
                pc: 0xDEADBEEF,
            }));
        }
    };

    let name = ctx.vm().java_string_to_rust(name_ref)?;

    let initialize = match args[1] {
        Value::Int(value) => value != 0,

        _ => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "boolean".to_string(),
                found: format!("{:?}", args[1]),
                class: "java/lang/Class".to_string(),
                method: "forName0".to_string(),
                pc: 0xDEADBEEF,
            }));
        }
    };

    let loader = match args[2] {
        Value::Reference(reference) => reference,

        _ => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "java/lang/ClassLoader".to_string(),
                found: format!("{:?}", args[2]),
                class: "java/lang/Class".to_string(),
                method: "forName0".to_string(),
                pc: 0xDEADBEEF,
            }));
        }
    };

    let _caller = match args[3] {
        Value::Reference(reference) => reference,

        _ => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "java/lang/Class".to_string(),
                found: format!("{:?}", args[3]),
                class: "java/lang/Class".to_string(),
                method: "forName0".to_string(),
                pc: 0xDEADBEEF,
            }));
        }
    };

    let internal_name = if name.starts_with('[') {
        name.replace('.', "/")
    } else {
        name.replace('.', "/")
    };

    let class_object = if let Some(loader_ref) = loader {
        let name_string = ctx.vm().allocate_string(&name)?;

        let result = ctx.vm().invoke_virtual_to_completion(
            loader_ref,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            vec![Value::Reference(Some(name_string))],
        );

        match result {
            Ok(Value::Reference(Some(class_object))) => class_object,

            Ok(Value::Reference(None)) => {
                return ctx.throw("java/lang/ClassNotFoundException", Some(&name));
            }

            Ok(other) => {
                return Err(RuntimeError::Internal(InternalError::InvalidType {
                    expected: "java/lang/Class".to_string(),
                    found: format!("{:?}", other),
                    class: "java/lang/Class".to_string(),
                    method: "forName0".to_string(),
                    pc: 0xDEADBEEF,
                }));
            }

            Err(RuntimeError::Thrown(exception)) => {
                return Err(RuntimeError::Thrown(exception));
            }

            Err(error) => return Err(error),
        }
    } else {
        let class_ref = match ctx.vm().resolve_class(&internal_name) {
            Ok(class_ref) => class_ref,

            Err(RuntimeError::Internal(InternalError::ClassNotFound { .. })) => {
                return ctx.throw("java/lang/ClassNotFoundException", Some(&name));
            }

            Err(error) => return Err(error),
        };

        ctx.vm().class_object_for(class_ref)
    };

    let class_ref = ctx.vm().heap().get_class_object(class_object)?;

    if initialize {
        ctx.vm().ensure_class_initialized(class_ref)?;
    }

    Ok(Some(Value::Reference(Some(class_object))))
}
