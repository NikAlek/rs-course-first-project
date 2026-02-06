use crate::comparer as comparer_logic;
use clap::Parser;
use lib::console::commands::Cli;
use lib::console::commands::Commands;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::CompareCommand {
            first_from,
            first_format,
            second_from,
            second_format,
        } => {
            println!("Comparing:");
            println!("  Input: {:?} (format: {:?})", first_from, first_format);
            println!("  Input: {:?} (format: {:?})", second_from, second_format);

            let res = comparer_logic::logic::process_comparer_logic(first_from, first_format, second_from, second_format);

            println!("result : {:?}", res)
        },

          _ => {
              println!("Error. Work only with CompareCommand");
        } 

        
    }
}

pub mod comparer;