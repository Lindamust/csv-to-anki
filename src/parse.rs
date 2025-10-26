use csv_partitioner::{CsvSliceParser, FromColumnSlice};
use std::{error::Error, sync::Arc};


#[derive(Debug, Clone)]
pub struct Word {
    japanese: String,
    english: String,
    kanji: String,
}

impl Word {
    pub fn japanese(&self) -> &String {
        &self.japanese
    }

    pub fn english(&self) -> &String {
        &self.english
    }

    pub fn kanji(&self) -> &String {
        &self.kanji
    }
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
pub struct Topic {
    name: String,
    words: Vec<Word>,
}

impl Topic {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn words(&self) -> &Vec<Word> {
        &self.words
    }
}


pub struct TopicWithWordIter {
    name: String,
    parser: Arc<CsvSliceParser>,
    slice_index: usize,
}

impl TopicWithWordIter {
    pub fn words(&self) -> Result<impl Iterator<Item = Result<Word, Box<dyn Error>>> + '_, Box<dyn Error>> {
        self.parser.parse_slice_iter::<Word>(self.slice_index)
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

pub fn parse_topics_nested_iter(file_path: &str)
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