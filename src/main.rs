mod block;
mod blockchain;
mod transaction;
mod tx;
mod errors;
mod cli;
mod wallet;

use errors::Result;
use cli::Cli;

fn main() -> Result<()> {
    let mut cli = Cli::new()?;
    cli.run()?;

    Ok(())
}
