use crate::{
    error::{InternalError, RuntimeError},
    native::native_context::NativeContext,
    vm::{
        heap::ArrayElementType,
        value::Value::{self, Reference},
    },
};
use dioptase_native_macros::native;

#[native(
    class = "java/lang/System",
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
    class = "java/lang/System",
    name = "initProperties",
    descriptor = "(Ljava/util/Properties;)Ljava/util/Properties;"
)]
pub fn init_properties(
    ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Option<Value>, RuntimeError> {
    let props_ref = match args[0] {
        Reference(Some(r)) => r,
        _ => return ctx.throw("java/lang/NullPointerException", None),
    };

    let entries: &[(&str, &str)] = &[
        ("java.version", "1.8.0"),
        ("java.vendor", "dioptase"),
        ("java.vendor.url", "https://github.com/NotNekodev/dioptase"),
        ("java.home", "/usr/lib/jvm/openjdk8/jre"),
        ("java.class.version", "52.0"),
        ("java.class.path", "."),
        ("os.name", std::env::consts::OS),
        ("os.arch", std::env::consts::ARCH),
        ("os.version", "unknown"),
        ("file.separator", "/"),
        ("path.separator", ":"),
        ("line.separator", "\n"),
        ("user.name", "user"),
        ("user.home", "/home/user"),
        ("user.dir", "."),
        ("java.io.tmpdir", "/tmp"),
        ("java.specification.version", "1.8"),
        ("java.specification.name", "Java Platform API Specification"),
        ("java.specification.vendor", "dioptase"),
        ("java.vm.name", "dioptase VM"),
        ("java.vm.version", env!("CARGO_PKG_VERSION")),
        ("java.vm.vendor", "dioptase"),
        ("java.vm.specification.version", "1.8"),
        (
            "java.vm.specification.name",
            "Java Virtual Machine Specification",
        ),
        ("java.vm.specification.vendor", "dioptase"),
    ];

    for (key, value) in entries {
        let key_ref = ctx.vm().allocate_string(key)?;
        let val_ref = ctx.vm().allocate_string(value)?;

        ctx.vm().invoke_virtual_to_completion(
            props_ref,
            "setProperty",
            "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;",
            vec![
                Value::Reference(Some(key_ref)),
                Value::Reference(Some(val_ref)),
            ],
        )?;
    }

    Ok(Some(Reference(Some(props_ref))))
}

#[native(
    class = "java/lang/System",
    name = "arraycopy",
    descriptor = "(Ljava/lang/Object;ILjava/lang/Object;II)V"
)]
pub fn arraycopy(ctx: &mut NativeContext, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    if args.len() != 5 {
        return Err(RuntimeError::Internal(InternalError::InvalidType {
            expected: "5 arguments to System.arraycopy".to_string(),
            found: format!("{} arguments", args.len()),
            class: "java/lang/System".to_string(),
            pc: 0xDEADBEEF,
            method: "arraycopy".to_string(),
        }));
    }

    let src = match args[0] {
        Value::Reference(Some(r)) => r,
        Value::Reference(None) => {
            return ctx.throw("java/lang/NullPointerException", Some("src"));
        }
        ref other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "Object reference for src".to_string(),
                found: format!("{:?}", other),
                class: "java/lang/System".to_string(),
                method: "arraycopy".to_string(),
                pc: 0xDEADBEEF,
            }));
        }
    };

    let src_pos = match args[1] {
        Value::Int(v) => v,
        ref other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "int for srcPos".to_string(),
                found: format!("{:?}", other),
                class: "java/lang/System".to_string(),
                pc: 0xDEADBEEF,
                method: "arraycopy".to_string(),
            }));
        }
    };

    let dest = match args[2] {
        Value::Reference(Some(r)) => r,
        Value::Reference(None) => {
            return ctx.throw("java/lang/NullPointerException", Some("dest"));
        }
        ref other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "Object reference for dest".to_string(),
                found: format!("{:?}", other),
                pc: 0xDEADBEEF,
                class: "java/lang/System".to_string(),
                method: "arraycopy".to_string(),
            }));
        }
    };

    let dest_pos = match args[3] {
        Value::Int(v) => v,
        ref other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "int for destPos".to_string(),
                found: format!("{:?}", other),
                pc: 0xDEADBEEF,
                class: "java/lang/System".to_string(),
                method: "arraycopy".to_string(),
            }));
        }
    };

    let length = match args[4] {
        Value::Int(v) => v,
        ref other => {
            return Err(RuntimeError::Internal(InternalError::InvalidType {
                expected: "int for length".to_string(),
                found: format!("{:?}", other),
                pc: 0xDEADBEEF,
                class: "java/lang/System".to_string(),
                method: "arraycopy".to_string(),
            }));
        }
    };

    if src_pos < 0 || dest_pos < 0 || length < 0 {
        return ctx.throw(
            "java/lang/IndexOutOfBoundsException",
            Some("Negative arraycopy index"),
        );
    }

    let src_array = ctx.vm().heap().with_array(src, |a| Ok(a.clone()));
    let src_array = match src_array {
        Ok(array) => array,
        Err(_) => {
            return ctx.throw(
                "java/lang/ArrayStoreException",
                Some("source is not an array"),
            );
        }
    };

    let dest_array = ctx.vm().heap().with_array(dest, |a| Ok(a.clone()));
    let dest_array = match dest_array {
        Ok(array) => array,
        Err(_) => {
            return ctx.throw(
                "java/lang/ArrayStoreException",
                Some("destination is not an array"),
            );
        }
    };

    let src_end = match src_pos.checked_add(length) {
        Some(v) => v as usize,
        None => {
            return ctx.throw(
                "java/lang/IndexOutOfBoundsException",
                Some("source array range overflow"),
            );
        }
    };

    let dest_end = match dest_pos.checked_add(length) {
        Some(v) => v as usize,
        None => {
            return ctx.throw(
                "java/lang/IndexOutOfBoundsException",
                Some("destination array range overflow"),
            );
        }
    };

    if src_end > src_array.elements.len() {
        return ctx.throw(
            "java/lang/IndexOutOfBoundsException",
            Some("source array bounds"),
        );
    }

    if dest_end > dest_array.elements.len() {
        return ctx.throw(
            "java/lang/IndexOutOfBoundsException",
            Some("destination array bounds"),
        );
    }

    if length == 0 {
        return Ok(None);
    }

    let src_type = src_array.element_type;
    let dest_type = dest_array.element_type;

    let src_is_reference = matches!(src_type, ArrayElementType::Reference(_));
    let dest_is_reference = matches!(dest_type, ArrayElementType::Reference(_));

    if !src_is_reference || !dest_is_reference {
        if src_is_reference || dest_is_reference {
            return ctx.throw(
                "java/lang/ArrayStoreException",
                Some("incompatible array types"),
            );
        }

        if src_type != dest_type {
            return ctx.throw(
                "java/lang/ArrayStoreException",
                Some("incompatible primitive array types"),
            );
        }

        let values = {
            let array = ctx.vm().heap().get_array(src)?;

            array.elements[src_pos as usize..src_end as usize].to_vec()
        };

        ctx.vm().heap().with_array_mut(dest, |a| {
            a.elements[src_pos as usize..src_end as usize].clone_from_slice(&values);
            Ok(())
        })?;

        return Ok(None);
    }

    let dest_component = match dest_type {
        ArrayElementType::Reference(class) => class,
        _ => unreachable!(),
    };

    let values = {
        let array = ctx.vm().heap().get_array(src)?;
        array.elements[src_pos as usize..src_end as usize].to_vec()
    };

    for value in &values {
        let reference = match value {
            Value::Reference(Some(r)) => *r,
            Value::Reference(None) => continue,
            _ => {
                return Err(RuntimeError::Internal(InternalError::InvalidHeapEntry {
                    expected: "reference value in reference array",
                    found: format!("{:?}", value),
                }));
            }
        };

        let actual_class = ctx.vm().runtime_class_of(reference)?;

        if !ctx.vm().is_assignable(actual_class, dest_component)? {
            return ctx.throw(
                "java/lang/ArrayStoreException",
                Some("array element has incompatible type"),
            );
        }
    }

    ctx.vm().heap().with_array_mut(dest, |a| {
        a.elements[src_pos as usize..src_end as usize].clone_from_slice(&values);
        Ok(())
    })?;

    Ok(None)
}
