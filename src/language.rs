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

use super::itertools::*;
use super::diff;

#[derive(Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,Debug)]
pub enum Provenance {
  Old,
  Shared,
  New,
}

pub trait Language {
  fn name(&self) -> String;

  fn description(&self) -> String;

  fn diff(&self, old: String, new: String) -> Vec<(String, Provenance)>;
}

#[derive(Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,Default,Debug)]
pub struct DefaultLanguage;

impl Language for DefaultLanguage {
  fn name(&self) -> String { "default".to_string() }

  fn description(&self) -> String { "The default “language;” implements a naïve character-by-character diff".to_string() }

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

impl<L: Language + ?Sized> Language for Box<L> {
  fn name(&self) -> String { self.as_ref().name() }

  fn description(&self) -> String { self.as_ref().description() }

  fn diff(&self, old: String, new: String) -> Vec<(String, Provenance)> { self.as_ref().diff(old, new) }
}
