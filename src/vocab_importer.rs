// ============================================================================================
//                          High-Level API for Japanese Vocabularly
// ============================================================================================

use crate::anki::AnkiConnectClient;

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

    
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.client = AnkiConnectClient::with_url(url);
        self
    }


}