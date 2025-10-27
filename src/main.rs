#[allow(dead_code)]

use std::{error::Error, env};

mod parse;
mod anki;
mod vocab_importer;

use csv_partitioner::{CsvSliceParser, FromColumnSlice};

use crate::parse::{Topic, Word};
use crate::vocab_importer::{ImportResult, JapaneseVocabImporter};

// ============================================================================================
//                                          csv-to-anki
// ============================================================================================

fn main() -> Result<(), Box<dyn Error>> {
    run()?;

    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let (path, deck_name) = get_inputs()?;

    println!("Step 1: Parsing CSV file...");
    let topics: Vec<Topic> = handle_parsing(&path)?;

    println!("\nStep 2: Creating Anki importer...");
    let importer = JapaneseVocabImporter::new(deck_name);

    println!("\nStep 3: Initializing connection to Anki...");
    connect_to_anki(&importer)?;

    println!("\nStep 4: Building sub-decks in Anki...");
    build_sub_decks(&importer, &topics)?;

    println!("\nStep 5: Populating decks with vocabulary in Anki...");
    let results: Vec<ImportResult> = importer.import_all_topics(&topics)?;

    display_import_results(results);

    Ok(())
}

fn build_sub_decks(importer: &JapaneseVocabImporter, topics: &[Topic]) -> Result<(), Box<dyn Error>> {
    importer.initialise_with_topics(&topics)?;

    Ok(())
}

fn connect_to_anki(importer: &JapaneseVocabImporter) -> Result<(), Box<dyn Error>> {
    importer.client.check_connection()
        .map_err(
            |e|
            format!("Cannot connect to to Anki. Is Anki running with AnkiConnect installed? Error: {}", e)
        )?;

    Ok(())
}

fn get_inputs() -> Result<(String, String), Box<dyn Error>> {
    let mut args = env::args();
    args.next(); // skip first argument (program name)

    let file_path = args.next()
        .ok_or(format!("Error: Missing file path argument.\nUSAGE: [path to input] [desired deck name]"))?;

    let deck_name = args.next()
        .ok_or(format!("Error: Missing deck name argument.\nUSAGE: [path to input] [desired deck name]"))?;

    Ok((file_path, deck_name))
}

fn handle_parsing(file_path: &str) -> Result<Vec<Topic>, Box<dyn Error>> {
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


fn display_import_results(results: Vec<ImportResult>) {
    println!("\n========================================");
    println!("IMPORT COMPLETE");
    println!("========================================");
    
    // for result in &results {
    //     result.print_summary();
    // }

    let total_added: usize = results.iter().map(|r| r.added).sum();
    let total_duplicates: usize = results.iter().map(|r| r.duplicates).sum();
    let total_errors: usize = results.iter().map(|r| r.errors).sum();
    
    println!("\nOverall Summary:");
    println!("  ✓ Successfully added: {}", total_added);
    println!("  ⊘ Duplicates skipped: {}", total_duplicates);
    println!("  ✗ Errors: {}", total_errors);
}