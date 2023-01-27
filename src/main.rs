use dap::line_reader::FileLineReader;
use dap::prelude::*;
mod libdapd;
use crate::libdapd::run_async;
use libdapd::IdeAcceptor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async move {
        let adapter = IdeAcceptor::default();
        let mut server = Server::new(adapter, StdoutWriter::new());

        let mut reader = FileLineReader::new("session.txt").await;
        let _ = server.run(&mut reader).await;
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StdoutWriter;

    #[test]
    fn test_initialise_request_from_file() -> Result<(), Box<dyn std::error::Error>> {
        run_async(async move {
            let adapter = IdeAcceptor::default();
            let transport = StdoutWriter::new();
            let mut server = Server::new(adapter, transport);
            let mut reader = FileLineReader::new("initialize.txt").await;
            tracing::debug!("input file opened!");
            if let Err(e) = server.run(&mut reader).await {
                tracing::debug!("{}", e.to_string());
            }
        })
    }
}
