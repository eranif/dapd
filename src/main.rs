use dap::line_reader::FileLineReader;
use dap::prelude::*;
mod libdapd;
use crate::libdapd::run_async;
use crate::libdapd::DriverGDB;
use crate::responses::SetBreakpointsResponse;
use std::error::Error as StdError;

/// handlers
use crate::libdapd::handle_initialise;
use crate::libdapd::handle_launch;

async fn handle_configuration_stage(
    server: &mut Server,
    stdin: &mut impl LineReader,
    driver: &mut DriverGDB,
    stdout: &mut StdoutWriter,
) {
    loop {
        tokio::select!(
            request = server.accept_request(stdin) => {
                // received request from the IDE
                if let Ok(request) = request {
                    match &request.command {
                        Command::SetBreakpoints(args) => {
                            tracing::debug!("> SetBreakpoints called");
                            let response = if let Ok(breakpoints) = driver.set_breakpoints(&args).await {
                                Response::make_success(
                                    &request,
                                    ResponseBody::SetBreakpoints(SetBreakpointsResponse { breakpoints }),
                                )
                            } else {
                                Response::make_error(&request, &format!("failed to set breakpoints"))
                            };
                            stdout.write(dap::client::Sendable::Response(response)).expect("failed to write to stdout!");
                        }
                        Command::SetFunctionBreakpoints(_args) => {
                            tracing::debug!("> SetFunctionBreakpoints called");
                            let response = Response::make_error(&request, "command SetFunctionBreakpoints is unsupported");
                            stdout.write(dap::client::Sendable::Response(response)).expect("failed to write to stdout!");
                        }
                        Command::ConfigurationDone => {
                            tracing::debug!("> ConfigurationDone called");
                            let response = Response::make_success(&request, ResponseBody::ConfigurationDone);
                            stdout.write(dap::client::Sendable::Response(response)).expect("failed to write to stdout!");
                            break;
                        }
                        _ => {
                            let response = Response::make_error(&request, "Can only accept configuration requests at this stage");
                            stdout.write(dap::client::Sendable::Response(response)).expect("failed to write to stdout!");
                        }
                    }
                }
            }
        )
    }
}

fn main() -> Result<(), Box<dyn StdError>> {
    run_async(async move {
        // the adapter: accepts DAP requests and sends them over to the gdb process
        let mut stdout = StdoutWriter::new();
        let mut server = Server::new();
        let mut stdin = FileLineReader::new("session.txt").await;
        let mut driver_gdb = DriverGDB::new();

        // the session starts with the "initialize" request
        let request = server.accept_request(&mut stdin).await?;
        handle_initialise(request, &mut stdout).await;

        // we are now expecting "launch" request
        let request = server.accept_request(&mut stdin).await?;
        let _gdb_output_channel = handle_launch(request, &mut driver_gdb, &mut stdout).await;

        // we are now reading accept configuration requests (set breakpoints)
        // until we hit the "configurationDone" request
        handle_configuration_stage(&mut server, &mut stdin, &mut driver_gdb, &mut stdout).await;

        Ok::<(), Box<dyn StdError>>(())
    })
}
