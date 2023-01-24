use dap::prelude::*;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::{result::Result, error::Error};

mod libdapd;
use libdapd::IdeAcceptor;

fn main() -> Result<(), Box<dyn Error>> {
    let adapter = IdeAcceptor::default();
    let transport = BasicClient::new(BufWriter::new(std::io::stdout()));
    let mut server = Server::new(adapter, transport);

    let infile = File::open("testinput.txt")?;
    let mut reader = BufReader::new(infile);
    server.run(&mut reader)?;
    Ok(())
}
