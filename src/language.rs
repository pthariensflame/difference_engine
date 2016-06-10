// language.rs
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

use std::path::Path;
use super::itertools::*;
use super::diff;
use diff::Result::*;
use super::libloading::{self, Library, Symbol};
use super::{ExtensionPoint, Provenance};
use Provenance::*;

pub trait Language: ExtensionPoint {
  fn diff(&self, old: String, new: String) -> Vec<(String, Provenance)>;
}

impl<L: Language + ?Sized> Language for Box<L> {
  fn diff(&self, old: String, new: String) -> Vec<(String, Provenance)> { self.as_ref().diff(old, new) }
}

#[derive(Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,Default,Debug)]
pub struct SimpleLinewise;

impl ExtensionPoint for SimpleLinewise {
  fn name(&self) -> String { "simple-linewise".to_string() }

  fn description(&self) -> String { "A “language” that implements a naïve line-by-line diff".to_string() }
}

impl Language for SimpleLinewise {
  fn diff(&self, old: String, new: String) -> Vec<(String, Provenance)> {
    diff::lines(&old, &new)
      .into_iter()
      .map(|cr| match cr {
        Left(x) => (x.to_string() + "\n", Old),
        Right(x) => (x.to_string() + "\n", New),
        Both(x, _) => (x.to_string() + "\n", Shared),
      })
      .coalesce(|(x, sx), (y, sy)| if sx == sy { Ok((x + &y, sx)) } else { Err(((x, sx), (y, sy))) })
      .collect()
  }
}

#[derive(Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,Default,Debug)]
pub struct SimpleCharwise;

impl ExtensionPoint for SimpleCharwise {
  fn name(&self) -> String { "simple-charwise".to_string() }

  fn description(&self) -> String { "A “language” that implements a naïve character-by-character diff".to_string() }
}

impl Language for SimpleCharwise {
  fn diff(&self, old: String, new: String) -> Vec<(String, Provenance)> {
    use diff::Result::*;
    use Provenance::*;
    diff::chars(&old, &new)
      .into_iter()
      .map(|cr| match cr {
        Left(x) => (x.to_string(), Old),
        Right(x) => (x.to_string(), New),
        Both(x, _) => (x.to_string(), Shared),
      })
      .coalesce(|(x, sx), (y, sy)| if sx == sy { Ok((x + &y, sx)) } else { Err(((x, sx), (y, sy))) })
      .collect()
  }
}

#[derive(Debug)]
pub struct RawPluginLanguage {
  lib: Library,
}

impl RawPluginLanguage {
  pub fn load(path: &Path) -> libloading::Result<RawPluginLanguage> {
    Library::new(path).map(|lib| {
      if let Ok(raw_fn) = unsafe { lib.get::<fn()>(b"deng_plugin_initialize") } {
        raw_fn();
      }
      RawPluginLanguage { lib: lib }
    })
  }
}

impl Drop for RawPluginLanguage {
  fn drop(&mut self) {
    if let Ok(raw_fn) = unsafe { self.lib.get::<fn()>(b"deng_plugin_deinitialize") } {
      raw_fn();
    }
  }
}

impl ExtensionPoint for RawPluginLanguage {
  fn name(&self) -> String {
    let raw_fn: Symbol<fn() -> String> = unsafe { self.lib.get(b"deng_plugin_name") }.expect("error in loading raw plugin");
    return raw_fn();
  }

  fn description(&self) -> String {
    let raw_fn: Symbol<fn() -> String> = unsafe { self.lib.get(b"deng_plugin_description") }.expect("error in loading raw plugin");
    return raw_fn();
  }
}

impl Language for RawPluginLanguage {
  fn diff(&self, old: String, new: String) -> Vec<(String, Provenance)> {
    let raw_fn: Symbol<fn(String, String) -> Vec<(String, Provenance)>> = unsafe { self.lib.get(b"deng_plugin_diff") }
      .expect("error in loading raw plugin");
    return raw_fn(old, new);
  }
}
