mod drivers;
mod handlers;
mod in_flight_requests_queue;

pub use in_flight_requests_queue::InFlightRequestQueue;
pub use std::future::Future;
pub use tokio::runtime::Runtime;

pub use drivers::*;
pub use handlers::*;

use std::sync::Once;
static INIT_TRACING: Once = Once::new();

fn initialise_tracing() {
    tracing_subscriber::fmt::init();
}

/// Helper function to bridge between the async <-> sync code
pub fn run_async(future: impl Future) -> Result<(), Box<dyn std::error::Error>> {
    INIT_TRACING.call_once(|| {
        // run initialization here
        initialise_tracing();
    });

    let rt = Runtime::new().unwrap();
    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, future);
    Ok(())
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum InFlightRequest {
    SetBreakpoint(dap::requests::Request),
}
