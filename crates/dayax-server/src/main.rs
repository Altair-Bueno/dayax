mod cli;

use clap::Parser;
use mlua::Lua;
use signal_hook::{consts::*, low_level::signal_name};
use signal_hook_tokio::Signals;

use futures::stream::StreamExt;

use std::sync::Arc;

use axum::{Router, Server};
use tokio::sync::{oneshot::Sender, Mutex};
use tracing::info;

use crate::cli::Cli;
use dayax::Dayax;

const GLOBAL_DAYAX: &str = "dayax";
const GLOBAL_VERSION: &str = "_VERSION";
const UNKNOWN_SIGNAL_NAME: &str = "UNKNOWN";

async fn handle_signals(mut signals: Signals, sender: Sender<i32>) {
    if let Some(signal) = signals.next().await {
        _ = sender.send(signal);
    }
}

fn init_lua() -> mlua::Result<Lua> {
    let lua = Lua::new();
    let version: String = lua.globals().get(GLOBAL_VERSION)?;
    info!(version, "Lua engine running");
    let dayax = Dayax::new();
    lua.globals().set(GLOBAL_DAYAX, dayax)?;
    Ok(lua)
}

async fn run(Cli { file, address }: Cli) -> eyre::Result<()> {
    info!(pid = std::process::id(), "Initializing server");

    // Init the signal handler
    let (signal_sender, signal_reciver) = tokio::sync::oneshot::channel();
    let signals = Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
    let handle = signals.handle();
    let signals_task = tokio::spawn(handle_signals(signals, signal_sender));

    let lua = init_lua()?;
    // Load the router configuration
    info!(?file, "Loading source script");
    lua.load(file.as_path()).exec()?;
    let dayax: Dayax = lua.globals().get(GLOBAL_DAYAX)?;
    let appstate = Arc::new(Mutex::new(lua));

    // Kick off the server
    info!(%address, "Server ready");
    Server::bind(&address)
        .serve(
            Router::from(dayax)
                .with_state(appstate.clone())
                .into_make_service(),
        )
        .with_graceful_shutdown(async {
            let signal = signal_reciver.await.ok();
            let name = signal.and_then(signal_name).unwrap_or(UNKNOWN_SIGNAL_NAME);
            info!(?signal, name, "Shutting down server");
        })
        .await?;

    // Cleanup
    {
        let lock = appstate.lock().await;
        lock.expire_registry_values();
    }
    // Terminate the signal stream.
    handle.close();
    signals_task.await?;
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    run(Cli::parse()).await
}
