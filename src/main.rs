use std::{error::Error, env};

mod parse;
mod anki;
mod vocab_importer;

use crate::parse::parse_topics_nested_iter;

// ============================================================================================
//                                          csv-to-anki
// ============================================================================================

fn main() {
    println!("Hello, world!");

    if let Err(e) = run() {
        println!("Error: {}", e)
    }
}

#[inline]
fn get_file_path(args: env::Args) -> Option<String> {
    args.into_iter().nth(1)
}

fn run() -> Result<(), Box<dyn Error>> {
    if let Some(path) = get_file_path(env::args()) {
        for topic in parse_topics_nested_iter(&path)? {
            let topic = topic?;
            println!("Topic: {}", topic.name());

            for word in topic.words()? {
                let word = word?;
                println!("  - {}", word.japanese())
            }
        }

        return Ok(());
    }

    return Err("No file path specified".into());
}
