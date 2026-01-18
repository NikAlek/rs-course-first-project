use lib::console::commands::Resource;
use lib::model::errors::ParserErr;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, stdin, stdout, Read, Write};

pub fn read(resource: &Resource) -> Result<Box<dyn Read>, ParserErr> {
    
    match resource {
        Resource::Console => Ok(Box::new(stdin())),
        Resource::File{path} => {
            let file = File::open(path).map_err(|e| ParserErr::ParseErr { msg:e.to_string() })?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}