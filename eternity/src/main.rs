use crate::config::Config;
use crate::matrix::login;
use crate::wasm_plugins::Plugins;
use clap::Clap;
use lazy_static::lazy_static;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

mod config;
mod error;
mod matrix;
mod wasm_plugins;

#[derive(Clap)]
#[clap(version = "1.0", author = "MTRNord <info@nordgedanken.de>")]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "config.yaml")]
    config: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

lazy_static! {
    pub static ref PLUGINS: Arc<Mutex<Plugins<'static>>> = Arc::new(Mutex::new(Plugins::new()));
}

#[tokio::main]
async fn main() -> crate::error::Result<()> {
    setup_logger()?;
    // Parse args
    let opts: Opts = Opts::parse();

    // Parse config
    let config = Config::load(opts.config)?;

    // Load Plugins
    let paths = fs::read_dir(config.plugins_path).unwrap();

    for path in paths {
        let safe_path = path?;
        if safe_path.file_name().to_str().unwrap().ends_with("wasm") {
            println!("loading: {:?}", safe_path.path());
            let mut state = PLUGINS.lock().expect("Could not lock mutex");
            state.load(safe_path.path())?;
            println!("loaded: {:?}", safe_path.path());
        }
    }

    println!("loading complete");

    login(
        config.matrix.homeserver_url,
        config.matrix.username,
        config.matrix.access_token,
        config.matrix.store_path,
    )
    .await?;

    Ok(())
}

fn setup_logger() -> crate::error::Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("hyper", log::LevelFilter::Off)
        .level_for("reqwest", log::LevelFilter::Off)
        .level_for("matrix_sdk_base", log::LevelFilter::Off)
        .level_for("tracing", log::LevelFilter::Off)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
