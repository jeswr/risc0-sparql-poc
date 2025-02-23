// Copyright 2023 RISC Zero, Inc.
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

#![no_main]

use json_core::{I, BlankNode, run, I2, I2Content};
use risc0_zkvm::guest::env;
use oxrdf::{NamedNode, Quad, Literal};

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let data: String = env::read();
    let query: String = env::read();
    let quads: I = env::read();

    if quads.result_string != "boo" {
        panic!("Expected 'boo' but got {:?}", quads.result_string);
    }

    let i2: I2 = env::read();

    match i2 {
        I2(I2Content::A) => (),
        _ => panic!("Expected 'A' but got {:?}", i2),
    }

    let _q: NamedNode = env::read();

    if _q.as_str() != "http://example.com/subject" {
        panic!("[NamedNode] Expected 'http://example.com/subject' but got {:?}", _q.as_str());
    }

    // let _q2: BlankNode = env::read();

    // if "a" != "b" {
    //     panic!("[BlankNode] Expected 'a' but got {:?}", "b");
    // }

    // if _q2.as_str() != "http://example.com/subject" {
    //     panic!("[BlankNode] Expected 'http://example.com/subject' but got {:?}", _q2.as_str());
    // }

    // let _q3: Literal = env::read();

    if quads.result_string != "boo" {
        panic!("Expected 'boo' but got {:?}", quads.result_string);
    }

    // if quads.result_string != "boo2" {
    //     panic!("Expected 'boo2' but got {:?}", quads.result_string);
    // }
    
    let out = run(&data, &query, &quads);
    env::commit(&out);
}
