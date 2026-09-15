use crate::{
    error::RuntimeError,
    vm::{thread::ThreadRef, value::Value, vm::VM},
};

#[allow(dead_code)]
pub struct NativeContext<'a> {
    vm: &'a VM,
    thread: ThreadRef,
}

#[allow(dead_code)]
impl<'a> NativeContext<'a> {
    pub fn new(vm: &'a VM, thread: ThreadRef) -> Self {
        Self { vm, thread }
    }

    pub fn vm(&self) -> &VM {
        self.vm
    }

    pub fn thread(&self) -> ThreadRef {
        self.thread
    }

    pub fn throw<T>(&mut self, class: &str, message: Option<&str>) -> Result<T, RuntimeError> {
        Err(self.vm.throw(class, message))
    }
}

pub type NativeFunction =
    fn(&mut NativeContext<'_>, &[Value]) -> Result<Option<Value>, RuntimeError>;
