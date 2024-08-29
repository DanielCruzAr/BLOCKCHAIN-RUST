use std::process::exit;
use std::vec;

use bitcoincash_addr::Address;
use clap::{Arg, Command};
use crate::blockchain::Blockchain;
use crate::errors::Result;
use crate::server::Server;
use crate::transaction::Transaction;
use crate::utxoset::UTXOSet;
use crate::wallet::Wallets;

pub struct Cli {}

impl Cli {
    pub fn new() -> Result<Cli> {
        Ok(Cli {})
    }

    pub fn run(&mut self) -> Result<()> {
        let matches = Command::new("blockchain-rust-demo")
            .version("0.1")
            .author("cruzarciniega.d@gmail.com")
            .about("Blockchain in rust: a simple blockchain for learning")
            .subcommand(
                Command::new("create")
                    .about("Creates a new blockchain")
                    .arg(Arg::new("ADDRESS")
                    .help("Address to receive the genesis block reward")
                    .required(true)),
            )
            .subcommand(
                Command::new("getbalance")
                    .about("Get balance of the address")
                    .arg(Arg::new("ADDRESS")
                    .help("Address to get balance")
                    .required(true)),
            )
            .subcommand(
                Command::new("send")
                    .about("Send amount of coins to address")
                    .arg(Arg::new("FROM")
                    .required(true))
                    .arg(Arg::new("TO")
                    .required(true))
                    .arg(Arg::new("AMOUNT")
                    .required(true))
                    .arg(Arg::new("--mine")
                    .required(false)),
            )
            .subcommand(
                Command::new("startnode")
                    .about("Start the node server")
                    .arg(Arg::new("PORT")
                    .required(true)),
            )
            .subcommand(
                Command::new("startminer")
                    .about("Start the miner server")
                    .arg(Arg::new("PORT")
                    .required(true))
                    .arg(Arg::new("ADDRESS")
                    .required(true)),

            )
            .subcommand(Command::new("reindex").about("Rebuilds the UTXO set"))
            .subcommand(Command::new("printchain").about("Print all the blocks of the blockchain"))
            .subcommand(Command::new("createwallet").about("Create a new wallet"))
            .subcommand(Command::new("listaddresses").about("List all addresses"))
            .get_matches();
        
        if let Some(ref matches) = matches.subcommand_matches("startnode") {
            if let Some(port) = matches.get_one::<String>("PORT") {
                let bc = Blockchain::new()?;
                let utxo_set = UTXOSet { blockchain: bc };
                let server = Server::new(port, "", utxo_set)?;
                server.start_server()?;
            }
        }

        if let Some(ref matches) = matches.subcommand_matches("startminer") {
            let port = if let Some(port) = matches.get_one::<String>("PORT") {
                port
            } else {
                println!("PORT not supply!: usage");
                exit(1)
            };

            let address = if let Some(address) = matches.get_one::<String>("ADDRESS") {
                address
            } else {
                println!("ADDRESS not supply!: usage");
                exit(1)
            };
            let bc = Blockchain::new()?;
            let utxo_set = UTXOSet { blockchain: bc };
            let server = Server::new(port, address, utxo_set)?;
            server.start_server()?;
        }

        if let Some(ref matches) = matches.subcommand_matches("create") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let address = String::from(address);
                let bc = Blockchain::create_blockchain(address.clone())?;
                let utxo_set = UTXOSet { blockchain: bc };
                utxo_set.reindex()?;
                print!("create blockchain \n")
            }
            /*else {
                print!("Not printing testing lists...");
            }*/
        }
    
        if let Some(ref matches) = matches.subcommand_matches("getbalance") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let pub_key_hash = Address::decode(address).unwrap().body;
                let bc = Blockchain::new()?;
                // let utxos = bc.find_utxo(&pub_key_hash);
                let utxo_set = UTXOSet { blockchain: bc };
                let utxos = utxo_set.find_utxo(&pub_key_hash)?;
                let mut balance = 0;
                
                for out in utxos.outputs {
                    balance += out.value;
                }
                println!("Balance of '{}': {}", address, balance);
            }
        }
    
        if let Some(ref matches) = matches.subcommand_matches("send") {
            let from = if let Some(address) = matches.get_one::<String>("FROM") {
                address
            } else {
                println!("from not supply!: usage");
                exit(1)
            };

            let to = if let Some(address) = matches.get_one::<String>("TO") {
                address
            } else {
                println!("from not supply!: usage");
                exit(1)
            };

            let amount: i32 = if let Some(amount) = matches.get_one::<String>("AMOUNT") {
                amount.parse()?
            } else {
                println!("from not supply!: usage");
                exit(1)
            };

            if matches.contains_id("mine") {
                cmd_send(from, to, amount, true)?;
            } else {
                cmd_send(from, to, amount, false)?;
            }
        }

        if let Some(_) = matches.subcommand_matches("reindex") {
            let bc = Blockchain::new()?;
            let utxo_set = UTXOSet { blockchain: bc };
            utxo_set.reindex()?;
            let count = utxo_set.count_transactions()?;
            print!("Done! There are {} transactions in the UTXO set.", count);
        }
        
        if let Some(ref _matches) = matches.subcommand_matches("printchain") {
            cmd_print_chain()?;
        }

        if let Some(_) = matches.subcommand_matches("createwallet") {
            let mut ws = Wallets::new()?;
            let address = ws.create_wallet();
            ws.save_all()?;
            print!("Wallet created: {}\n", address);
        }

        if let Some(_) = matches.subcommand_matches("listaddresses") {
            let ws = Wallets::new()?;
            let addresses = ws.get_all_addresses();
            for address in addresses {
                println!("{}", address);
            }
        }

        Ok(())
    }
}

fn cmd_send(from: &str, to: &str, amount: i32, mine_now: bool) -> Result<()> {
    let bc = Blockchain::new()?;
    let mut utxo_set = UTXOSet { blockchain: bc };
    let wallets = Wallets::new()?;
    let wallet = wallets.get_wallet(from).unwrap();
    let tx = Transaction::new_utxo(wallet, to, amount, &utxo_set)?;
    if mine_now {
        let cbtx = Transaction::new_coinbase(from.to_string(), String::from("reward!"))?;
        let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;

        utxo_set.update(&new_block)?;
    } else {
        Server::send_transaction(&tx, utxo_set)?;
    }

    print!("success! \n");
    Ok(())
}

fn cmd_print_chain() -> Result<()> {
    let bc = Blockchain::new()?;
    for b in bc.iter() {
        println!("{:#?}", b);
    }
    Ok(())
}