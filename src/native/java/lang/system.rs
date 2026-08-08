use crate::{
    error::RuntimeError,
    native::native_context::NativeContext,
    vm::value::Value::{self, Reference},
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
        let key_ref = ctx.vm_mut().heap_mut().allocate_string(key.to_string());
        let val_ref = ctx.vm_mut().heap_mut().allocate_string(value.to_string());

        ctx.vm_mut().invoke_virtual_to_completion(
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
