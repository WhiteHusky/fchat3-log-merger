use clap::App;
use clap::load_yaml;
use fchat3_log_lib::{FChatMessageReader, FChatWriter, fchat_message::FChatMessage};
use humantime::{parse_duration, format_duration};
use log::{info, trace, error};
use pretty_env_logger;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::ffi::OsString;
use std::fs::{File, OpenOptions, create_dir, read_dir};
use std::io::BufReader;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use chrono::Duration;
use rayon::prelude::*;

#[derive(Debug)]
enum Error {
    OutputExists(PathBuf),
    NotEnoughInputs,
    InputDoesNotExist(PathBuf),
    InputIsNotDirectory(PathBuf),
    BadTimeDiff(humantime::DurationError),
    UnableToCreateDirectory(std::io::Error),
    MessageParseError(fchat3_log_lib::error::Error),
    ExitingWithError
}

impl From<fchat3_log_lib::error::Error> for Error {
    fn from(e: fchat3_log_lib::error::Error) -> Self {
        Self::MessageParseError(e)
    }
}
struct SortedMessage {
    message: FChatMessage
}

impl Ord for SortedMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.message.datetime.cmp(&other.message.datetime)
    }
}

impl PartialOrd for SortedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SortedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.message.datetime == other.message.datetime
    }
}

impl Eq for SortedMessage {
    fn assert_receiver_is_total_eq(&self) { unimplemented!() }
}

type CharacterName = OsString;
type LogName = OsString;
type Logs = HashMap<LogName, Vec<PathBuf>>;
type Characters = HashMap<CharacterName, Logs>;

