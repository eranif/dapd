use crate::events::ProcessEventBody;
use crate::requests::LaunchRequestArguments;
use async_trait::async_trait;
use std::{error::Error as StdError, result::Result};

#[allow(dead_code)]
pub enum Message {
    Launch(LaunchRequestArguments),
}

#[async_trait]
pub trait Driver {
    /// Start the driver.
    ///
    /// This method does not start the debugger, it only loads the debuggee
    /// into the debugger
    async fn launch(
        &mut self,
        launch_args: &LaunchRequestArguments,
    ) -> Result<ProcessEventBody, Box<dyn StdError>>;
}
