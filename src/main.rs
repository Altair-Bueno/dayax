mod cli;
mod handler;
mod runtime;
mod userdata;

use clap::Parser;
use signal_hook::{consts::*, low_level::signal_name};
use signal_hook_tokio::Signals;

use futures::stream::StreamExt;

use std::sync::Arc;

use axum::Server;
use tokio::sync::{oneshot::Sender, Mutex};
use tracing::info;

use crate::cli::CLI;
use crate::userdata::Dayax;

async fn handle_signals(mut signals: Signals, sender: Sender<i32>) {
    if let Some(signal) = signals.next().await {
        _ = sender.send(signal);
    }
}

async fn run(CLI { file, address }: CLI) -> eyre::Result<()> {
    let pid = std::process::id();
    info!(pid, "Initializing server");

    // Init the signal handler
    let (signal_sender, signal_reciver) = tokio::sync::oneshot::channel();
    let signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
    let handle = signals.handle();
    let signals_task = tokio::spawn(handle_signals(signals, signal_sender));

    // Init Lua runtime
    let lua = crate::runtime::start()?;
    let version: String = lua.globals().get("_VERSION")?;
    info!(version, "Lua engine running");

    // Load the router configuration
    info!(?file, "Loading source script");
    lua.load(file.as_path()).exec()?;
    let Dayax { router } = crate::runtime::get_dayax(&lua)?;
    let appstate = Arc::new(Mutex::new(lua));

    // Kick off the server
    info!(%address, "Server ready");
    Server::bind(&address)
        .serve(router.with_state(appstate.clone()).into_make_service())
        .with_graceful_shutdown(async {
            let signal = signal_reciver.await.ok();
            let name = signal.map(signal_name).flatten().unwrap_or("UNKNOWN");
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
    run(CLI::parse()).await
}