fn main() -> Result<(), Error> {
    pretty_env_logger::init();
    let yml = load_yaml!("app.yaml");
    let matches = App::from_yaml(yml).get_matches();
    let output_path = Path::new(matches.value_of("output").unwrap());

    if output_path.exists() {
        error!("Output already exists.");
        return Err(Error::OutputExists(output_path.to_owned()))
    }

    if let Err(e) = create_dir(output_path) {
        error!("Unable to create output directory: {}", output_path.to_string_lossy());
        return Err(Error::UnableToCreateDirectory(e))
    }

    let mut characters: Characters = HashMap::new();

    {
        let folder_strings: Vec<&str> = matches.values_of("folders").unwrap().collect();
        if folder_strings.len() < 2 {
            error!("Specifiy more than one folder.");
            return Err(Error::NotEnoughInputs)
        }
        for folder_string in folder_strings {
            let folder_path = PathBuf::from(folder_string);

            if !folder_path.exists() {
                error!("Path does not exist: {:?}", folder_path);
                return Err(Error::InputDoesNotExist(folder_path))
            } else if !folder_path.is_dir() {
                error!("Path is not directory: {:?}", folder_path);
                return Err(Error::InputIsNotDirectory(folder_path))
            }
            for c in read_dir(folder_path).unwrap() {
                let log_folder_entry = c.unwrap();
                if log_folder_entry.metadata().unwrap().is_dir() {
                    let mut character_folder_path = log_folder_entry.path();
                    let character_name = character_folder_path.file_name().unwrap().to_owned();
                    trace!("Getting logs for {:?}", character_name);
                    character_folder_path.push("logs");
                    if character_folder_path.exists() {
                        let mut logs: Vec<(OsString, PathBuf)> = Vec::new();
                        for e in read_dir(character_folder_path).unwrap() {
                            let log_file_entry = e.unwrap();
                            // Log files do not have a extension.
                            if log_file_entry.path().extension() == None {
                                let log_name = log_file_entry.file_name();
                                trace!("-- {:?}", log_name);
                                logs.push((log_name, log_file_entry.path()));
                            }
                        }
                        if logs.len() > 0 {
                            let character = characters.entry(character_name).or_insert(Logs::new());
                            for (log_name, entry) in logs {
                                character.entry(log_name).or_insert(Vec::new()).push(entry);
                            }
                        }
                    }
                }
            }
        }
    }

    {
        let mut sum = 0;
        for (_, logs) in &characters {
            for (_, files) in logs {
                sum += files.len();
            }
        }
        info!("{} files to merge.", sum)
    }

    let time_diff = match matches.value_of("time-diff") {
        Some(s) => match parse_duration(s) {
            Ok(d) => Duration::from_std(d).unwrap(),
            Err(e) => {
                error!("Bad time diff specified: {}.", e);
                return Err(Error::BadTimeDiff(e))
            },
        },
        None => Duration::seconds(60*5) // 5 minutes
    };
    
    info!("Merging messages with at most a difference in the future of {}.", format_duration(time_diff.to_std().unwrap()));

    let results: Vec<Result<Vec<Result<(), Error>>, Error>> = characters.par_iter().map(|(character_name, log_entries)| {
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        info!("Merging {}", character_name.to_string_lossy());
        let mut output_log_location = output_path.to_path_buf();
        output_log_location.push(character_name.clone());
        if let Err(e) = create_dir(output_log_location.clone()) {
            error!("Unable to create directory for {}", character_name.to_string_lossy());
            return Err(Error::UnableToCreateDirectory(e))
        }
        let results: Vec<Result<(), Error>> = log_entries.par_iter().map(|(log_name, locations)| {
            //info!("Merging tab {}", log_name.to_string_lossy());
            let mut log_path = output_log_location.clone();
            log_path.push(log_name);
            let mut idx_path = log_path.clone();
            idx_path.set_extension("idx");
            let mut w = FChatWriter::new(
                options.open(log_path).unwrap(),
                options.open(idx_path).unwrap(),
                character_name.to_str().unwrap().to_owned()).unwrap();
            // For single locations, just write them out without comparing.
            if locations.len() == 1 {
                let f = File::open(&locations[0]).unwrap();
                for r in FChatMessageReader::new(BufReader::new(f)) {
                    let message = r?;
                    w.write_message(message).unwrap();
                }
            // Otherwise, start queuing messages, matching, sorting by send-time, before rotating them into the output
            } else {
                let mut readers: Vec<Peekable<FChatMessageReader>> = locations.iter()
                .map(|p| {
                    FChatMessageReader::new(BufReader::new(File::open(p).unwrap())).peekable()
                }).collect();

                let mut messages = BinaryHeap::new();
                loop {
                    if messages.peek().is_none() {
                        let mut index = 0;
                        while index < readers.len() {
                            if let Some(r) = readers[0].next() {
                                let message = r?;
                                messages.push(Reverse(SortedMessage {message}));
                                index+=1;
                            } else {
                                let _ = readers.remove(index);
                            }
                        }
                        if messages.is_empty() { break };
                    } else {
                        let mut message_batch = Vec::new();
                        {
                            let mut index = 0;
                            let oldest_message = &messages.peek().unwrap().0.message;
                            while index < readers.len() {
                                let reader = &mut readers[index];
                                if let Some(peeked_result) = reader.peek() {
                                    if let Ok(peeked_message) = peeked_result.as_ref() {
                                        if peeked_message.datetime < oldest_message.datetime + time_diff {
                                            message_batch.push(reader.next().unwrap()?);
                                        } else {
                                            index += 1;
                                        }
                                    } else {
                                        // We know an error is coming up, so result it to nothing and catch it.
                                        let _ = reader.next().unwrap()?;
                                        panic!("We were expecting an error to unwrap, but it did not.");
                                    }               
                                } else {
                                    let _ = readers.remove(index);
                                }
                            }
                        }
                        for check_message in message_batch {
                            let mut duplicate = false;
                            for Reverse(SortedMessage {message}) in &messages {
                                if check_message.datetime < message.datetime + time_diff {
                                    if message_compare(&check_message, &message) {
                                        //trace!("Duplicate Hit: {:?} == {:?}", check_message, message);
                                        duplicate = true;
                                        break
                                    }
                                } else {
                                    break
                                }
                            }
                            if !duplicate {
                                messages.push(Reverse(SortedMessage{message:check_message}));
                            }
                        }
                        w.write_message(messages.pop().unwrap().0.message)?;
                    }
                }
            }
            Ok(())
        }).collect();
        Ok(results)
    }).collect();
    let mut character_index = 0;
    let mut error_count = 0;
    for (character, log_entries) in characters {
        if let Err(e) = &results[character_index] {
            error_count += 1;
            error!("{} had an error: {:?}", character.to_string_lossy(), e);
        }
        let mut log_entry_index = 0;
        for (log_name, _) in log_entries {
            if let Err(e) = &results[character_index].as_ref().unwrap()[log_entry_index] {
                error_count += 1;
                error!("{} for {} had an error: {:?}", log_name.to_string_lossy(), character.to_string_lossy(), e);
            }
            log_entry_index += 1;
        }
        character_index += 1;
    }
    return if error_count > 0 {
        error!("{} errors were hit", error_count);
        Err(Error::ExitingWithError)
    } else {
        info!("Success.");
        Ok(())
    }
}

fn message_body(message: &FChatMessage) -> &String {
    {
        use fchat3_log_lib::fchat_message::FChatMessageType::*;
        match &message.body {
            Message(string) | Action(string) | Ad(string) | Roll(string) | Warn(string)
            | Event(string) => string,
        }
    }
}

fn message_body_type(message: &FChatMessage) -> u8 {
    {
        use fchat3_log_lib::fchat_message::FChatMessageType::*;
        match message.body {
            Message(_) => 0,
            Action(_) => 1,
            Ad(_) => 2,
            Roll(_) => 3,
            Warn(_) => 4,
            Event(_) => 5,
        }
    }
}

fn message_compare(a: &FChatMessage, b: &FChatMessage) -> bool {
    (message_body_type(a) == message_body_type(b)) && (message_body(a) == message_body(b))
}