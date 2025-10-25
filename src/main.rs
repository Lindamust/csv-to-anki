use csv_partitioner::{CsvSliceParser, FromColumnSlice};
use std::{error::Error};
use std::sync::Arc;
use std::env;

fn main() {
    println!("Hello, world!");

    if let Err(e) = run() {
        println!("Error: {}", e)
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if let Some(path) = args.get(1) {
        for topic in parse_topics_nested_iter(&path)? {
            let topic = topic?;
            println!("Topic: {}", topic.name);

            for word in topic.words()? {
                let word = word?;
                println!("  - {}", word.japanese)
            }
        }

        return Ok(());
    }

    return Err("No file path specified".into());
}

#[derive(Debug, Clone)]
struct Word {
    japanese: String,
    english: String,
    kanji: String,
}

#[derive(Debug, Clone)]
struct Topic {
    name: String,
    words: Vec<Word>,
}

impl FromColumnSlice for Word {
    const COLUMN_COUNT: usize = 3;

    fn from_record(record: &csv::StringRecord, start_col: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let japanese = record.get(start_col)    
            .ok_or("Missing japanese field")?
            .to_string();

        let english = record.get(start_col + 1)    
            .ok_or("Missing english field")?
            .to_string();

        let kanji = record.get(start_col + 2)    
            .ok_or("Missing kanji field")?
            .to_string();

        Ok(Word { japanese, english, kanji })
    }
}


fn parse_topics_from_csv(file_path: &str) -> Result<Vec<Topic>, Box<dyn Error>> {
    let parser: CsvSliceParser = CsvSliceParser::from_file(file_path)
        .map_err(|e: Box<dyn Error>| format!("Failed to load CSV: {}", e))?;

    let slice_count: usize = parser.slice_count::<Word>();

    if slice_count == 0 {
        return Err("No valid slices found".into());
    }

    let mut topic_vec: Vec<Topic> = Vec::with_capacity(slice_count);  // low-hanging fruit optimisation

    for slice_idx in 0..slice_count {
        // get name
        let name = parser.headers()
            .get(slice_idx * Word::COLUMN_COUNT)
            .unwrap_or("")
            .to_string();

        // get words
        let words = parser.parse_slice::<Word>(slice_idx)?;

        topic_vec.push(Topic { name, words });
    }

    Ok(topic_vec)
}

fn parse_topics_from_csv_lazy(file_path: &str) -> Result<Vec<Topic>, Box<dyn Error>> {
    let parser = CsvSliceParser::from_file(file_path)?;

    (0..parser.slice_count::<Word>())
        .map(|i| Ok(Topic {
            name: parser.headers().get(i * Word::COLUMN_COUNT).unwrap_or("").to_string(),
            words: parser.parse_slice::<Word>(i)?,
        }))
        .collect()
}


struct TopicWithWordIter {
    name: String,
    parser: Arc<CsvSliceParser>,
    slice_index: usize,
}

impl TopicWithWordIter {
    fn words(&self) -> Result<impl Iterator<Item = Result<Word, Box<dyn Error>>> + '_, Box<dyn Error>> {
        self.parser.parse_slice_iter::<Word>(self.slice_index)
    }
}

fn parse_topics_nested_iter(file_path: &str)
    -> Result<impl Iterator<Item = Result<TopicWithWordIter, Box<dyn Error>>>, Box<dyn Error>> 
    {
        let parser = Arc::new(CsvSliceParser::from_file(file_path)?);
        let slice_count = parser.slice_count::<Word>();

        Ok(
            (0..slice_count).map(move |i: usize| {
                let topic_name = parser
                    .headers()
                    .get(i * Word::COLUMN_COUNT)
                    .unwrap_or("")
                    .to_string();

                Ok(TopicWithWordIter { 
                    name: topic_name, parser: Arc::clone(&parser), slice_index: i 
                })
            }))
    }
