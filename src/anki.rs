use std::error::Error;
use serde::{Deserialize, Serialize};
use reqwest::{self, Response};

use super::{Word, Topic};


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
        AnkiRequest { action: action.into(), version: 6, params }
    }
}

/// Generic response structure
#[derive(Debug, Deserialize)]
struct AnkiResponse<T> {
    result: Option<T>,
    error: Option<String>,
}


/// Anki note structure
#[derive(Debug, Serialize, Clone)]
struct Note {
    #[serde(rename = "deckName")]
    deck_name: String,

    #[serde(rename = "modelName")]
    model_name: String,

    fields: String,
    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    audio: Option<Vec<AudioField>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    picture: Option<Vec<PictureField>>,
}


/// Note fields for Japanese vocabularly
#[derive(Debug, Serialize, Clone)]
struct NoteFields {
    #[serde(rename = "Front")]
    front: String,

    #[serde(rename = "Back")]
    back: String,

    #[serde(rename = "Kanji", skip_serializing_if = "String::is_empty")]
    kanji: String,
}


#[derive(Debug, Serialize, Clone)]
struct AudioField {
    url: String,
    filename: String,
    fields: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct PictureField {
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

    fn send_request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        request: &T
    ) -> Result<R, Box<dyn Error>> {
        let response = self.client
            .post(&self.base_url)
            .json(request)
            .send()?;
    }


}