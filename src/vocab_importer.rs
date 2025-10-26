use crate::{anki::{AnkiConnectClient, NoteFields, Note}, parse::{Word, Topic}};
use std::error::Error;

// ============================================================================================
//                          High-Level API for Japanese Vocabularly
// ============================================================================================

// TODO: 
// Bulk import - import_topicS, add_noteS 

pub struct JapaneseVocabImporter {
    pub client: AnkiConnectClient,
    deck_name: String,
    model_name: String,
}

impl JapaneseVocabImporter {
    
    /// create a new importer with default settings
    pub fn new(deck_name: impl Into<String>) -> Self {
        JapaneseVocabImporter {
            client: AnkiConnectClient::new(),
            deck_name: deck_name.into(),
            model_name: "Basic".to_string()  // <--- will add support for other models later
        }
    }

    /// Set a custom note type/model
    pub fn with_model(mut self, model_name: impl Into<String>) -> Self {
        self.model_name = model_name.into();
        self
    }

    /// Set a custom AnkiConnect URl
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.client = AnkiConnectClient::with_url(url);
        self
    }

    pub fn initialise(&self) -> Result<(), Box<dyn Error>> {
        // Check connection
        self.client.check_connection()
            .map_err(
                |e| 
                format!("Cannot connect to to Anki. Is Anki running with AnkiConnect installed? Error: {}", e)
            )?;

        println!("Success: Connected to Anki");

        // create deck (won't fail if it exists)
        self.client.create_deck(&self.deck_name)?;

        println!("Success: Deck '{}' ready", self.deck_name);

        Ok(())
    }


    pub fn initialise_with_topics(&self, topics: &[Topic]) -> Result<(), Box<dyn Error>> {
        self.client.check_connection()
            .map_err(
                |e|
                format!("Cannot connect to to Anki. Is Anki running with AnkiConnect installed? Error: {}", e)
            )?;

        println!("Success: Connected to Anki");
        
        self.client.create_deck(&self.deck_name)?;

        println!("Success: Main Deck '{}' ready", self.deck_name);

        println!("\nCreating subdecks for topics: ");
        for topic in topics {
            let subdeck_name = format!("{}::{}", self.deck_name, topic.name());
            let deck_id = self.client.create_deck(&subdeck_name)?;
            println!("  Success: Created - '{}', id = {}", subdeck_name, deck_id);
        }

        Ok(())
    }

    /// Convert a Word to an Anki Note
    /// Creates a subdeck for each topic using :: notation
    /// 
    /// 
    /// front: kanji, if present, else japanese
    /// back: if front = kanji, japanese + english, else just english
    pub fn word_to_note(&self, word: &Word, topic: &str) -> Note {
        let full_deck_name = if topic.is_empty() {
            self.deck_name.clone()
        } else {
            format!("{}::{}", self.deck_name, topic)
        };


        let front = if word.kanji().trim().is_empty() {
            word.japanese().clone()
        } else {
            word.kanji().clone()
        };

        let back = if word.kanji().trim().is_empty() {
            word.english().clone()
        } else {
            word.japanese().clone() + " | " + &word.english().clone()
        };


        Note {
            deck_name: full_deck_name,
            model_name: self.model_name.clone(),
            fields: NoteFields {
                front: front,
                back: back,
            },
            tags: vec![topic.to_string(), "japanese".to_string(), "vocabularly".to_string()]
            .into_iter().filter(|t| !t.is_empty()).collect(),
            audio: None,
            picture: None,
        }
    }

    /// Import a single word
    pub fn import_word(&self, word: &Word, topic_name: &str) -> Result<i64, Box<dyn Error>> {
        let note = self.word_to_note(word, topic_name);
        self.client.add_note(note)
    }

    pub fn import_words(&self, topic: &Topic) -> Result<Vec<Result<i64, String>>, Box<dyn Error>> {
        let notes: Vec<Note>= topic.words().iter().map(|word| {
            self.word_to_note(word, topic.name())
        }).collect();

        self.client.add_notes(notes)
    }

    fn retry_error_word(&self, word: &Word, topic_name: &str) -> Result<i64, Box<dyn Error>> {
        self.import_word(word, topic_name)
    }

    /// import all words for a topic
    /// 
    /// 1. create deck
    /// 2. populate deck
    pub fn import_topic(&self, topic: &Topic) -> Result<ImportResult, Box<dyn Error>> {
        let mut result = ImportResult::new(&topic.name());

        for word in topic.words() {
            match self.import_word(word, topic.name()) {
                Ok(note_id) => {
                    result.added += 1;

                    if word.kanji().trim().is_empty() {
                        println!("  Success: Added - {} -> {}, id = {}", word.kanji(), word.english(), note_id);
                    } else {
                        println!("  Success: Added - {} -> {}, id = {}", word.japanese(), word.english(), note_id);
                    }

                },

                Err(e) if e.to_string().contains("Duplicate") => {
                    result.duplicates += 1;
                    println!("  Error: Duplicate no. {} - {}", result.duplicates, word.japanese());
                },

                Err(e) => {
                    result.errors += 1;
                    println!(" Error: adding {}: {}. Retrying...", word.japanese(), e);
                }
            }
        }

        Ok(result)
    }


    /// import all topics
    pub fn import_all_topics(&self, topics: &[Topic]) -> Result<Vec<ImportResult>, Box<dyn Error>> {
        let mut results: Vec<ImportResult> = Vec::new();

        for topic in topics {
            println!("\nImporting topic: {}", topic.name());
            let result = self.import_topic(topic)?;
            results.push(result);
        }

        Ok(results)
    }
}

pub struct ImportResult {
    pub topic_name: String,
    pub added: usize,
    pub duplicates: usize,
    pub errors: usize,
}

impl ImportResult {
    fn new(topic_name: &str) -> Self {
        ImportResult { 
            topic_name: topic_name.to_string(), 
            added: 0, 
            duplicates: 0, 
            errors: 0 
        }
    }

    // fn id(mut self, deck_id: i64) -> Self {
    //     self.deck_id = deck_id;
    //     self
    // }  

    pub fn total(&self) -> usize {
        self.added + self.duplicates + self.errors
    }

    pub fn print_summary(&self) {
        println!("\n{} Summary: ", self.topic_name);
        println!("  Added: {}", self.added);
        println!("  Duplicates: {}", self.duplicates);
        println!("  Errors: {}", self.errors);
        println!("  Total: {}", self.total());
    }
}