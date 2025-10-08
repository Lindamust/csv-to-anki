#![allow(dead_code)]

//! made this specifically to parse a excel spreadsheet of japanese
//! vocabulary that my teacher gave
//! 
//! takes a csv whose headers are topics seperated by 2 blank fields
//! with rows that follow the pattern:  word, translation, kanji
//! 
//! as such:
//! verbs  ,                        ,       , adjectives,     ,
//! おどろく, to be surprised        , 驚く  , はやい     , fast, 早い
//! みえる  , able to see (naturally), 見える, おそい     , slow, 遅い 
//! 
//! or generally, in brackets is omitted
//! topic1, (translation), (kanji), topic2, (translation), (kanji), ...
//! t1_word1, t1_translation1, t1_kanji1, t2_word2, t2_word2, t2_kanji2 ...
//! 
//! 

use std::{error::Error, env, process};
use csv::{ReaderBuilder};
use csv_partitioner::{row_to_groups, CsvPartitioner};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        println!("Program name {}", args[0]);
        println!("Arguments provided: ");
        for (i, arg) in args.iter().skip(1).enumerate() {
            println!("{}: {}", i + 1, arg);
        }
    } else {
        println!("No arguments provided");
        process::exit(1);
    }

    let path = &args[1];

    if let Err(e) = run(&path) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }



    Ok(())
}

fn run(file_path: &str) -> Result<(), Box<dyn Error>> {
    let topics: Vec<Topic> = parse_to_populated_topics(file_path)?;

    for topic in topics {
        println!("{:?}:", topic.name);
        for word in topic.words {
            println!("{}, {}, {}", word.japanese, word.translation, word.kanji)
        }
    }




    Ok(())
}

#[derive(Debug, PartialEq)]
struct Word {
    pub japanese: String,
    pub translation: String,
    pub kanji: String,
}

#[derive(Debug, PartialEq)]
struct Topic {
    pub name: String,
    pub words: Vec<Word>,
}

fn parse_to_populated_topics(file_path: &str) -> Result<Vec<Topic>, Box<dyn Error>> {
    let rdr = ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(file_path)?;

    let mut csvp = CsvPartitioner::new_from_header(rdr)?;

    // initialise topics
    let mut topics: Vec<Topic> = csvp.get_partitions_owned() // ignore this error, my library is weird
        .iter()
        .map(|p| Topic { name: p.name.clone(), words: Vec::new() })
        .collect();

    // populate topics
    csvp.for_each_row(|row, partitions| {
        let groups = row_to_groups(row, partitions);

        for (i, group) in groups.iter().enumerate() {
            let word = Word {
                japanese: group.get(0).unwrap_or("").to_string(),
                translation: group.get(1).unwrap_or("").to_string(),
                kanji: group.get(2).unwrap_or("").to_string(),
            };

            topics[i].words.push(word);
        }
    })?;
    Ok(topics)
}





