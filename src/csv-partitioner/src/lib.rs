//! # csv-slice-parser
//!
//! Simple Rust library for parsings CSV files with repeating column structures.
//! Things like vocab lists seperated by topic/language (my use case, which prompted me to make this)
//! or just any CSV where columns repeat in a predictable pattern.
//!
//! ## Features
//!
//! - **Type-safe deserialisation** into custom structs
//! - **Configurable parsing** behaviour
//!
//! TODO:
//! - parallel processing support
//!
//! ## Quick Start
//!
//! ```rust
//! use csv_slice_parser::{CsvSliceParser, FromColumnSlice};
//! use csv::StringRecord;
//! use std::error::Error;
//!
//! // 1. Define your struct
//! #[derive(Debug, Clone)]
//! struct VocabEntry {
//!     word: String,
//!     translation: String,
//!     example: String,
//! }
//!
//! // 2. implement FromColumnSlice
//! impl FromColumnSlice for VocabEntry {
//!     const COLUMN_COUNT: usize = 3;
//!
//!     fn from_record(record: &StringRecord, start_col: usize) -> Result<Self, Box<dyn Error>> {
//!         Ok(VocabEntry {
//!             word: record.get(start_col)
//!                 .ok_or("Missing word")?
//!                 .to_string(),
//!             translation: record.get(start_col + 1)
//!                 .ok_or("Missing translation")?
//!                 .to_string(),
//!             example: record.get(start_col + 2)
//!                 .ok_or("Missing example")?
//!                 .to_string(),
//!         })
//!     }
//! }
//!
//! // 3. parse your CSV
//! # fn example() -> Result<(), Box<dyn Error>> {
//!     let parser = CsvSliceParser::from_file("vocabulary.csv")?;
//!     let slice1_entries = Vec<VocabEntry> = parser.parse_slice(0)?;
//!     Ok(())
//! }
//! ```
//!
//! ## CSV structure example
//!
//! for a csv like this:
//! ```csv
//! English,Spanish,Example,French,Translation,Example
//! hello,hola,Hello friend,bonjour,hello,bonjour ami
//! ```


use csv::{ReaderBuilder, StringRecord};
use std::error::Error;
use std::fmt::format;
use std::fs::{read, File};
use std::path::Path;

pub trait FromColumnSlice: Sized {
    /// The number of columns this type contains
    ///
    /// This determines how the CSV is sliced. For a struct with 3 fields,
    /// this should be 3.
    const COLUMN_COUNT: usize;


    /// Deserialise from a 'StringRecord' starting at the given column index
    ///
    /// # Arguments:
    ///
    /// * 'record' - The CSV record containing the data
    /// * 'start_col' - The starting column index for this slice
    ///
    /// # Returns
    ///
    /// * 'Ok(Self)' - successfully returns parsed struct
    /// * 'Err(Box<dyn Error>)' - parsing error with description
    ///
    /// # Example
    ///
    /// ```rust
    /// # use csv_slice_parser::FromColumnSlice;
    /// # use csv::StringRecord;
    /// # use std::error::Error;
    /// # struct MyStruct { field1: String, field2: String }
    /// # impl FromColumnSlice for MyStruct {
    /// #     const COLUMN_COUNT: usize = 2;
    /// fn from_record(record: &StringRecord, start_col: usize) -> Result<Self, Box<dyn Error>> {
    ///     // Validate column exists before parsing
    ///     let field1 = record.get(start_col)
    ///         .ok_or("Missing first column")?
    ///         .to_string();
    ///
    ///     // Parse numeric types with error handling
    ///     let field2 = record.get(start_col + 1)
    ///         .ok_or("Missing second column")?
    ///         .to_string();
    ///
    ///     Ok(MyStruct { field1, field2 })
    /// }
    /// # }
    /// ```
    fn from_record(record: &StringRecord, start_col: usize) -> Result<Self, Box<dyn Error>>;
}


/// Configuration for CSV parsing behaviour
///
/// Use this to customize how the parser handles edge cases and performance trade-offs.
///
/// # Example
///
/// ```rust
/// use csv_partitioner::ParseConfig;
/// use csv_slice_parser::ParseConfig;
///
/// let config = ParseConfig {
///     skip_empty_rows = true,
///     reserve_capacity = true,
///     trim_fields = true,
/// }
/// ```
pub struct ParseConfig {
    /// Skip rows where all columns in the slice are empty.
    ///
    /// When `true`, rows like `"", "", ""` are filtered out.
    /// Default: `true`
    pub skip_empty_rows: bool,

    /// Pre-allocate capacity in result vectors based on record count.
    ///
    /// When `true`. reduces reallocations but uses more memory upfront.
    /// Default: `true`
    pub reserve_capacity: bool,

    /// Trim whitespaces from all fields during CSV reading
    ///
    /// When `true`, `" hello "` becomes `"hello"`.
    /// Default: `true`
    pub trim_fields: bool,
}

