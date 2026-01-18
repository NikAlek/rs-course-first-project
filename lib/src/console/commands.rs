use crate::model::data::Format;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Debug)]
pub enum Resource {
    Console,
    File { path: PathBuf },
}

fn parse_resource(s: &str) -> Result<Resource, String> {
    if s == "console" {
        Ok(Resource::Console)
    } else if let Some(path) = s.strip_prefix("file:") {
        Ok(Resource::File { path: path.into() })
    } else {
        Err("Resource must be 'console' or 'file:<path>'".into())
    }
}

#[derive(Subcommand)]
pub enum Commands {
    CompareCommand {
        #[arg(long, required = true, value_parser = parse_resource)]
        first_from: Resource,

        #[arg(long, required = true)]
        first_format: Format,

        #[arg(long, required = true, value_parser = parse_resource)]
        second_from: Resource,

        #[arg(long, required = true)]
        second_format: Format,
    },

    ReadParseWriteCommand {
        #[arg(long, required = true, value_parser = parse_resource)]
        from: Resource,

        #[arg(long, required = true)]
        from_format: Format,

        #[arg(long, required = true, value_parser = parse_resource)]
        to: Resource,

        #[arg(long, required = true)]
        to_format: Format,
    },
}
