use matrix_sdk::events::room::message::{MessageEventContent, NoticeMessageEventContent};
use std::collections::HashMap;
use std::path::Path;
use wasmer_runtime::{imports, instantiate, Array, Func, Instance, WasmPtr};

#[derive(Default)]
pub struct PluginInstance {
    plugin_name: String,
    instance: Option<Instance>,
}

#[derive(Default)]
pub struct Plugins {
    instances: Vec<PluginInstance>,
    pluginname_by_function: HashMap<String, String>,
}

// TODO rewrite to load when calling

impl Plugins {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load<P: AsRef<Path>>(&mut self, plugin_path: P) -> crate::error::Result<()> {
        // Let's get the .wasm file as bytes
        let wasm_bytes = std::fs::read(plugin_path.as_ref())?;
        let plugin_filename = plugin_path.as_ref().file_stem().unwrap();

        // Our import object, that allows exposing functions to our Wasm module.
        // We're not importing anything, so make an empty import object.
        let import_object = imports! {};

        // Let's create an instance of Wasm module running in the wasmer-runtime
        let instance = instantiate(&wasm_bytes, &import_object)?;

        let plugin_name = plugin_filename.to_str().unwrap().to_owned();
        let plugin_instance = PluginInstance {
            plugin_name: plugin_name.clone(),
            instance: Some(instance),
        };
        self.instances.push(plugin_instance);
        // TODO have a config to iterate over and add all funcs to that hashmap
        self.pluginname_by_function
            .insert("test".to_string(), plugin_name);

        Ok(())
    }

    // TODO better error handling
    pub fn call(&self, function_name: &str) -> crate::error::Result<MessageEventContent> {
        let plugin_name = self.pluginname_by_function.get(function_name).unwrap();

        let instance = self
            .instances
            .iter()
            .find(|x| &x.plugin_name == plugin_name)
            .unwrap()
            .instance
            .as_ref()
            .unwrap();

        // Lets get the context and memory of our Wasm Instance for the return value
        let wasm_instance_context = instance.context();
        let wasm_instance_memory = wasm_instance_context.memory(1);

        let get_wasm_memory_buffer_pointer: Func<(), WasmPtr<u8, Array>> = instance
            .exports
            .get(format!("{}_return_value", function_name).as_str())
            .expect("get_plugin_return_value");

        // TODO properly load plugins that we need
        let func: Func<(), ()> = instance.exports.get(function_name)?;
        let result = func.call()?;

        let msg = MessageEventContent::Notice(NoticeMessageEventContent {
            body: String::from("blubtest"),
            formatted: None,
            // TODO allow relates_to
            relates_to: None,
        });
        Ok(msg)
    }
}
