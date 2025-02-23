// Copyright 2024 RISC Zero, Inc.
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

use std::result;

use oxrdf::{Dataset, GraphName, Quad};
use oxttl::TurtleParser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spareval::{QueryEvaluator, QueryResults};
use spargebra::Query;

pub mod i;
pub mod blank_node;

// Import I from type.rs
pub use i::{I, I2, I2Content};
pub use blank_node::BlankNode;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Outputs {
    pub data: [u8; 32],
    pub query: [u8; 32],
    pub result: [u8; 32],
    pub result_string: String,
}

// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct I {
//     pub result_string: String,
// }

// impl<'de> Deserialize<'de> for I {
//     fn deserialize<D>(deserializer: D) -> result::Result<I, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let result_string = String::deserialize(deserializer)?;
//         Ok(I { result_string })
//     }
// }

// impl Serialize for I {
//     fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         self.result_string.serialize(serializer)
//     }
// }

// Performance wise, really all that needs to be input is
// a proof of query execution and a verifier
pub fn run(data: &String, query_string: &String, _quads: &I) -> Outputs {
    let result_string = "".to_string();

    if _quads.result_string != "boo" {
        panic!("[IN RUN] Expected 'boo' but got {:?}", _quads.result_string);
    }

    // if _quads.result_string != "boo2" {
    //     panic!("[IN RUN] Expected 'boo2' but got {:?}", _quads.result_string);
    // }

    return Outputs {
        data: Sha256::digest(data).into(),
        query: Sha256::digest(query_string).into(),            
        result: Sha256::digest(result_string.clone()).into(),
        result_string: _quads.result_string.clone(),
    };


    let mut dataset: Dataset = Dataset::new();

    for triple in TurtleParser::new().for_reader(data.as_bytes()) {
        let t1 = triple.unwrap();
        let subject = t1.subject;
        let predicate = t1.predicate;
        let object = t1.object;
        let quad = Quad::new(subject, predicate, object, GraphName::DefaultGraph);
        dataset.insert(&quad);
    }

    let query = Query::parse(query_string, None).unwrap();
    let results = QueryEvaluator::new().execute(dataset, &query);
    let solution: QueryResults = results.unwrap();

    if let QueryResults::Graph(solutions) = solution {
        let mut deset: Dataset = Dataset::from_iter(std::iter::empty::<Quad>());
        for solution in solutions {
            let s = solution.unwrap();
            deset.insert(&Quad::new(
                s.subject,
                s.predicate,
                s.object,
                GraphName::DefaultGraph,
            ));
        }

        // deset.canonicalize(algorithm);

        // let result_string = canonicalize(&deset).unwrap();

        // if _quads.result_string != "boo" {
        //     panic!("[IN RUN] Expected 'boo' but got {:?}", _quads.result_string);
        // }
    
        // if _quads.result_string != "boo2" {
        //     panic!("[IN RUN] Expected 'boo2' but got {:?}", _quads.result_string);
        // }

        return Outputs {
            data: Sha256::digest(data).into(),
            query: Sha256::digest(query_string).into(),            
            result: Sha256::digest(result_string.clone()).into(),
            result_string: result_string,
        };
    }

    panic!("QueryResults::Solutions expected");
}
