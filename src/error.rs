use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Method at index {index} not found in class {class}")]
    MethodNotFound { class: String, index: usize },

    #[error("Method {method} does not contain any code but isnt abstract")]
    NoCodeInMethod { method: String },

    #[error("Class at index {index} not found")]
    ClassNotFound { index: usize },

    #[error("Invalid constant pool entry")]
    InvalidConstantPoolEntry,
}
