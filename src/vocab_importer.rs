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
    
    pub fn new(deck_name: impl Into<String>) -> Self {
        JapaneseVocabImporter {
            client: AnkiConnectClient::new(),
            deck_name: deck_name.into(),
            model_name: "Basic".to_string()  // <--- will add support for other models later
        }
    }


}