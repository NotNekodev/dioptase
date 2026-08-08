use std::collections::HashMap;

use crate::native::native_context::NativeFunction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NativeMethodIdentifier {
    pub class_name: String,
    pub method_name: String,
    pub descriptor: String,
}

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

    pub fn from_inventory() -> Self {
        let mut registry = Self::new();

        for native in inventory::iter::<NativeRegistration> {
            registry.register_parts(
                native.class_name,
                native.method_name,
                native.descriptor,
                native.function,
            );
        }

        registry
    }
}

impl Default for NativeMethodRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NativeRegistration {
    pub class_name: &'static str,
    pub method_name: &'static str,
    pub descriptor: &'static str,
    pub function: NativeFunction,
}

inventory::collect!(NativeRegistration);
