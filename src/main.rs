mod block;
mod blockchain;
mod transaction;
mod errors;
mod cli;

use errors::Result;
use cli::Cli;

fn main() -> Result<()> {
    let mut cli = Cli::new()?;
    cli.run()?;

    Ok(())
}
