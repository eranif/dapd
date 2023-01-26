use crate::events::ProcessEventBody;
#[allow(unused_imports)]
use crate::libdapd::{drivers::Message, Driver};
use crate::requests::LaunchRequestArguments;
use crate::types::ProcessEventStartMethod;
use async_trait::async_trait;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::{error::Error as StdError, result::Result};

#[allow(unused_imports)]
use crossbeam_channel::{Receiver, Sender};
//use std::thread;

#[allow(dead_code)]
pub struct DriverGDB {
    /// Used by the contol thread to send commands to gdb
    debugger: Option<gdb::Debugger>,
}

impl DriverGDB {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { debugger: None }
    }
}

impl DriverGDB {
    async fn send_cmd_raw(
        &mut self,
        cmd: &str,
    ) -> Result<gdb::MessageRecord<gdb::ResultClass>, Box<dyn StdError>> {
        let Some(debugger) = &mut self.debugger else {
            return Err(Box::new(IoError::new(IoErrorKind::InvalidInput, "gdb is not running")));
        };

        debugger.send_cmd_raw(cmd).await;
        Ok(debugger.read_result_record().await)
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
#[async_trait]
impl Driver for DriverGDB {
    async fn launch(
        &mut self,
        launch_args: &LaunchRequestArguments,
    ) -> Result<ProcessEventBody, Box<dyn StdError>> {
        let Some(exepath) = &launch_args.program else {
            return Err(make_error_from_string(IoErrorKind::InvalidInput, "missing executable"));
        };
        // always use forward slash
        let exepath = exepath.replace("\\", "/");
        let Ok(debugger) = gdb::Debugger::start().await else {
            return Err(make_error_from_string(IoErrorKind::InvalidInput, "failed to start gdb debugger!"));
        };

        // Load the executable
        self.debugger = Some(debugger);
        let res = self
            .send_cmd_raw(&format!(r#"-file-exec-and-symbols "{}"#, exepath))
            .await?;
        if res.class != gdb::ResultClass::Done {
            return Err(make_error_from_string(
                IoErrorKind::InvalidData,
                &format!("{:?}", res.content),
            ));
        }

        Ok(ProcessEventBody {
            name: exepath.clone(),
            system_process_id: Some(-1),
            is_local_process: Some(true),
            start_method: Some(ProcessEventStartMethod::Launch),
            pointer_size: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_launch_gdb() -> Result<(), Box<dyn StdError>> {
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
        let _ = driver.launch(&launch_args).await?;
        Ok(())
    }
}
