use clap::{App, crate_authors, crate_name, crate_version, load_yaml};
use fchat3_log_lib::{fchat_index::FChatIndex};
use fchat3_log_lib::{FChatWriter, fchat_message::FChatMessage};
use humantime::{parse_duration, format_duration};
use log::{error, trace, warn};
use pretty_env_logger;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::ffi::OsString;
use std::fs::{File, OpenOptions, create_dir, create_dir_all, read_dir};
use std::io::{BufWriter};
use std::path::{Path, PathBuf};
use std::process;
use chrono::Duration;
use rayon::prelude::*;
use std::sync::Mutex;
use linya::Progress;
use humansize::{FileSize, file_size_opts as size_opts};

mod error;
pub(crate) use error::Error;

mod sorted_message;
pub(crate) use sorted_message::SortedMessage;

mod reader;
pub(crate) use reader::Reader;

type CharacterName = String;
type LogName = String;
type Logs = HashMap<LogName, Vec<PathBuf>>;
type Characters = HashMap<CharacterName, Logs>;

fn main() {
    match _main() {
        Err(e) => {
            error!("{}", e);
            process::exit(-1);
        }
        _ => {}
    }
}

fn _main() -> Result<(), Error> {
    pretty_env_logger::init();
    let yml = load_yaml!("app.yaml");
    let app = App::from_yaml(yml)
        .name(crate_name!())
        .version(crate_version!())
        .author(crate_authors!());
    let matches = app
        .get_matches();
    let dry_run = matches.is_present("dry-run");
    let dupe_warning = matches.is_present("dupe-warning");
    let folder_paths  = matches.values_of("folders")
        .unwrap()
        .map(|s| {
            let p = PathBuf::from(s);
            // Fail early
            if !p.exists() {
                return Err(Error::InputDoesNotExist(p))
            } else if !p.is_dir() {
                return Err(Error::InputIsNotDirectory(p))
            }
            Ok(p)
        })
        .collect::<Result<Vec<_>,_>>()?;
    
    if folder_paths.len() < 2 {
        return Err(Error::NotEnoughInputs)
    }

    let (characters, size_total, file_total) = collect_logs(folder_paths)?;

    println!("{} files to merge, {}.", file_total, size_total.file_size(size_opts::CONVENTIONAL).unwrap());

    let time_diff = match matches.value_of("time-diff") {
        Some(s) => Duration::from_std(parse_duration(s)?).unwrap(),
        None => Duration::minutes(5)
    };
    
    println!("Merging messages with at most a difference in the future of {}.", format_duration(time_diff.to_std().unwrap()));

    if dry_run {
        println!("Dry run enabled. Printing out what would be collected...");
        for (character, log_entries) in characters {
            println!("=== {} ===", character);
            for (log_name, paths) in log_entries {
                println!("== {} ==", log_name);
                for path in paths {
                    println!("{}", path.to_string_lossy());
                }
            }
        }
        println!("More information could be available. Set the RUST_LOG environment variable.");
        return Ok(());
    }

    let output_path = Path::new(matches.value_of("output").unwrap());

    if output_path.exists() {
        return Err(Error::OutputExists(output_path.to_owned()))
    }

    create_dir(output_path).map_err(|e| Error::UnableToCreateDirectory(output_path.into(), e))?;

    let results: MergeResults = merge_logs(&characters, output_path, time_diff, dupe_warning);
    let mut character_index = 0;
    let mut error_count = 0;
    for (character, log_entries) in characters {
        if let Err(e) = &results[character_index] {
            error_count += 1;
            error!("{} had an error: {}", character, e);
        }
        let mut log_entry_index = 0;
        for (log_name, _) in log_entries {
            if let Err(e) = &results[character_index].as_ref().unwrap()[log_entry_index] {
                error_count += 1;
                error!("{} for {} had an error: {}", log_name, character, e);
            }
            log_entry_index += 1;
        }
        character_index += 1;
    }
    return if error_count > 0 {
        error!("{} errors were hit", error_count);
        Err(Error::ExitingWithError)
    } else {
        Ok(())
    }
}

