use crate::requests::LaunchRequestArguments;

pub trait Driver {
    /// Start the driver.
    ///
    /// This method does not start the debugger, it only loads the debuggee
    /// into the debugger
    fn start(&mut self, launch_args: &LaunchRequestArguments);
}
