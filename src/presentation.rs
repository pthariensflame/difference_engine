// presentation.rs
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
use super::colored::*;
use super::{ExtensionPoint, Provenance};
use Provenance::*;

pub trait Presentation: ExtensionPoint {
  fn present(&self, diff: Vec<(String, Provenance)>);
}

impl<L: Presentation + ?Sized> Presentation for Box<L> {
  fn present(&self, diff: Vec<(String, Provenance)>) { self.as_ref().present(diff); }
}

#[derive(Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,Default,Debug)]
pub struct BasicColored;

impl ExtensionPoint for BasicColored {
  fn name(&self) -> String { "basic-colored".to_string() }

  fn description(&self) -> String { "Makes new-file-only parts red and old-file-only parts green".to_string() }
}

impl Presentation for BasicColored {
  fn present(&self, diff: Vec<(String, Provenance)>) {
    for (x, sx) in diff {
      print!("{}",
             match sx {
               Old => x.red(),
               Shared => x.as_str().into(),
               New => x.green(),
             });
      io::stdout().flush().expect("error when attempting to flush standard ouput");
    }
  }
}

#[derive(Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,Default,Debug)]
pub struct BasicStyled;

impl ExtensionPoint for BasicStyled {
  fn name(&self) -> String { "basic-styled".to_string() }

  fn description(&self) -> String { "Makes new-file-only parts bold and old-file-only parts italic".to_string() }
}

impl Presentation for BasicStyled {
  fn present(&self, diff: Vec<(String, Provenance)>) {
    for (x, sx) in diff {
      print!("{}",
             match sx {
               Old => x.italic(),
               Shared => x.as_str().into(),
               New => x.bold(),
             });
      io::stdout().flush().expect("error when attempting to flush standard ouput");
    }
  }
}
