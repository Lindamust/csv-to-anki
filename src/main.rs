use std::result;
use std::{error::Error, env};

mod parse;
mod anki;
mod vocab_importer;

use csv_partitioner::{CsvSliceParser, FromColumnSlice};

use crate::parse::{Topic, Word};
use crate::vocab_importer::JapaneseVocabImporter;

// ============================================================================================
//                                          csv-to-anki
// ============================================================================================

fn main() -> Result<(), Box<dyn Error>> {
    run()?;

    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let path = get_file_path(env::args()).ok_or("No file path specified")?;

    let topics: Vec<Topic> = handle_parsing(&path)?;


    Ok(())
}


#[inline]
fn get_file_path(args: env::Args) -> Option<String> {
    args.into_iter().nth(1)
}

fn handle_parsing(file_path: &str) -> Result<Vec<Topic>, Box<dyn Error>> {
    println!("Step 1: Parsing CSV file...");
    let topics: Vec<Topic> = parse_topics_from_csv(file_path)?;

    println!("\nParsed {} topics:", topics.len());
    for topic in &topics {
        println!("  - {}: {} words", topic.name, topic.words.len());
    }

    Ok(topics)
}

fn parse_topics_from_csv(file_path: &str) -> Result<Vec<Topic>, Box<dyn Error>> {
    let parser = CsvSliceParser::from_file(file_path)?;

    Ok((0..parser.slice_count::<Word>())
        .filter_map(|slice_idx| {
            let topic_name: String = parser.headers()
                .get(slice_idx * Word::COLUMN_COUNT)?
                .to_string();

            // skip empty topic names
            if topic_name.trim().is_empty() {
                return None;
            }

            let words: Vec<Word> = parser.parse_slice::<Word>(slice_idx).ok()?;

            // skip empty word vecs
            if words.is_empty() {
                return None;
            }

            Some(Topic {
                name: topic_name,
                words,
            })
        })
        .collect::<Vec<_>>())
}



