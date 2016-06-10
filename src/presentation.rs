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
use std::path::Path;
use super::colored::*;
use super::libloading::{self, Library, Symbol};
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
               Shared => x.normal(),
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

  fn description(&self) -> String { "Makes new-file-only parts bold and old-file-only parts underlined".to_string() }
}

impl Presentation for BasicStyled {
  fn present(&self, diff: Vec<(String, Provenance)>) {
    for (x, sx) in diff {
      print!("{}",
             match sx {
               Old => x.underline(),
               Shared => x.normal(),
               New => x.bold(),
             });
      io::stdout().flush().expect("error when attempting to flush standard ouput");
    }
  }
}

#[derive(Debug)]
pub struct RawPluginPresentation {
  lib: Library,
}

impl RawPluginPresentation {
  pub fn load(path: &Path) -> libloading::Result<RawPluginPresentation> {
    Library::new(path).map(|lib| {
      if let Ok(raw_fn) = unsafe { lib.get::<fn()>(b"deng_plugin_initialize") } {
        raw_fn();
      }
      RawPluginPresentation { lib: lib }
    })
  }
}

impl Drop for RawPluginPresentation {
  fn drop(&mut self) {
    if let Ok(raw_fn) = unsafe { self.lib.get::<fn()>(b"deng_plugin_deinitialize") } {
      raw_fn();
    }
  }
}

impl ExtensionPoint for RawPluginPresentation {
  fn name(&self) -> String {
    let raw_fn: Symbol<fn() -> String> = unsafe { self.lib.get(b"deng_plugin_name") }.expect("error in loading raw plugin");
    return raw_fn();
  }

  fn description(&self) -> String {
    let raw_fn: Symbol<fn() -> String> = unsafe { self.lib.get(b"deng_plugin_description") }.expect("error in loading raw plugin");
    return raw_fn();
  }
}

impl Presentation for RawPluginPresentation {
  fn present(&self, diff: Vec<(String, Provenance)>) {
    let raw_fn: Symbol<fn(Vec<(String, Provenance)>)> = unsafe { self.lib.get(b"deng_plugin_present") }
      .expect("error in loading raw plugin");
    raw_fn(diff);
  }
}
