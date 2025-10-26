use csv_partitioner::{CsvSliceParser, FromColumnSlice};
use std::{error::Error, sync::Arc, env};

mod anki;


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

#[derive(Debug, Clone)]
struct Topic {
    name: String,
    words: Vec<Word>,
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