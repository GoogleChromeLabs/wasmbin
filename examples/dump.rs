// Copyright 2020 Google Inc. All Rights Reserved.
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

use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use wasmbin::Module;

fn main() {
    let f = File::open(std::env::args().nth(1).expect("expected filename")).unwrap();
    let mut f = BufReader::new(f);
    let m = Module::decode_from(&mut f).unwrap_or_else(|err| {
        panic!(
            "Parsing error at offset 0x{:08X}: {}",
            f.seek(SeekFrom::Current(0)).unwrap(),
            err
        )
    });
    println!("{:#?}", m);
}
