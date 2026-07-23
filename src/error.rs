#[derive(Debug)]
#[allow(dead_code)]
pub enum RuntimeError {
    MethodNotFound { class: String, index: usize },
    ClassNotFound { index: usize },
    InvalidClass,
    NoImplementation,
    InvalidConstantPoolEntry,
}
