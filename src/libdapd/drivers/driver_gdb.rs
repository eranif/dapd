use crate::events::ProcessEventBody;
use crate::requests::{LaunchRequestArguments, SetBreakpointsArguments};
use crate::types::ProcessEventStartMethod;
use gdb::Record;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::{error::Error as StdError, result::Result};
use tokio::sync::mpsc::Receiver;

#[allow(dead_code)]
pub enum Message {
    Launch(LaunchRequestArguments),
}

#[allow(dead_code)]
pub struct DriverGDB {
    /// Used by the contol thread to send commands to gdb
    gdb_process: Option<gdb::Debugger>,
}

impl DriverGDB {
    pub fn new() -> Self {
        Self { gdb_process: None }
    }

    /// Send command to gdb
    #[allow(dead_code)]
    pub async fn send_cmd_raw(&mut self, cmd: &str) -> Result<(), Box<dyn StdError>> {
        let Some(gdb_process) = &mut self.gdb_process else {
            return Err(Box::new(IoError::new(IoErrorKind::InvalidInput, "gdb is not running")));
        };

        gdb_process.send_cmd_raw(cmd).await;
        Ok(())
    }

    /// Launch gdb, but do not start the debug session
    pub async fn launch(
        &mut self,
        launch_args: &LaunchRequestArguments,
    ) -> Result<(ProcessEventBody, Receiver<Record>), Box<dyn StdError>> {
        let Some(exepath) = &launch_args.program else {
            return Err(make_error_from_string(IoErrorKind::InvalidInput, "missing executable"));
        };
        // always use forward slash
        let exepath = exepath.replace("\\", "/");
        let Ok((mut gdb_process, mut output_channel)) = gdb::Debugger::start().await else {
            return Err(make_error_from_string(IoErrorKind::InvalidInput, "failed to start gdb debugger!"));
        };

        // Load the executable
        let _ = gdb_process
            .send_cmd_raw(&format!(r#"-file-exec-and-symbols "{}""#, exepath))
            .await;
        let res = gdb_process.read_result_record(&mut output_channel).await;
        if res.class != gdb::ResultClass::Done {
            return Err(make_error_from_string(
                IoErrorKind::InvalidData,
                &format!("{:?}", res.content),
            ));
        }
        tracing::debug!("gdb process started and loaded executable successfully!");
        self.gdb_process = Some(gdb_process);
        tracing::trace!("{:?}", res);
        Ok((
            ProcessEventBody {
                name: exepath.clone(),
                system_process_id: Some(-1),
                is_local_process: Some(true),
                start_method: Some(ProcessEventStartMethod::Launch),
                pointer_size: Some(8),
            },
            output_channel,
        ))
    }

    /// Set or remove breakpoints
    pub async fn set_breakpoints(
        &mut self,
        _args: &SetBreakpointsArguments,
    ) -> Result<(), Box<dyn StdError>> {
        Ok(())
    }
}

#[allow(dead_code)]
fn make_error(gdb_err: gdb::Error) -> Box<dyn StdError> {
    match gdb_err {
        gdb::Error::IOError(e) => Box::new(e),
        gdb::Error::ParseError => Box::new(IoError::new(
            IoErrorKind::InvalidData,
            "failed to parse gdb output!",
        )),
        gdb::Error::IgnoredOutput => Box::new(IoError::new(IoErrorKind::Other, "output ignored")),
    }
}

fn make_error_from_string(io_kind: IoErrorKind, msg: &str) -> Box<dyn StdError> {
    Box::new(IoError::new(io_kind, msg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libdapd::run_async;

    #[test]
    fn test_launch_gdb() -> Result<(), Box<dyn std::error::Error>> {
        run_async(async move {
            let mut driver = DriverGDB::new();
            let launch_args = LaunchRequestArguments {
                no_debug: Some(false),
                restart_data: None,
                program: Some(
                    r#"C:\Users\eran\Documents\HellWorldCxx\build-Debug\bin\HellWorldCxx.exe"#
                        .to_string(),
                ),
                args: None,
                cwd: None,
                env: None,
            };
            let _ = driver.launch(&launch_args).await;
        })
    }
}
