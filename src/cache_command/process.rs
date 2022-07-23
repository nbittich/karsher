use std::{collections::BTreeMap, path::Path};

use nom::error::ErrorKind;

use crate::{
    adana_script::Primitive,
    db::DbOp,
    prelude::colors::*,
    reserved_keywords::{check_reserved_keyword, CACHE_COMMAND_DOC},
};

use super::{
    cache::*, clear_terminal, os_command::exec_command, parser::parse_command,
    CacheCommand,
};

const BACKUP_FILE_NAME: &str = "adanadb.json";

pub fn process_command(
    db: &mut impl DbOp<String, String>,
    script_context: &BTreeMap<String, Primitive>,
    current_cache: &mut String,
    line: &str,
) -> anyhow::Result<()> {
    match parse_command(line) {
        Ok((_, command)) => {
            match command {
                CacheCommand::Put { aliases, value } => {
                    if check_reserved_keyword(&aliases) {
                        eprintln!("You cannot use a reserved keyword name as an alias.");
                    } else if let Some(key) =
                        insert_value(db, current_cache, aliases, value)
                    {
                        println!(
                            "added {} with hash keys {}",
                            Yellow.paint(value),
                            Red.paint(key)
                        );
                    } else {
                        eprintln!("could not insert!");
                    }
                }
                CacheCommand::Del(key) => {
                    if let Some(v) = remove_value(db, current_cache, key) {
                        println!(
                            "removed {} with hash key {}",
                            Yellow.paint(v),
                            Red.paint(key.to_string())
                        );
                    } else {
                        println!(
                        "key {key} not found in current cache {current_cache}"
                    );
                    }
                }
                CacheCommand::Get(key) => {
                    if let Some(value) = get_value(db, current_cache, key) {
                        println!("found '{}'", Yellow.paint(value));
                    } else {
                        println!("{key} not found");
                    }
                }
                CacheCommand::Exec { key, args } => {
                    if let Some(value) = get_value(db, current_cache, key) {
                        let _ = exec_command(&value, &args)
                            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
                    } else if !key.trim().is_empty() {
                        println!("{key} not found");
                    }
                }
                CacheCommand::Using(key) => {
                    if set_default_cache(db, key).is_some() {
                        println!(
                            "previous: {}",
                            LightCyan.paint(current_cache.as_str())
                        );
                        current_cache.clear();
                        current_cache.push_str(key);
                    }
                }
                CacheCommand::ListCache => {
                    println!(
                        ">> [ {} ]",
                        get_cache_names(db)
                            .iter()
                            .map(|c| Red.bold().paint(c).to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                CacheCommand::CurrentCache => {
                    println!(
                        ">> {}",
                        LightBlue.bold().paint(current_cache.to_string())
                    );
                }
                CacheCommand::Merge(key) if key == current_cache => {
                    eprintln!("You cannot merge a cache with itself!")
                }
                CacheCommand::Merge(key) => {
                    if merge(db, key, current_cache).is_some() {
                        println!(
                            "cache {} has been merged with cache {}.",
                            Red.bold().paint(&current_cache.to_string()),
                            Yellow.bold().paint(key)
                        );
                    } else {
                        eprintln!("something went wrong!");
                    }
                }
                CacheCommand::Dump(key) => {
                    if let Some(json) = dump(db, key) {
                        println!("{json}");
                    } else {
                        println!("cache doesn't exist!");
                    }
                }
                CacheCommand::DeleteCache(key) => {
                    if let Some(cache_name) = key {
                        if cache_name != current_cache {
                            println!(
                                "remove {cache_name}: {}",
                                remove_cache(db, cache_name)
                            );
                        } else {
                            clear_values(db, current_cache);
                            println!("clear all values from {current_cache}",);
                        }
                    }
                }
                CacheCommand::Describe => {
                    if let Some(values) = list_values(db, current_cache) {
                        for (key, value) in values {
                            println!(
                                ">> Key: {} => Value: '{}'",
                                Red.paint(key),
                                LightCyan.paint(value)
                            );
                        }
                    }
                }
                CacheCommand::Backup => {
                    let backup_path =
                        std::env::current_dir()?.join(BACKUP_FILE_NAME);
                    let backup_path = backup_path.as_path();
                    if let Some(()) = backup(db, backup_path) {
                        println!(
                            "db backed up to {}",
                            Red.paint(backup_path.to_string_lossy())
                        );
                    }
                }
                CacheCommand::Restore => {
                    let backup_path =
                        std::env::current_dir()?.join(BACKUP_FILE_NAME);
                    let backup_path = backup_path.as_path();
                    if let Some(()) = restore(db, backup_path) {
                        println!(
                            "db restored from {}",
                            Red.paint(backup_path.to_string_lossy())
                        );
                    }
                }
                CacheCommand::Cd(path) => {
                    if Path::new(path).exists() {
                        std::env::set_current_dir(path)?;
                        println!(
                            ">> working directory {}",
                            LightMagenta.paint(path)
                        );
                    } else {
                        eprintln!("path {} doesn't exist", Red.paint(path));
                    }
                }
                CacheCommand::Help => {
                    for doc in CACHE_COMMAND_DOC {
                        let (command, doc) = doc;
                        println!(
                            ">> {} : {}",
                            command
                                .iter()
                                .map(|c| Yellow.paint(*c).to_string())
                                .collect::<Vec<_>>()
                                .join("/"),
                            LightBlue.paint(*doc)
                        );
                    }
                }
                CacheCommand::Clear => {
                    clear_terminal();
                }
                CacheCommand::PrintScriptContext => {
                    let json = serde_json::to_string_pretty(&script_context)?;
                    println!("{json}")
                }
            }
        }
        Err(e) => match e {
            nom::Err::Failure(failure) if failure.code == ErrorKind::Verify => {
                eprintln!("invalid command: {}", Red.paint(failure.to_string()))
            }
            nom::Err::Error(err) if err.input.trim().is_empty() => {}
            _ => {
                eprintln!("error parsing command: {}", Red.paint(e.to_string()))
            }
        },
    }
    Ok(())
}
