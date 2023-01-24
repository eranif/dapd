use crate::events::ProcessEventBody;
use crate::libdapd::{drivers::Message, Driver};
use crate::requests::LaunchRequestArguments;
use crate::types::ProcessEventStartMethod;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::{error::Error as StdError, result::Result};

use crossbeam_channel::{Receiver, Sender};
use std::thread;

#[allow(dead_code)]
pub struct DriverGDB {
    /// Used by the contol thread to send commands to gdb
    in_write: Option<Sender<Message>>,
    /// Used by gdb to read commands
    in_read: Option<Receiver<Message>>,
}

impl DriverGDB {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<Message>();
        Self {
            in_write: Some(sender),
            in_read: Some(receiver),
        }
    }
}

impl Driver for DriverGDB {
    fn launch(
        &mut self,
        launch_args: &LaunchRequestArguments,
    ) -> Result<ProcessEventBody, Box<dyn StdError>> {
        let Some(exepath) = &launch_args.program else {
            return Err(Box::new(IoError::new(IoErrorKind::InvalidInput, "missing executable")));
        };

        thread::spawn(|| {});
        Ok(ProcessEventBody {
            name: exepath.clone(),
            system_process_id: Some(-1),
            is_local_process: Some(true),
            start_method: Some(ProcessEventStartMethod::Launch),
            pointer_size: None,
        })
    }
}
