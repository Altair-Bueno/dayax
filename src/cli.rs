use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

/// Command line arguments
#[derive(Parser, Debug, Clone)]
#[command(name = "dayax", author, version, about, long_about = None)]
pub struct CLI {
    #[arg(long, default_value = "127.0.0.1:8000")]
    pub address: SocketAddr,
    #[arg()]
    pub file: PathBuf,
}
