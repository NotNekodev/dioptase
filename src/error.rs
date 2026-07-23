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

    #[error("Invalid opcode {opcode:#04x}")]
    InvalidOpcode { opcode: u8 },

    #[error("No thread frame given on thread {thread_id}")]
    NoCurrentFrame { thread_id: usize },

    // TODO: also pass class name and method name
    #[error("Operand stack underflow in [class].[method] at pc {pc}")]
    OperandStackUnderflow { pc: usize },

    #[error("Thread with id {thread_id} not found")]
    ThreadNotFound { thread_id: usize },

    #[error("Invalid type")]
    InvalidType,
}