fn collect_logs(folder_paths: Vec<PathBuf>) -> Result<(Characters, u64, u64), Error> {
    let mut characters = Characters::new();
    let mut size_total: u64 = 0;
    let mut file_total: u64 = 0;
    for folder_path in folder_paths {
        if !folder_path.exists() {
            return Err(Error::InputDoesNotExist(folder_path))
        } else if !folder_path.is_dir() {
            return Err(Error::InputIsNotDirectory(folder_path))
        }

        let log_folders = read_dir(&folder_path)
            .map_err(|e| Error::UnableToOpenDirectory(folder_path, e))?
            .map(|e| e.unwrap())
            .filter(|e| e.metadata().unwrap().is_dir());
    
        for log_folder_entry in log_folders {
            let mut character_folder_path = log_folder_entry.path();
            let character_name = character_folder_path.file_name().unwrap().to_owned();

            trace!("Getting logs for {:?}", character_name);
            character_folder_path.push("logs");

            if !character_folder_path.exists() { continue; }

            let mut logs: Vec<(OsString, PathBuf)> = Vec::new();
            let log_files = read_dir(&character_folder_path)
                .map_err(|e| Error::UnableToOpenDirectory(character_folder_path, e))?
                .map(|e| e.unwrap())
                // Log files do not have a extension.
                .filter(|e| e.path().extension() == None )
                // Check if an idx is present. Required to get correct tab name.
                .filter(|e| {
                    let mut p = e.path();
                    p.set_extension("idx");
                    if !p.exists() {
                        warn!("{:?} is missing its idx file and has been skipped", p);
                        false
                    } else {
                        true
                    }
                });
        
            for log_file_entry in log_files {
                let log_name = log_file_entry.file_name();
                size_total += log_file_entry.metadata()
                    .map_err(|e| Error::UnableToOpenLog(log_file_entry.path(), e))?.len();
                file_total += 1;
                trace!("-- {:?}", log_name);
                logs.push((log_name, log_file_entry.path()));
            }
        
            if logs.len() > 0 {
                let character = characters.entry(character_name.to_string_lossy().into()).or_insert(Logs::new());
                for (log_name, entry) in logs {
                    character.entry(log_name.to_string_lossy().into()).or_insert(Vec::new()).push(entry);
                }
            }
        }
    }
    Ok((characters, size_total, file_total))
}

type PerLogMergeResults = Vec<Result<(), Error>>;
type MergeResults = Vec<Result<PerLogMergeResults, Error>>;

fn merge_logs(characters: &Characters, output_path: &Path, time_diff: Duration, dupe_warning: bool) -> MergeResults {
    let progress = Mutex::new(Progress::new());
    characters.par_iter().map(|(character_name, log_entries)| {
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);

        let mut output_log_location = output_path.to_path_buf();
        output_log_location.push(character_name.clone());
        output_log_location.push("logs");

        create_dir_all(&output_log_location)
            .map_err(|e| Error::UnableToCreateDirectory(output_log_location.clone(), e))?;

        let bar = Mutex::new(
            progress
                .lock()
                .unwrap()
                .bar(
                    log_entries.len(),
                    format!("{}", character_name)
                )
        );

        Ok(log_entries.par_iter().map(|(log_name, locations)| {
            //info!("Merging tab {}", log_name.to_string_lossy());
            let tab_name = {
                let mut source_idx = locations[0].clone();
                source_idx.set_extension("idx");

                let mut f = File::open(&source_idx).map_err(|e| Error::UnableToOpenIndex(source_idx, e))?;
                FChatIndex::read_header_from_buf(&mut f)?.name
            };

            let mut log_path = output_log_location.clone();
            log_path.push(log_name);

            let mut idx_path = log_path.clone();
            idx_path.set_extension("idx");

            let mut idx_buf = BufWriter::new(options.open(&idx_path)
                .map_err(|e| Error::UnableToOpenIndex(idx_path, e))?);
            let mut log_buf = BufWriter::new(options.open(&log_path)
                .map_err(|e| Error::UnableToOpenLog(log_path, e))?);

            let mut w = FChatWriter::new(
                &mut idx_buf,
                tab_name.clone()
            )?;

            // For single locations, just write them out without comparing.
            if locations.len() == 1 {
                let f = File::open(&locations[0])
                    .map_err(|e| Error::UnableToOpenLog(locations[0].clone(), e))?;
                for r in reader::Reader::new_buffered(f) {
                    let message = r?;
                    w.write_message(&mut log_buf, &mut idx_buf, message)?;
                }
            // Otherwise get the oldest message and compare it against up-to
            // time-diff from each reader before determining to commit it.
            // All messages within time-diff are added to the queue otherwise
            // more are pulled from the readers.
            } else {
                deduplicate_messages(locations, tab_name, time_diff, w, log_buf, idx_buf, &dupe_warning)?;
            }
            progress.lock().unwrap().inc_and_draw(&bar.lock().unwrap(), 1);
            Ok(())
        }).collect())
    }).collect()
}

