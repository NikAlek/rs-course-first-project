use lib::console::commands::Resource;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, stdin, stdout, Read, Write};

fn write(resource: &Resource) -> Result<Box<dyn Write>, io::Error> {
    match resource {
        Resource::Console => Ok(Box::new(stdout())),
        Resource::File{path} => {
            let file = File::create(path)?;
            Ok(Box::new(BufWriter::new(file)))
        }
    }
}