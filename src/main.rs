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

    let mut reader = FileLineReader::new("inputfile.txt").await;
    server.run(&mut reader).await?;
    Ok(())
}
