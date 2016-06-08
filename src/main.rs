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

use std::io;
use std::io::prelude::*;
use std::fs;

#[macro_use]
extern crate clap;
use clap::{App, Arg};

extern crate colored;
use colored::*;

extern crate difference_engine;
// use difference_engine;

fn main() {
  use clap::AppSettings::*;
  let arg_matches = App::new("DEng")
    .version(crate_version!())
    .author("Alexander Altman")
    .about("An extensible language-aware diff")
    .settings(&[ColoredHelp])
    .args(&[Arg::with_name("show languages")
              .long("show-languages")
              .short("L")
              .help("Lists all of the languages currently supported by DEng")
              .conflicts_with_all(&["language name", "old file", "new file"]),
            Arg::with_name("language name")
              .long("language")
              .short("l")
              .help("The language to use when examining the supplied files for \
                     differences.{n}If this option isn't provided, DEng will perform a naïve \
                     character-by-character diff.")
              .takes_value(true),
            Arg::with_name("old file")
              .help("The file to consider “old” for the purposes of finding \
                     differences.{n}DEng will interpret a ‘-’ here as standard input.")
              .required(true),
            Arg::with_name("new file")
              .help("The file to consider “new” for the purposes of finding \
                     differences.{n}DEng will interpret a ‘-’ here as standard input.")
              .required(true)])
    .get_matches();
}
