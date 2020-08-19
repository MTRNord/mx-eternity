use matrix_sdk::{
    events::room::message::{MessageEventContent, NoticeMessageEventContent},
    identifiers::RoomId,
    Client,
};
use std::{collections::HashMap, convert::TryFrom, path::Path, str};
use wasmer_runtime::{func, imports, instantiate, Ctx, Func, Instance};

use crate::{PLUGINS, utils::pointer_to_str};
use log::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct PluginInstance {
    plugin_name: String,
    instance: Option<Instance>,
}

#[derive(Default)]
pub struct Plugins {
    pub matrix_client: Option<Client>,
    instances: Vec<PluginInstance>,
    pluginname_by_function: HashMap<String, String>,
}

// TODO rewrite to load when calling

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct EventDummy {
    pub r#type: String,
    pub content: EventContentDummy,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct EventContentDummy {
    pub msgtype: String,
    pub body: String,
}

/// Function that provides a way of sending matrix messages from plugins
fn send_message(
    ctx: &mut Ctx,
    content_ptr: u32,
    content_len: u32,
    room_id_ptr: u32,
    room_id_len: u32,
) {
    info!("send message");
    // Get a slice that maps to the memory currently used by the webassembly
    // instance.
    //
    // Webassembly only supports a single memory for now,
    // but in the near future, it'll support multiple.
    //
    // Therefore, we don't assume you always just want to access first
    // memory and force you to specify the first memory.
    let memory = ctx.memory(0);

    // Get a subslice that corresponds to the memory used by the string.
    let content_str_vec: Vec<_> = memory.view()
        [content_ptr as usize..(content_ptr + content_len) as usize]
        .iter()
        .map(|cell| cell.get())
        .collect();

    // Get a subslice that corresponds to the memory used by the string.
    let room_id_str_vec: Vec<_> = memory.view()
        [room_id_ptr as usize..(room_id_ptr + room_id_len) as usize]
        .iter()
        .map(|cell| cell.get())
        .collect();
    tokio::spawn(async move {
        // Convert the subslice to a `&str`.
        let content_str = str::from_utf8(&content_str_vec).unwrap();

        // Convert the subslice to a `&str`.
        let room_id_str = str::from_utf8(&room_id_str_vec).unwrap();

        let content_string = String::from(content_str);

        let room_id_string = String::from(room_id_str);

        // Print it!
        println!("{}", content_string);
        // Print it!
        println!("{}", room_id_string);

        let e: EventDummy = serde_json::from_str(&content_string.clone()).unwrap();
        let room_id = RoomId::try_from(room_id_string).unwrap();
        // TODO properly convert
        let msg = MessageEventContent::Notice(NoticeMessageEventContent {
            body: e.content.body,
            formatted: None,
            // TODO allow relates_to
            relates_to: None,
        });

        let plugins = PLUGINS.lock().await;
        plugins
            .matrix_client
            .clone()
            .unwrap()
            .room_send(&room_id, msg.clone(), None)
            .await
            .unwrap();
    });
}

/// Function that prints the log
fn println(ctx: &mut Ctx, text_ptr: u32, text_len: u32) {
    // Get a slice that maps to the memory currently used by the webassembly
    // instance.
    //
    // Webassembly only supports a single memory for now,
    // but in the near future, it'll support multiple.
    //
    // Therefore, we don't assume you always just want to access first
    // memory and force you to specify the first memory.
    let memory = ctx.memory(0);

    // Get a subslice that corresponds to the memory used by the string.
    let text_str_vec: Vec<_> = memory.view()[text_ptr as usize..(text_ptr + text_len) as usize]
        .iter()
        .map(|cell| cell.get())
        .collect();

    // Convert the subslice to a `&str`.
    let text_str = str::from_utf8(&text_str_vec).unwrap();
    info!("{}", text_str);
}

/// Function that prints the log
fn error(ctx: &mut Ctx, text_ptr: u32, text_len: u32) {
    // Get a slice that maps to the memory currently used by the webassembly
    // instance.
    //
    // Webassembly only supports a single memory for now,
    // but in the near future, it'll support multiple.
    //
    // Therefore, we don't assume you always just want to access first
    // memory and force you to specify the first memory.
    let memory = ctx.memory(0);

    // Get a subslice that corresponds to the memory used by the string.
    let text_str_vec: Vec<_> = memory.view()[text_ptr as usize..(text_ptr + text_len) as usize]
        .iter()
        .map(|cell| cell.get())
        .collect();

    // Convert the subslice to a `&str`.
    let text_str = str::from_utf8(&text_str_vec).unwrap();
    error!("{}", text_str);
}

/// Function that prints the log
fn warn(ctx: &mut Ctx, text_ptr: u32, text_len: u32) {
    let text_str = pointer_to_str(ctx, text_ptr, text_len);
    warn!("{}", text_str);
}

impl Plugins {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load<P: AsRef<Path>>(
        &mut self,
        plugin_path: P,
        config_path: P,
    ) -> crate::error::Result<()> {
        // Parse plugin config
        let file = std::fs::File::open(config_path)?;
        let plugin_config: PluginConfig = serde_json::from_reader(file)?;
        plugin_config.validate_config()?;

        // Let's get the .wasm file as bytes
        let wasm_bytes = std::fs::read(plugin_path.as_ref())?;
        let plugin_filename = plugin_path.as_ref().file_stem().unwrap();

        // Our import object, that allows exposing functions to our Wasm module.
        // We're not importing anything, so make an empty import object.
        let import_object = imports! {
            // Define the "env" namespace that was implicitly used
            // by our sample application.
            "env" => {
                // name        // the func! macro autodetects the signature
                "send_message" => func!(send_message),
                "println" => func!(println),
                "info" => func!(println),
                "warn" => func!(warn),
                "error" => func!(error),
            },
        };

        // Let's create an instance of Wasm module running in the wasmer-runtime
        let instance = instantiate(&wasm_bytes, &import_object)?;

        let plugin_name = plugin_filename.to_str().unwrap().to_owned();
        let plugin_instance = PluginInstance {
            plugin_name: plugin_name.clone(),
            instance: Some(instance),
        };
        self.instances.push(plugin_instance);
        for function in plugin_config.exported_functions {
            match function {
                ExportedFunction::String(function_name) => {
                    self.pluginname_by_function
                        .insert(function_name, plugin_name.clone());
                }
                ExportedFunction::Object(function_object) => {
                    let function_name = function_object.name;
                    // TODO handle args
                    self.pluginname_by_function
                        .insert(function_name, plugin_name.clone());
                }
            }
        }

        Ok(())
    }

    // TODO better error handling
    pub fn call(&self, function_name: &str) -> crate::error::Result<()> {
        info!("call running");
        let plugin_name = self.pluginname_by_function.get(function_name).unwrap();

        let instance = self
            .instances
            .iter()
            .find(|x| &x.plugin_name == plugin_name)
            .unwrap()
            .instance
            .as_ref()
            .unwrap();

        info!("got instance");

        let func: Func<(), ()> = instance.exports.get(function_name)?;
        info!("got func");
        func.call()?;
        info!("called");

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PluginConfig {
    name: String,
    version: String,
    exported_functions: Vec<ExportedFunction>,
    has_help: bool,
    license: String,
    dependencies: Vec<String>,
}

impl PluginConfig {
    fn validate_config(&self) -> crate::error::Result<()> {
        // Check for valid version (semver)
        semver::Version::parse(self.version.as_str())?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
enum ExportedFunction {
    String(String),
    Object(LongExportedFunction),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LongExportedFunction {
    name: String,
    args: Vec<String>,
}
