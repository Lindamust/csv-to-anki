use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_json;
use reqwest::{self};



// ============================================================================================
//                                  AnkiConnect API Structures
// ============================================================================================


/// Main request structure for AnkiConnect
#[derive(Debug, Serialize)]
struct AnkiRequest<T> {
    action: String,
    version: u32,
    params: T,
}

impl<T> AnkiRequest<T> {
    fn new(action: impl Into<String>, params: T) -> Self {
        AnkiRequest { 
            action: action.into(), 
            version: 6,     // AnkiConnect API version
            params 
        }
    }
}

/// Generic response structure
#[derive(Debug, Deserialize)]
struct AnkiResponse<T> {
    result: Option<T>,
    error: Option<String>,
}

/// Parameters for adding a note
#[derive(Debug, Serialize)]
struct AddNoteParams {
    note: Note
}

/// Anki note structure
#[derive(Debug, Serialize, Clone)]
pub struct Note {
    #[serde(rename = "deckName")]
    pub(crate) deck_name: String,

    #[serde(rename = "modelName")]
    pub(crate) model_name: String,

    pub(crate) fields: NoteFields,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tags: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) audio: Option<Vec<AudioField>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) picture: Option<Vec<PictureField>>,
}


/// Note fields for Japanese vocabularly
#[derive(Debug, Serialize, Clone)]
pub struct NoteFields {
    #[serde(rename = "Front")]
    pub(crate) front: String,

    #[serde(rename = "Back")]
    pub(crate) back: String,
}


#[derive(Debug, Serialize, Clone)]
pub struct AudioField {
    url: String,
    filename: String,
    fields: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PictureField {
    url: String,
    filename: String,
    fields: Vec<String>,
}


/// Parameters for creating a deck
#[derive(Debug, Serialize)]
struct CreateDeckParams {
    deck: String
}


/// Parameters for checking permissions
#[derive(Debug, Serialize)]
struct RequestPermissionParams {}


/// Parameters for getting deck names
#[derive(Debug, Serialize)]
struct GetDeckNamesParams {}


// ============================================================================================
//                                  AnkiConnect Client
// ============================================================================================


pub struct AnkiConnectClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl AnkiConnectClient {
    /// create a new AnkiConnect client
    /// default URL is http://localhost:8765
    pub fn new() -> Self {
        Self::with_url("http://localhost:8765")
    }

    pub fn with_url(url: impl Into<String>) -> Self {
        AnkiConnectClient { 
            base_url: url.into(), 
            client: reqwest::blocking::Client::new() 
        }
    }

    /// check if ankiconnect is available and request permission
    pub fn check_connection(&self) -> Result<(), Box<dyn Error>> {
        let request = AnkiRequest::new("requestPermission", RequestPermissionParams {});
        let response: AnkiResponse<serde_json::Value> = self.send_request(&request)?;

        if let Some(error) = response.error {
            return Err(format!("AnkiConnect error: {}", error).into());
        }

        Ok(())
    }


    /// get all deck names
    pub fn get_deck_names(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let request = AnkiRequest::new("deckNames", GetDeckNamesParams {});
        let response: AnkiResponse<Vec<String>> = self.send_request(&request)?;

        if let Some(error) = response.error {
            return Err(format!("Failed to get deck names: {}", error).into());
        }

        Ok(response.result.unwrap_or_default())
    }


    /// create a new deck (idempotent - won't fail if deck exists)66
    pub fn create_deck(&self, deck_name: &str) -> Result<i64, Box<dyn Error>> {
        let request = AnkiRequest::new(
            "createDeck", 
            CreateDeckParams { deck: deck_name.to_string() },
        );

        let response: AnkiResponse<i64> = self.send_request(&request)?;

        if let Some(error) = response.error {
            return Err(format!("Failed to create deck: {}", error).into());
        }

        // println!("{:?}", Ok::<&AnkiResponse<i64>, String>(&response));

        Ok(response.result.unwrap_or(0))
    }

    /// Add a single note to anki
    pub fn add_note(&self, note: Note) -> Result<i64, Box<dyn Error>> {
        let request = AnkiRequest::new(
            "addNote", 
            AddNoteParams { note },
        );

        let response: AnkiResponse<i64> = self.send_request(&request)?;

        if let Some(error) = response.error {
            // check if duplicate note error
            if error.contains("duplicate") {
                return Err("Duplicate note".into());
            }

            return Err(format!("Failed to add note: {}", error).into());
        }

        Ok(response.result.unwrap_or(0))
    }


    /// Add multiple notes in batch 
    pub fn add_notes(&self, notes: Vec<Note>) -> Result<Vec<Result<i64, String>>, Box<dyn Error>> {
        let mut results = Vec::new();

        for note in notes {
            match self.add_note(note) {
                Ok(id) => results.push(Ok(id)),
                Err(e) => results.push(Err(e.to_string())),
            }
        }

        Ok(results)
    }

    /// send a request to ankiconnect
    fn send_request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        request: &T
    ) -> Result<R, Box<dyn Error>> {
        let response = self.client
            .post(&self.base_url)
            .json(request)
            .send()?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let result = response.json::<R>()?;
        Ok(result)
    }
}

impl Default for AnkiConnectClient {
    fn default() -> Self {
        Self::new()
    }
}