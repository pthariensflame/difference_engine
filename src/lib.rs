// lib.rs
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

extern crate diff;
extern crate itertools;
extern crate colored;

#[derive(Eq,Ord,PartialEq,PartialOrd,Hash,Clone,Copy,Debug)]
pub enum Provenance {
  Old,
  Shared,
  New,
}

pub trait ExtensionPoint {
  fn name(&self) -> String;

  fn description(&self) -> String;
}

impl<L: ExtensionPoint + ?Sized> ExtensionPoint for Box<L> {
  fn name(&self) -> String { self.as_ref().name() }

  fn description(&self) -> String { self.as_ref().description() }
}

mod language;
pub use language::*;

mod presentation;
pub use presentation::*;
