use thiserror::Error;

use crate::vm::value::ObjectRef;

#[derive(Debug, Error)]
pub enum InternalError {
    #[error("Method {method} not found in class {class}")]
    MethodNotFound { class: String, method: String },
    #[error("Method {method} does not contain any code but isnt abstract")]
    NoCodeInMethod { method: String },
    #[error("Class {class} not found")]
    ClassNotFound { class: String },
    #[error("Invalid constant pool entry")]
    InvalidConstantPoolEntry,
    #[error("Invalid opcode {opcode:#04x} @ pc {pc:#x}")]
    InvalidOpcode { opcode: u8, pc: usize },
    #[error("No thread frame given on thread {thread_id}")]
    NoCurrentFrame { thread_id: usize },
    #[error("Operand stack underflow in [class].[method] at pc {pc:#x}")]
    OperandStackUnderflow { pc: usize },
    #[error("Thread with id {thread_id} not found")]
    ThreadNotFound { thread_id: usize },
    #[error("Invalid type")]
    InvalidType,
    #[error("Failed to load class {class} from classpath: {source_cp}")]
    ClassLoadError { class: String, source_cp: String },
    #[error("Local variable at index {index} not found")]
    NoLocalVar { index: usize },
    #[error("Invalid array type {atype}")]
    InvalidArrayType { atype: u8 },
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Internal(#[from] InternalError),
    #[error("uncaught or in-flight exception object {0:?}")]
    Thrown(ObjectRef),
}
