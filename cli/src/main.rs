use clap::Parser;
use lib::console::interprier::Cli;
use lib::console::interprier::Commands;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::CompareCommand {
            first_from,
            first_format,
            second_from,
            second_format,
        } => {
            println!("Converting:");
            println!("  Input: {:?} (format: {:?})", first_from, first_format);
            println!("  Input: {:?} (format: {:?})", second_from, second_format);
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
