use dap::line_reader::FileLineReader;
use dap::prelude::*;
mod libdapd;
use crate::libdapd::{run_async, DriverGDB};
use std::error::Error as StdError;

use crate::libdapd::InFlightRequestQueue;

use crate::libdapd::handle_configuration_stage;
/// handlers
use crate::libdapd::handle_initialise;
use crate::libdapd::handle_launch;

fn main() -> Result<(), Box<dyn StdError>> {
    run_async(async move {
        // the adapter: accepts DAP requests and sends them over to the gdb process
        let mut stdout = StdoutWriter::new();
        let mut server = Server::new();
        let mut stdin = FileLineReader::new("session.txt").await;
        let mut driver_gdb = DriverGDB::new();
        let mut in_flight_requests = InFlightRequestQueue::new();

        // the session starts with the "initialize" request
        let request = server.accept_request(&mut stdin).await?;
        handle_initialise(request, &mut stdout).await;

        // we are now expecting "launch" request
        let request = server.accept_request(&mut stdin).await?;
        let mut gdb_output_channel = handle_launch(request, &mut driver_gdb, &mut stdout).await;

        // we are now reading accept configuration requests (set breakpoints)
        // until we hit the "configurationDone" request
        handle_configuration_stage(
            &mut server,
            &mut stdin,
            &mut driver_gdb,
            &mut stdout,
            &mut gdb_output_channel,
            &mut in_flight_requests,
        )
        .await;

        Ok::<(), Box<dyn StdError>>(())
    })
}
