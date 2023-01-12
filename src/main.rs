mod cli;
mod handler;
mod runtime;

use std::sync::Arc;

use axum::routing::any;
use axum::Router;
use axum::Server;
use clap::Parser;
use cli::CLI;
use mlua::Function;
use tokio::sync::Mutex;
use tracing::debug;
use tracing::info;

async fn run(CLI { file, address }: CLI) -> eyre::Result<()> {
    let lua = crate::runtime::start()?;
    let version: String = lua.globals().get("_VERSION")?;
    info!(version, "Lua engine running");

    info!(?file, "Loading source script");
    lua.load(file.as_path()).exec()?;
    let dayax = crate::runtime::get_dayax(&lua)?;
    let mut router = Router::new();
    for pair in dayax.pairs::<String, Function>() {
        let (route, callback) = pair?;
        router = router.route(&route, any(crate::handler::request_handler));
        debug!(
            route,
            def = format!(
                "{}:{}",
                file.to_string_lossy(),
                callback.info().line_defined
            ),
            "Found route handler"
        );
    }
    let router = router.with_state(Arc::new(Mutex::new(lua)));
    info!(%address, "Server ready");
    Server::bind(&address)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    run(CLI::parse()).await
}
