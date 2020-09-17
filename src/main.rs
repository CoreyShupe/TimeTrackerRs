extern crate clap;

use std::fs::OpenOptions;
use std::io::{Read, stdin, stdout, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{App, AppSettings, Arg, SubCommand};

const SPLIT_WEEK_CHAR: char = '\n';
const SPLIT_WEEK_CHAR_BYTES: &[u8] = b"\n";
const SPLIT_DAY_CHAR: char = '?';
const SPLIT_DAY_CHAR_BYTES: &[u8] = b"?";

const CLEAR_CMD_VERSION: &str = "0.0.1";
const TRACK_CMD_VERSION: &str = "0.0.1";
const SPLIT_CMD_VERSION: &str = "0.0.1";
const SHOW_CMD_VERSION: &str = "0.0.1";
const EXPORT_CMD_VERSION: &str = "0.0.1";

fn main() {
    let home_dir = dirs::home_dir();

    let base = match home_dir {
        Some(path) => path,
        None => {
            println!("Your home directory could not be found. Exiting program.");
            return;
        }
    };

    let path = Path::new(&base).join(Path::new(".tracker_time"));

    let app_matcher = App::new("Time Tracker")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(base_sub_command("clear", "Clears tracker information.", CLEAR_CMD_VERSION).alias("c"))
        .subcommand(base_sub_command("track", "Begins the time tracker.", TRACK_CMD_VERSION))
        .subcommand(
            base_sub_command("split", "Splits the current time counter.", SPLIT_CMD_VERSION)
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(base_sub_command("day", "Splits on the day.", SPLIT_CMD_VERSION))
                .subcommand(base_sub_command("week", "Splits on the week.", SPLIT_CMD_VERSION))
        )
        .subcommand(base_sub_command("show", "Shows the time tracked.", SHOW_CMD_VERSION).alias("s"))
        .subcommand(
            base_sub_command("export", "Exports the time logs.", EXPORT_CMD_VERSION)
                .alias("e")
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .required(true)
                        .help("Sets the file to export to.")
                        .index(1)
                )
        ).get_matches();

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
            let current = get_current_ms();

            // block until typed
            stdout().flush().expect("Failed to flush stdout.");
            let mut _ignored = String::new();
            stdin().read_line(&mut _ignored).expect("Did not enter a proper string.");
            // end blocking

            let after = get_current_ms();

            append_to_file(&path, format!("{}|{},", current, after).as_bytes());

            println!("You have successfully tracked {} time.", ms_to_time(after - current));
        }
        Some("split") => {
            let matches = app_matcher.subcommand_matches("split").unwrap();
            match matches.subcommand_name() {
                Some("day") => append_to_file(&path, SPLIT_DAY_CHAR_BYTES),
                Some("week") => append_to_file(&path, SPLIT_WEEK_CHAR_BYTES),
                _ => unreachable!()
            }
        }
        Some("show") => {
            let mut result = String::new();

            if !path.exists() {
                println!("You have no time currently logged.");
                return;
            }

            read_from_file(&path, &mut result);

            println!("{}", show_time_spent(result));
        }
        Some("export") => {
            let exporting_to = Path::new(
                app_matcher.subcommand_matches("export").unwrap().value_of("file").unwrap()
            );
            if exporting_to.is_dir() {
                println!("The target file is a directory, cannot write to it.")
            } else {
                let mut result = String::new();

                if !path.exists() {
                    println!("You have no time currently logged.");
                    return;
                }

                read_from_file(&path, &mut result);
                write_to_file(&exporting_to.to_path_buf(), show_time_spent(result).as_bytes());
            }
        }
        _ => unreachable!()
    }
}

// result reading / compilation

fn show_time_spent(string: String) -> String {
    let mut display = String::new();

    let mut total: u128 = 0;
    let mut curr: u128 = 0;
    let mut week_pointer: usize = 0;

    let mut time_info: Vec<Vec<u128>> = Vec::new();
    let mut current_vec: Vec<u128> = Vec::new();

    let mut iterator = string.chars().into_iter();

    while let Some(x) = iterator.next() {
        match Some(x) {
            Some(SPLIT_DAY_CHAR) => if curr > 0 {
                current_vec.push(curr);
                curr = 0;
            },
            Some(SPLIT_WEEK_CHAR) => {
                // split the week
                if curr > 0 {
                    current_vec.push(curr);
                    curr = 0;
                }
                time_info.push(current_vec);
                current_vec = Vec::new();
                week_pointer += 1;
            }
            _ => {
                let mut prior = String::new();
                prior.push(x);
                let mut after = String::new();
                let mut flag = false;

                while let Some(z) = iterator.next() {
                    match Some(z) {
                        Some('|') => flag = true,
                        Some(',') => break,
                        _ => {
                            if flag {
                                after.push(z);
                            } else {
                                prior.push(z);
                            }
                        }
                    };
                };

                let prior_ms = prior.parse::<u128>()
                    .expect(format!("Failed to parse value: {}", prior).as_str());
                let after_ms = after.parse::<u128>()
                    .expect(format!("Failed to parse value: {}", prior).as_str());

                let time = after_ms - prior_ms;

                curr += time;
                total += time;
            }
        }
    }

    if curr > 0 {
        current_vec.push(curr);
    }

    if current_vec.len() > 0 {
        time_info.push(current_vec);
    }

    if time_info[0].len() == 0 {
        display += "You have no time currently logged.";
    } else {
        week_pointer = 0;

        display += format!("<====> Total time spent: {} <====>\n", ms_to_time(total)).as_str();
        display.push('\n');

        for x in time_info {
            display += display_week_time(week_pointer, x).as_str();
            display.push('\n');
            week_pointer += 1;
        }
        display += "<==================================>";
    };
    display
}

fn display_week_time(week: usize, vec: Vec<u128>) -> String {
    let mut display = String::new();

    display += format!("\t<===> Week {} Info <===>\n", (week + 1)).as_str();
    let mut pointer: u8 = 1;
    let mut week_total: u128 = 0;
    for i in vec {
        display += format!("\t\tDay {} ==> {}\n", pointer, ms_to_time(i)).as_str();
        pointer += 1;
        week_total += i;
    }
    display + format!("\n\t\tWeek Total ==> {}\n", ms_to_time(week_total)).as_str()
}

// sub command utilities

fn base_sub_command<'a, 'b, S: Into<&'b str>>(name: S, about: S, version: S) -> App<'a, 'b> {
    SubCommand::<'a>::with_name(name.into())
        .about(about)
        .version(version)
        .author("Corey Shupe")
}

// file utilities

fn read_from_file(path: &PathBuf, str: &mut String) {
    OpenOptions::new()
        .read(true)
        .open(path)
        .expect("Failed to open read file.")
        .read_to_string(str)
        .expect("Failed to read string from path.");
}

fn write_to_file(path: &PathBuf, bytes: &[u8]) {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .expect("Failed to open writing file.")
        .write(bytes)
        .expect("Failed to write bytes to path.");
}

fn append_to_file(path: &PathBuf, bytes: &[u8]) {
    OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .expect("Failed to open appending file.")
        .write(bytes)
        .expect("Failed to write bytes to path.");
}

// time utilities

fn get_current_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time is not working properly.")
        .as_millis()
}

fn ms_to_time(time: u128) -> String {
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