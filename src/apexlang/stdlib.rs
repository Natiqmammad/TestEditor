use std::collections::HashMap;

use crate::apexlang::ast::Value;
use crate::apexlang::error::ApexError;

pub type NativeFn = fn(&[Value]) -> Result<Value, ApexError>;

#[derive(Clone)]
pub struct NativeCallable {
    name: &'static str,
    func: NativeFn,
}

impl NativeCallable {
    pub fn new(name: &'static str, func: NativeFn) -> Self {
        Self { name, func }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn call(&self, args: &[Value]) -> Result<Value, ApexError> {
        (self.func)(args)
    }
}

#[derive(Default)]
pub struct NativeRegistry {
    modules: HashMap<String, HashMap<String, NativeCallable>>,
}

impl NativeRegistry {
    pub fn with_standard_library() -> Self {
        let mut registry = Self::default();
        nats::register(&mut registry);
        math::register(&mut registry);
        fractions::register(&mut registry);
        mem::register(&mut registry);
        assembly::register(&mut registry);
        r#async::register(&mut registry);
        fs::register(&mut registry);
        os::register(&mut registry);
        net::register(&mut registry);
        proc::register(&mut registry);
        signal::register(&mut registry);
        structs::register(&mut registry);
        serde::register(&mut registry);
        registry
    }

    pub fn register_module<S: Into<String>>(
        &mut self,
        name: S,
        functions: HashMap<String, NativeCallable>,
    ) {
        self.modules.insert(name.into(), functions);
    }

    pub fn get_module(&self, name: &str) -> Option<&HashMap<String, NativeCallable>> {
        self.modules.get(name)
    }

    pub fn get_callable(&self, module: &str, symbol: &str) -> Option<&NativeCallable> {
        self.modules.get(module)?.get(symbol)
    }
}

mod assembly;
mod r#async;
mod fractions;
mod fs;
mod math;
mod mem;
mod nats;
mod net;
mod os;
mod proc;
mod serde;
mod signal;
mod structs;
mod support;
