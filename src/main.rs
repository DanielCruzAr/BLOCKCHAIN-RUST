mod block;
mod blockchain;
mod transaction;
mod tx;
mod errors;
mod cli;
mod wallet;
mod utxoset;
mod server;

use errors::Result;
use cli::Cli;

fn main() -> Result<()> {
    let mut cli = Cli::new()?;
    cli.run()?;

    Ok(())
}
