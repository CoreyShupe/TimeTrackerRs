extern crate prettytable;
extern crate clap;

mod tracker;
mod transformers;

use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{App, AppSettings, Arg, SubCommand};
use crate::tracker::{Tracker, TrackEntry};

const CLEAR_CMD_VERSION: &str = "0.0.1";
const TRACK_CMD_VERSION: &str = "0.0.2";
const SHOW_CMD_VERSION: &str = "0.0.2";

fn main() {
    let app_matcher = App::new("Time Tracker")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("path")
            .index(1)
            .short("p")
            .required(true)
            .help("The file to contain and pull the tracker info for."))
        .subcommand(base_sub_command("clear", "Clears tracker information.", CLEAR_CMD_VERSION).alias("c"))
        .subcommand(base_sub_command("track", "Begins the time tracker.", TRACK_CMD_VERSION)
            .arg(Arg::with_name("description")
                .index(1)
                .short("d")
                .required(true)
                .help("Description of what's being tracked."))
            .arg(Arg::with_name("short")
                .index(2)
                .short("s")
                .required(false)
                .help("Short of what's being tracked."))
        )
        .subcommand(base_sub_command("show", "Shows the time tracked.", SHOW_CMD_VERSION).alias("s"))
        .get_matches();

    let path = Path::new(app_matcher.value_of("path").unwrap()).to_path_buf();

    // executions

    match app_matcher.subcommand_name() {
        Some("clear") => {
            if path.exists() {
                std::fs::remove_file(path).expect("Failed to remove file.");
            }
            println!("Your tracking progress has been cleared.")
        }
        Some("track") => {
            print!("Your tracker has started, type anything to stop the tracker: ");

            let sub = app_matcher.subcommand_matches("track").unwrap();

            let description = sub.value_of("description").unwrap();
            let short = sub.value_of("short");

            let current = get_current_ms();

            // block until typed
            stdout().flush().expect("Failed to flush stdout.");
            stdin().read_line(&mut String::new()).expect("Did not enter a proper string.");
            // end blocking

            let after = get_current_ms();

            let mut tracker = if *&path.exists() {
                Tracker::read_from(&path).expect("Failed to read CSV file.")
            } else {
                Tracker::new()
            };
            tracker = tracker.insert_entry(TrackEntry::new(
                get_current_ms(),
                after - current,
                String::from(description),
                if let Some(short_item) = short {
                    Option::Some(String::from(short_item))
                } else {
                    Option::None
                },
            ));
            tracker.write_to(&path).expect("Failed to write CSV file.");

            println!("You have successfully tracked {} time for {}.", tracker::ms_to_time(after - current), description);
        }
        Some("show") => {
            if !path.exists() {
                println!("You have no time currently logged.");
                return;
            }

            let tracker = Tracker::read_from(&path).expect("Failed to read CSV file.");
            tracker.gen_table().printstd();
        }
        _ => unreachable!()
    }
}

// sub command utilities

fn base_sub_command<'a, 'b, S: Into<&'b str>>(name: S, about: S, version: S) -> App<'a, 'b> {
    SubCommand::<'a>::with_name(name.into())
        .about(about)
        .version(version)
        .author("Corey Shupe")
}

// time utilities

fn get_current_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time is not working properly.")
        .as_millis()
}