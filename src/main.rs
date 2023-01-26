use dap::line_reader::FileLineReader;
use dap::prelude::*;
use std::io::BufWriter;
use std::{error::Error, result::Result};
mod libdapd;
use libdapd::IdeAcceptor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let adapter = IdeAcceptor::default();
    let transport = BasicClient::new(BufWriter::new(std::io::stdout()));
    let mut server = Server::new(adapter, transport);

    let mut reader = FileLineReader::new("session.txt").await;
    server.run(&mut reader).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialise_request_from_file() -> Result<(), Box<dyn Error>> {
        tracing_subscriber::fmt::init();

        let adapter = IdeAcceptor::default();
        let transport = BasicClient::new(BufWriter::new(std::io::stdout()));
        let mut server = Server::new(adapter, transport);
        let mut reader = FileLineReader::new("initialize.txt").await;
        tracing::debug!("input file opened!");
        if let Err(e) = server.run(&mut reader).await {
            tracing::debug!("{}", e.to_string());
        }
        Ok(())
    }
}
