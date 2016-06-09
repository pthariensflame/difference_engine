// main.rs
// Copyright 2016 Alexander Altman
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::iter;
use std::io;
use std::io::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::ffi::OsStr;

#[macro_use]
extern crate clap;
use clap::Arg;

extern crate colored;
use colored::*;

extern crate difference_engine;
use difference_engine::*;

fn main() {
  let registered_languages_raw = vec![Box::new(DefaultLanguage) as Box<Language>];
  let registered_languages: HashMap<String, Box<Language>> =
    registered_languages_raw.into_iter().map(|language| (language.name(), language)).collect();

  let arg_matches =
    clap::App::new("DEng, the Difference Engine")
      .version(crate_version!())
      .author("Alexander Altman")
      .about("An extensible language-aware diff")
      .settings(&[clap::AppSettings::ColoredHelp])
      .args(&[Arg::with_name("show languages")
                .long("show-languages")
                .short("L")
                .help("Lists all of the languages currently supported by DEng")
                .conflicts_with_all(&["language name", "old file", "new file"]),
              Arg::with_name("language name")
                .long("language")
                .short("l")
                .help("The language to use when examining the supplied files for differences.{n}If this option isn't provided, DEng \
                       will perform a naïve character-by-character diff.")
                .takes_value(true),
              Arg::with_name("old file")
                .help("The file to consider “old” for the purposes of finding differences.{n}DEng will interpret a ‘-’ here as \
                       standard input.")
                .required(true),
              Arg::with_name("new file")
                .help("The file to consider “new” for the purposes of finding differences.{n}DEng will interpret a ‘-’ here as \
                       standard input.")
                .required(true)])
      .get_matches();

  if arg_matches.is_present("show languages") {
    for language in registered_languages.values() {
      println!("{}: {}", language.name().bold(), language.description());
    }
    return;
  }

  let language_name = arg_matches.value_of("language name").unwrap_or("default");
  let language = registered_languages.get(language_name).expect("could not find language");
  let (old_file, new_file) = resolve_files(arg_matches.value_of_os("old file").unwrap(),
                                           arg_matches.value_of_os("new file").unwrap());

  let diff_result = language.diff(old_file, new_file);
}

fn resolve_files(old_file_arg: &OsStr, new_file_arg: &OsStr) -> (String, String) {
  let old_file_path = handle_stdin_notation(old_file_arg).map(canonicalize_path);
  let new_file_path = handle_stdin_notation(new_file_arg).map(canonicalize_path);
  let old_file = old_file_path.clone()
                              .map(|path| read_whole_file(fs::File::open(path).expect("error opening file")))
                              .unwrap_or_else(|| read_whole_file(io::stdin()));
  let new_file;
  if old_file_path == new_file_path.clone() {
    new_file = old_file.clone();
  } else {
    new_file = new_file_path.map(|path| read_whole_file(fs::File::open(path).expect("error opening file")))
                            .unwrap_or_else(|| read_whole_file(io::stdin()));
  }
  return (old_file, new_file);
}

fn handle_stdin_notation(arg: &OsStr) -> Option<&OsStr> {
  if arg == "-" {
    return None;
  } else {
    return Some(arg);
  }
}

fn canonicalize_path(path: &OsStr) -> PathBuf { fs::canonicalize(path).expect("error locating file") }

fn read_whole_file<F: Read>(mut file: F) -> String {
  let mut result = String::new();
  file.read_to_string(&mut result).expect("error reading file");
  return result;
}
