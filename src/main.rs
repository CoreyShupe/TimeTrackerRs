extern crate clap;

use std::fs::OpenOptions;
use std::io::{Read, stdin, stdout, Write};
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{App, AppSettings, Arg, SubCommand};

const SPLIT_WEEK_CHAR: char = '\n';
const SPLIT_DAY_CHAR: char = '?';

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
        .subcommand(
            SubCommand::with_name("clear")
                .about("Clears tracker information.")
                .author("Corey Shupe")
                .alias("c")
                .version("0.0.1")
        )
        .subcommand(
            SubCommand::with_name("track")
                .about("Begins the time tracker.")
                .author("Corey Shupe")
                .version("0.0.1")
        )
        .subcommand(
            SubCommand::with_name("split")
                .about("Splits the current time counter.")
                .author("Corey Shupe")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("day")
                        .about("Splits on the day.")
                        .author("Corey Shupe")
                        .version("0.0.1")
                )
                .subcommand(
                    SubCommand::with_name("week")
                        .about("Splits on the week.")
                        .author("Corey Shupe")
                        .version("0.0.1")
                )
                .version("0.0.1")
        )
        .subcommand(
            SubCommand::with_name("show")
                .about("Shows the time tracked.")
                .author("Corey Shupe")
                .alias("s")
                .version("0.0.1")
        )
        .subcommand(
            SubCommand::with_name("export")
                .about("Exports the time logs.")
                .author("Corey Shupe")
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .required(true)
                        .help("Sets the file to export to.")
                        .index(1)
                )
                .alias("e")
                .version("0.0.1")
        )
        .get_matches();

    if let Some(_) = app_matcher.subcommand_matches("clear") {
        if path.exists() {
            std::fs::remove_file(path).expect("Failed to remove file.");
        }
        println!("Your tracking progress has been cleared.")
    } else if let Some(_) = app_matcher.subcommand_matches("track") {
        print!("Your tracker has started, type anything to stop the tracker: ");
        let current = get_current_ms();

        // block until typed
        stdout().flush().expect("Failed to flush stdout.");
        let mut _ignored = String::new();
        stdin().read_line(&mut _ignored).expect("Did not enter a proper string.");
        // end blocking

        let after = get_current_ms();

        append(path, format!("{}|{},", current, after).as_bytes());

        println!("You have successfully tracked {} time.", ms_to_time(after - current));
    } else if let Some(cmd_matcher) = app_matcher.subcommand_matches("split") {
        if let Some(_) = cmd_matcher.subcommand_matches("day") {
            append(path, SPLIT_DAY_CHAR.to_string().as_bytes());
        } else if let Some(_) = cmd_matcher.subcommand_matches("week") {
            append(path, SPLIT_WEEK_CHAR.to_string().as_bytes());
        }
    } else if let Some(_) = app_matcher.subcommand_matches("show") {
        let mut result = String::new();

        if !path.exists() {
            println!("You have no time currently logged.");
            return;
        }

        OpenOptions::new()
            .read(true)
            .open(path)
            .expect("Failed to open read file.")
            .read_to_string(&mut result)
            .expect("Failed to read string from path.");

        println!("{}", show_time_spent(result));
    } else if let Some(matcher) = app_matcher.subcommand_matches("export") {
        let exporting_to = Path::new(matcher.value_of("file").unwrap());
        if exporting_to.is_dir() {
            println!("The target file is a directory, cannot write to it.")
        } else {
            let mut result = String::new();

            if !path.exists() {
                println!("You have no time currently logged.");
                return;
            }

            OpenOptions::new()
                .read(true)
                .open(path)
                .expect("Failed to open read file.")
                .read_to_string(&mut result)
                .expect("Failed to read string from path.");

            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(exporting_to)
                .expect("Failed to open output file.")
                .write_all(result.as_bytes())
                .expect("Failed to export to output file.");
        }
    } else {
        unreachable!();
    }
}

fn show_time_spent(string: String) -> String {
    let mut display = String::new();

    let mut total: u128 = 0;
    let mut curr: u128 = 0;
    let mut week_pointer: usize = 0;

    let mut time_info: Vec<Vec<u128>> = Vec::new();

    time_info.push(Vec::new());

    let mut iterator = string.chars().into_iter();

    while let Some(x) = iterator.next() {
        match Some(x) {
            Some(SPLIT_DAY_CHAR) => {
                // split the day
                time_info.get_mut(week_pointer).expect("Failed to unwrap vec.").push(curr);
                curr = 0;
            }
            Some(SPLIT_WEEK_CHAR) => {
                // split the week
                if curr > 0 {
                    time_info.get_mut(week_pointer).expect("Failed to unwrap vec.").push(curr);
                    curr = 0;
                }
                time_info.push(Vec::new());
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
        time_info.get_mut(week_pointer).expect("Failed to unwrap vec.").push(curr);
    }

    if time_info[0].len() == 0 {
        display = display.add("You have no time currently logged.");
    } else {
        week_pointer = 0;

        display = display.add(format!("<====> Total time spent: {} <====>\n", ms_to_time(total)).as_str());
        display.push('\n');

        for x in time_info {
            display = display.add(display_week_time(week_pointer, x).as_str());
            display.push('\n');
            week_pointer += 1;
        }
        display = display.add("<==================================>\n");
    };
    display
}

fn display_week_time(week: usize, vec: Vec<u128>) -> String {
    let mut display = String::new();

    display = display.add(format!("\t<===> Week {} Info <===>\n", (week + 1)).as_str());
    let mut pointer: u8 = 1u8;
    let mut week_total: u128 = 0u128;
    for i in vec {
        display = display.add(format!("\t\tDay {} ==> {}\n", pointer, ms_to_time(i)).as_str());
        pointer += 1;
        week_total += i;
    }
    display.push('\n');
    display = display.add(format!("\t\tWeek Total ==> {}\n", ms_to_time(week_total)).as_str());
    display
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
        display = display.add(format!("{} Hour", hours).as_str());
        if hours > 1 {
            display = display.add("s");
        }
        flag = true;
    }

    if minutes > 0 {
        if flag {
            display = display.add(" ");
        }
        display = display.add(format!("{} Minute", minutes).as_str());
        if minutes > 1 {
            display = display.add("s");
        }
        flag = true;
    }

    if seconds > 0 {
        if flag {
            display = display.add(" ");
        }
        display = display.add(format!("{} Second", seconds).as_str());
        if seconds > 1 {
            display = display.add("s");
        }
    }

    display
}

fn append(path: PathBuf, bytes: &[u8]) {
    OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .expect("Failed to open appending file.")
        .write(bytes)
        .expect("Failed to write bytes to path.");
}

fn get_current_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time is not working properly.")
        .as_millis()
}