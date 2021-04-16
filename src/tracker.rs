use prettytable::{Table, Row, Cell, Attr};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use prettytable::color::{RED, BLUE};

#[derive(Serialize, Deserialize, PartialEq)]
pub struct TrackEntry {
    #[serde(rename = "Entry Date MS")]
    entry_date: u128,
    #[serde(rename = "Time Spent MS")]
    time_spent_ms: u128,
    #[serde(rename = "Description")]
    description: String,
}

impl TrackEntry {
    pub fn new(entry_date: u128, time_spent_ms: u128, description: String) -> Self {
        Self { entry_date, time_spent_ms, description }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Tracker {
    entries: Vec<TrackEntry>
}

impl Tracker {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn with_vector(vector: Vec<TrackEntry>) -> Self {
        Self { entries: vector }
    }

    pub fn gen_table(&self) -> Table {
        let mut table = Table::new();

        table.add_row(Row::new(
            vec![
                Cell::new("Description").with_style(Attr::Bold),
                Cell::new("Time Spent").with_style(Attr::Bold)
            ]
        ));

        table.add_empty_row();

        let mut global_time = 0u128;
        let mut map = HashMap::<String, u128>::new();

        for entry in &self.entries {
            global_time += entry.time_spent_ms;
            let old_value = *map.get(&entry.description).unwrap_or_else(|| { &0u128 });
            &map.insert(
                entry.description.clone(),
                old_value + entry.time_spent_ms,
            );
        }

        for map_entry in map {
            let description = &*map_entry.0;
            let time_spent = &*ms_to_time(map_entry.1);

            table.add_row(Row::new(vec![
                Cell::new(description),
                Cell::new(time_spent)
            ]));
        }

        table.add_empty_row();
        table.add_row(Row::new(
            vec![
                Cell::new("Total").with_style(Attr::Bold),
                Cell::new(&*ms_to_time(global_time)).with_style(Attr::Italic(true))
            ]
        ));

        table
    }

    pub fn write_to(&self, path: &PathBuf) -> csv::Result<()> {
        let writer_result = csv::Writer::from_path(path);
        match writer_result {
            Ok(mut writer) => {
                for entry in &self.entries {
                    if let Err(e) = writer.serialize(entry) {
                        return Err(e);
                    }
                }
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn read_from(path: &PathBuf) -> csv::Result<Self> {
        let reader_result = csv::Reader::from_path(path);
        match reader_result {
            Ok(mut reader) => {
                let mut track_entries = Vec::<TrackEntry>::new();
                for result in reader.deserialize() {
                    let record: TrackEntry = result?;
                    track_entries.push(record);
                }
                Ok(Self::with_vector(track_entries))
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn insert_entry(mut self, entry: TrackEntry) -> Self {
        self.entries.push(entry);
        self
    }
}

pub fn ms_to_time(time: u128) -> String {
    let mut seconds = time / 1000;
    let mut minutes = seconds / 60;
    seconds %= 60;
    let hours = minutes / 60;
    minutes %= 60;

    let mut display = String::new();
    let mut flag = false;

    if hours > 0 {
        display += format!("{} Hour", hours).as_str();
        if hours > 1 {
            display += "s";
        }
        flag = true;
    }

    if minutes > 0 {
        if flag {
            display += " ";
        }
        display += format!("{} Minute", minutes).as_str();
        if minutes > 1 {
            display += "s";
        }
        flag = true;
    }

    if seconds > 0 {
        if flag {
            display += " ";
        }
        display += format!("{} Second", seconds).as_str();
        if seconds > 1 {
            display += "s";
        }
    }

    display
}