impl Default for ParseConfig {
    fn default() -> Self {
        ParseConfig {
            skip_empty_rows: true,
            reserve_capacity: true,
            trim_fields: true,
        }
    }
}

/// Main parser
pub struct CsvSliceParser {
    headers: StringRecord,
    records: Vec<StringRecord>,
    config: ParseConfig,
}

impl CsvSliceParser {
    /// Loads a CSV file with a default configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the CSV file
    ///
    /// # Returns
    ///
    /// * `Ok(CsvSliceParser)` - Successfully loaded parser
    /// * `Err(Box<dyn Error>)` - I/O or parsing error
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        Self::from_file_with_config(path, ParseConfig::default())
    }

    /// Load a CSV file with custom configuration
    pub fn from_file_with_config<P: AsRef<Path>>(
        path: P,
        config: ParseConfig
    ) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(file);

        let headers = reader.headers()?.clone();

        let mut records: Vec<StringRecord> = if config.reserve_capacity {
            Vec::with_capacity(headers.len())
        } else {
            Vec::new()
        };

        for result in reader.records() {
            records.push(result?);
        }

        if config.reserve_capacity {
            records.shrink_to_fit();
        }

        Ok(CsvSliceParser { headers, records, config })
    }


    /// Create a parser from in-memory `StringRecord` data.
    pub fn from_records(
        headers: StringRecord,
        records: Vec<StringRecord>,
        config: ParseConfig,
    ) -> Self {
        CsvSliceParser { headers, records, config }
    }

    /// get the number of column slices available for a given type
    #[inline]
    pub fn slice_count<T: FromColumnSlice>(&self) -> usize {
        self.headers.len() / T::COLUMN_COUNT
    }

    /// get the total number of records (rows) in the CSV.
    #[inline]
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    fn validate_slice_index<T: FromColumnSlice>(&self, slice_index: usize) -> Result<(usize, usize), Box<dyn Error>>{
        let start_col = slice_index * T::COLUMN_COUNT;
        let end_col = start_col + T::COLUMN_COUNT;

        if end_col > self.headers.len() {
            return Err(format!(
                "Slice {} out of bounds (columns {}-{} requested, but only {} columns available)",
                slice_index, start_col, end_col, self.headers.len()
            ).into());
        }

        Ok((start_col, end_col))
    }

    fn has_empty_fields(&self, start_col: usize, end_col: usize, record: &StringRecord) -> bool {
        (start_col..end_col)
            .all(|i| record.get(i).map_or(true, |s| s.trim().is_empty()))
    }

    /// Parse a specific column slice into a vector of structs.
    ///
    /// This is the main parsing method. It deserialises all rows for a given
    /// column slice into your custom struct type
    pub fn parse_slice<T: FromColumnSlice>(&self, slice_index: usize) -> Result<Vec<T>, Box<dyn Error>> {
        let (start_col, end_col) = self.validate_slice_index::<T>(slice_index)?;

        let mut results = if self.config.reserve_capacity {
            Vec::with_capacity(self.records.len())
        } else {
            Vec::new()
        };

        for record in &self.records {
            if self.config.skip_empty_rows {
                if self.has_empty_fields(start_col, end_col, record) {
                    continue
                }
            }
            results.push(T::from_record(record, start_col)?);
        }

        results.shrink_to_fit();

        Ok(results)
    }

    /// Parse a slice lazily with an iterator
    pub fn parse_slice_iter<'a, T: FromColumnSlice + 'a>(
        &'a self,
        slice_index: usize
    ) -> impl Iterator<Item = Result<T, Box<dyn Error>>> {
        let (start_col, end_col) = self.validate_slice_index::<T>(slice_index)?;

        Ok(self.records.iter().filter_map(move |record| {
            if self.config.skip_empty_rows {
                if self.has_empty_fields(start_col, end_col, record) {
                    return None;
                }
            }
            Some(T::from_record(record, start_col))
        }))
    }

    /// Parse all slices into separate vectors
    pub fn parse_all_slices<T: FromColumnSlice>(&self) -> Result<Vec<Vec<T>>, Box<dyn Error>> {
        let slice_count = self.slice_count::<T>();
        let mut all_slices: Vec<Vec<T>> = Vec::with_capacity(slice_count);

        for i in 0..slice_count {
            all_slices.push(self.parse_slice(i)?)
        }

        Ok(all_slices)
    }

    pub fn slice_headers<T: FromColumnSlice>(&self, slice_index: usize) -> Option<Vec<&str>> {
        let start_col = slice_index * T::COLUMN_COUNT;
        let end_col = slice_index + T::COLUMN_COUNT;

        if end_col > self.headers.len() {
            None
        } else {
            Some((start_col..end_col)
                .filter_map(|i| self.headers.get(i))
                .collect())
        }
    }

    #[inline]
    pub fn records(&self) -> &[StringRecord] {
        &self.records
    }

    #[inline]
    pub fn headers(&self) -> &StringRecord {
        &self.headers
    }
}