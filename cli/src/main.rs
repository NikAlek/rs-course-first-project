use crate::comparer::logic;
use clap::Parser;
use lib::console::commands::Cli;
use lib::console::commands::Commands;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::CompareCommand {
            first_from,
            first_format,
            second_from,
            second_format,
        } => {
            println!("Comparing:");
            println!("  Input: {:?} (format: {:?})", first_from, first_format);
            println!("  Input: {:?} (format: {:?})", second_from, second_format);

            let res = logic::process_comparer_logic(first_from, first_format, second_from, second_format);

            println!("result : {:?}", res)
        }

        Commands::ReadParseWriteCommand {
            from,
            from_format,
            to,
            to_format,
        } => {
            println!("Comparing:");
            println!("  File1: {:?} (format: {:?})", from, from_format);
            println!("  File2: {:?} (format: {:?})", to, to_format);
        }
    }
}

pub mod comparer;
pub mod converter;
pub mod utils;
