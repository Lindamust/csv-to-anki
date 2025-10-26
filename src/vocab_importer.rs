use crate::{anki::{AnkiConnectClient, NoteFields, Note}, parse::Word};
use std::error::Error;

// ============================================================================================
//                          High-Level API for Japanese Vocabularly
// ============================================================================================

pub struct JapaneseVocabImporter {
    client: AnkiConnectClient,
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
                format!("Cannot connectio to Anki. Is Anki running with AnkiConnect installed? Error: {}", e)
            )?;

        println!("Success: Connected to Anki");

        // create deck (won't fail if it exists)
        self.client.create_deck(&self.deck_name)?;

        println!("Success: Deck '{}' ready", self.deck_name);

        Ok(())
    }

    /// Convert a Word to an Anki Note
    pub fn word_to_note(&self, word: &Word, topic: &str) -> Note {
        Note {
            deck_name: self.deck_name.clone(),
            model_name: self.model_name.clone(),
            fields: NoteFields {
                front: word.japanese().clone(),
                back: word.english().clone(),
                kanji: word.kanji().clone(),
            },
            tags: vec![topic.to_string(), "japanese".to_string(), "vocabularly".to_string()],
            audio: None,
            picture: None,
        }
    }



}