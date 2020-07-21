use matrix_sdk::events::room::message::MessageEventContent;
use std::collections::HashMap;
use std::path::Path;
use wasmer_runtime::{imports, instantiate, Func};

#[derive(Default, Clone)]
pub struct Plugins<'a> {
    plugins: HashMap<String, Func<'a, i32, i32>>,
}

impl Plugins<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> crate::error::Result<()> {
        // Let's get the .wasm file as bytes
        let wasm_bytes = std::fs::read(path.as_ref())?;
        // TODO allow setting a descriptor file for a plugin to have multiple functions in a wasm file
        let file_name = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // Our import object, that allows exposing functions to our Wasm module.
        // We're not importing anything, so make an empty import object.
        let import_object = imports! {};

        // Let's create an instance of Wasm module running in the wasmer-runtime
        let instance = instantiate(&wasm_bytes, &import_object)?;
        // TODO properly load plugins that we need
        let func: Func<i32, i32> = instance.exports.get("test")?;
        self.plugins.insert(file_name, func);
        Ok(())
    }

    pub fn call(&self, plugin: &str) {
        let plugin_fn = self.plugins.get(plugin).unwrap();
    }
}
