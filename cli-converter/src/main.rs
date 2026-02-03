use crate::converter as converter_logic;
use clap::Parser;
use lib::console::commands::Cli;
use lib::console::commands::Commands;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ReadParseWriteCommand {
            from,
            from_format,
            to,
            to_format,
        } => {
            println!("Comparing:");
            println!("  File1: {:?} (format: {:?})", from, from_format);
            println!("  File2: {:?} (format: {:?})", to, to_format);

               let res  = converter_logic::logic::process_convert_logic(from, from_format, to, to_format);
                  println!("result : {:?}", res)
        }, 

        _ => {
              println!("Error. Work only with ReadParseWriteCommand");
        } 
    }
}

pub mod converter;