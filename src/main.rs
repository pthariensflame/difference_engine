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
use std::io;
use std::io::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::ffi::OsStr;
use std::env;
use std::env::consts::{DLL_EXTENSION, DLL_PREFIX};

extern crate itertools;
use itertools::*;

#[macro_use]
extern crate clap;
use clap::Arg;

extern crate colored;
use colored::*;

extern crate difference_engine;
use difference_engine::*;

fn main() {
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
                .conflicts_with_all(&["show presentations", "language name", "presentation name", "old file", "new file"]),
              Arg::with_name("show presentations")
                .long("show-presentations")
                .short("P")
                .help("Lists all of the presentations currently supported by DEng")
                .conflicts_with_all(&["show languages", "language name", "presentation name", "old file", "new file"]),
              Arg::with_name("language name")
                .long("language")
                .short("l")
                .help("The language to use when examining the supplied files for differences.{n}If this option isn't provided, DEng \
                       will perform a naïve line-by-line diff.")
                .takes_value(true),
              Arg::with_name("presentation name")
                .long("presentation")
                .short("p")
                .help("The presentation to use when outputting the calculated diff.{n}If this option isn't provided, DEng will color \
                       new-file-only parts red and old-file-only parts green.")
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

  let mut registered_languages_vec: Vec<Box<Language>> = vec![Box::new(SimpleLinewise), Box::new(SimpleCharwise)];
  let raw_language_plugin_path = env::var_os("DENG_RAW_LANGUAGE_PLUGIN_PATH");
  for plugin in Itertools::flatten(Itertools::flatten(env::current_dir()
      .into_iter()
      .map(|d| d.join(".deng").join("languages").join("raw"))
      .chain(raw_language_plugin_path.as_ref().into_iter().flat_map(env::split_paths))
      .chain(env::home_dir().into_iter().map(|d| d.join(".deng").join("languages").join("raw")))
      .flat_map(|d| d.read_dir().into_iter())
    ))
    .map(|e| e.path())
    .filter(|p| {
      p.file_stem().and_then(|s| s.to_str()).map(|s| s.starts_with(DLL_PREFIX)) == Some(true) &&
      p.extension().map(|s| s == DLL_EXTENSION) == Some(true)
    })
    .flat_map(|p| RawPluginLanguage::load(&p).into_iter()) {
    registered_languages_vec.push(Box::new(plugin));
  }
  let registered_languages: HashMap<String, Box<Language>> =
    registered_languages_vec.into_iter().map(|language| (language.name(), language)).unique_by(|nl| nl.0.clone()).collect();

  let mut registered_presentations_vec: Vec<Box<Presentation>> = vec![Box::new(BasicColored), Box::new(BasicStyled)];
  let raw_presentation_plugin_path = env::var_os("DENG_RAW_PRESENTATION_PLUGIN_PATH");
  for plugin in env::current_dir()
    .into_iter()
    .map(|d| d.join(".deng").join("presentations").join("raw"))
    .chain(raw_presentation_plugin_path.as_ref().into_iter().flat_map(env::split_paths))
    .chain(env::home_dir().into_iter().map(|d| d.join(".deng").join("presentations").join("raw")))
    .flat_map(|d| d.read_dir().into_iter())
    .flatten()
    .flatten()
    .map(|e| e.path())
    .filter(|p| {
      p.file_stem().and_then(|s| s.to_str()).map(|s| s.starts_with(DLL_PREFIX)) == Some(true) &&
      p.extension().map(|s| s == DLL_EXTENSION) == Some(true)
    })
    .flat_map(|p| RawPluginPresentation::load(&p).into_iter()) {
    registered_presentations_vec.push(Box::new(plugin));
  }
  let registered_presentations: HashMap<String, Box<Presentation>> =
    registered_presentations_vec.into_iter().map(|presentation| (presentation.name(), presentation)).unique_by(|np| np.0.clone()).collect();

  if arg_matches.is_present("show languages") {
    for language in registered_languages.values() {
      println!("{}:\t{}", language.name().bold(), language.description());
    }
    return;
  }

  if arg_matches.is_present("show presentations") {
    for presentation in registered_presentations.values() {
      println!("{}:\t{}", presentation.name().bold(), presentation.description());
    }
    return;
  }

  let language_name = arg_matches.value_of("language name").unwrap_or("simple-linewise");
  let language = registered_languages.get(language_name).expect("could not find language");

  let presentation_name = arg_matches.value_of("presentation name").unwrap_or("basic-colored");
  let presentation = registered_presentations.get(presentation_name).expect("could not find presentation");

  let (old_file, new_file) = resolve_files(arg_matches.value_of_os("old file").unwrap(),
                                           arg_matches.value_of_os("new file").unwrap());

  presentation.present(language.diff(old_file, new_file));
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
