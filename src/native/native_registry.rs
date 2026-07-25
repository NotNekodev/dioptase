use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use crate::native::native_context::NativeFunction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NativeMethodIdentifier {
    pub class_name: String,
    pub method_name: String,
    pub descriptor: String,
}

#[allow(dead_code)]
impl NativeMethodIdentifier {
    pub fn new(
        class_name: impl Into<String>,
        method_name: impl Into<String>,
        descriptor: impl Into<String>,
    ) -> Self {
        Self {
            class_name: class_name.into(),
            method_name: method_name.into(),
            descriptor: descriptor.into(),
        }
    }
}

pub struct NativeMethodRegistry {
    map: HashMap<NativeMethodIdentifier, NativeFunction>,
}

#[allow(dead_code)]
impl NativeMethodRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        identifier: NativeMethodIdentifier,
        function: NativeFunction,
    ) -> Option<NativeFunction> {
        self.map.insert(identifier, function)
    }

    pub fn register_parts(
        &mut self,
        class_name: impl Into<String>,
        method_name: impl Into<String>,
        descriptor: impl Into<String>,
        function: NativeFunction,
    ) -> Option<NativeFunction> {
        self.register(
            NativeMethodIdentifier::new(class_name, method_name, descriptor),
            function,
        )
    }

    pub fn get(&self, id: &NativeMethodIdentifier) -> Option<NativeFunction> {
        self.map.get(id).copied()
    }

    pub fn contains(&self, identifier: &NativeMethodIdentifier) -> bool {
        self.map.contains_key(identifier)
    }
}

impl Default for NativeMethodRegistry {
    fn default() -> Self {
        Self::new()
    }
}

static NATIVE_REGISTRY: OnceLock<Mutex<NativeMethodRegistry>> = OnceLock::new();

pub fn native_registry() -> &'static Mutex<NativeMethodRegistry> {
    NATIVE_REGISTRY.get_or_init(|| Mutex::new(NativeMethodRegistry::new()))
}
