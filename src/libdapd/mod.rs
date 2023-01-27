mod drivers;
mod ide;
use std::future::Future;
use tokio::runtime::Runtime;

pub use drivers::*;
pub use ide::IdeAcceptor;

fn initialise_tracing() {
    tracing_subscriber::fmt::init();
}

/// Helper function to bridge between the async <-> sync code
pub fn run_async(future: impl Future) -> Result<(), Box<dyn std::error::Error>> {
    initialise_tracing();
    let rt = Runtime::new().unwrap();
    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, future);
    Ok(())
}
