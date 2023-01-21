use dap::prelude::*;
use std::fs::File;
use std::io::{BufReader, BufWriter};

mod ide;
use ide::IdeAcceptor;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let adapter = IdeAcceptor::default();
    let transport = BasicClient::new(BufWriter::new(std::io::stdout()));
    let mut server = Server::new(adapter, transport);

    let infile = File::open("testinput.txt")?;
    let mut reader = BufReader::new(infile);
    server.run(&mut reader)?;
    Ok(())
}