fn format_message(message: &FChatMessage) -> String {
    use fchat3_log_lib::fchat_message::FChatMessageType::*;
    format!("[{}] {}", message.datetime,
    match &message.body {
        Message(m)  => format!("{}: {}",   message.sender, m),
        Action(m)   => format!("{}{}",     message.sender, m),
        Ad(m)       => format!("{}^ {}",   message.sender, m),
        Roll(m)     => format!("* {}{}",   message.sender, m),
        Warn(m)     => format!("! {}: {}", message.sender, m),
        Event(m)    => format!("? {}{}",   message.sender, m),
    })
    
}

fn deduplicate_messages(
    locations: &Vec<PathBuf>,
    tab_name: String,
    time_diff: Duration,
    mut w: FChatWriter,
    mut log_buf: BufWriter<File>,
    mut idx_buf: BufWriter<File>,
    dupe_warning: &bool,
) -> Result<(), Error> {
    let mut readers = Vec::with_capacity(locations.len());
    for p in locations {
        let file = File::open(p).map_err(|e| Error::UnableToOpenLog(p.into(), e))?;
        readers.push(Reader::new_buffered(file).peekable())
    }
    let mut messages = BinaryHeap::new();
    loop {
        match messages.peek() {
            None => {
                /* This peeks into all the readers to seed messages with the
                   oldest message and will discard the matching message on the
                   first iteration in the next step but continue normally. This
                   is to prevent redundant messages appearing at the start of
                   the logs since messages are popped and pushed, not indexed.
                */
                let mut index = 0;
                let mut sorted = Vec::with_capacity(readers.len());
                while index < readers.len() {
                    let reader = &mut readers[index];
                    match reader.peek() {
                        Some(Ok(message)) => {
                            sorted.push(Reverse(SortedMessage(message.clone())));
                            index += 1;
                        },
                        Some(Err(_)) => {
                            let _ = reader.next().unwrap()?;
                            panic!("We were expecting an error to unwrap, but it did not.");
                        }
                        None => {
                            let _ = readers.remove(index);
                        }
                    }
                }
                sorted.sort();
                if sorted.is_empty() { 
                    trace!("finished {}", tab_name);
                    break
                } else {
                    messages.push(sorted.remove(0));
                }
            },
            Some(Reverse(SortedMessage(oldest_message))) => {
                // Make a clone since the messages collection will be modified
                let oldest_message_datetime = oldest_message.datetime.clone();
                let mut index = 0;
                while index < readers.len() {
                    let reader = &mut readers[index];
                    match reader.peek() {
                        /* Readers too far in the future are skipped to prevent
                           the collection from getting too big but also the
                           message collection should always contain messages
                           within the current time-diff.
                        */
                        Some(Ok(peeked_message))
                        if peeked_message.datetime >= oldest_message_datetime + time_diff => {
                            index += 1;
                        },
                        Some(Ok(_)) => {
                            let check_message = reader.next().unwrap()?;
                            let mut duplicate = false;
                            let mut duplicate_hit = 0;
                            for Reverse(SortedMessage(message)) in &messages {
                                if check_message.sender == message.sender &&
                                   check_message.body   == message.body
                                {
                                    trace!("Duplicate Hit:\n{}\n{}", format_message(&check_message), format_message(&message));
                                    duplicate = true;
                                    if *dupe_warning {
                                        duplicate_hit += 1;
                                        continue
                                    } else {
                                        break
                                    }
                                }
                            }
                            if !duplicate {
                                messages.push(Reverse(SortedMessage(check_message)));
                            } else if *dupe_warning && duplicate_hit > 1 {
                                warn!("Message was duplicated {} times:\n{}", duplicate_hit, format_message(&check_message));
                            }
                        },
                        Some(Err(_)) => {
                            reader.next().unwrap()?;
                            unreachable!("We were expecting an error to unwrap, but it did not.");
                        },
                        None => {let _ = readers.remove(index);},
                    }
                }
                let Reverse(SortedMessage(message)) = messages.pop().unwrap();
                trace!("Message queue: {}", messages.len());
                trace!("Committing message:\n[{}] {}", tab_name , format_message(&message));
                w.write_message(&mut log_buf, &mut idx_buf, message)?;
            }
        }
    }
    Ok(())
}