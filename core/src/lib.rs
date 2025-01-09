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

use json::stringify;
use oxrdf::{Dataset, GraphName, NamedNode, Quad};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spareval::{QueryEvaluator, QueryResults, QuerySolution, QuerySolutionIter};
use spargebra::Query;
use oxsdatatypes::{DayTimeDuration, Duration};

#[no_mangle]
fn custom_ox_now() -> Duration {
  return DayTimeDuration::new(0).into()
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Outputs {
    // pub query_result: [u8; 32],
    pub data: [u8; 32],
    pub query: [u8; 32],
}

pub fn run(data: &String, query_string: &String) -> Outputs {
    let ex = NamedNode::new("http://example.com").unwrap();
    let dataset = Dataset::from_iter([Quad::new(
        ex.clone(),
        ex.clone(),
        ex.clone(),
        GraphName::DefaultGraph,
    )]);

    let query = Query::parse(query_string, None).unwrap();
    let results = QueryEvaluator::new().execute(dataset, &query);
    let solution: QueryResults = results.unwrap();
   
    if let QueryResults::Solutions(solutions) = solution {
        let solutions = solutions.collect::<Result<Vec<_>, _>>().unwrap();
        
        // solutions[0].unwrap();
        return Outputs {
            // query_result: Sha256::digest(json::stringify(solutions)).into(),
            data: Sha256::digest(data).into(),
            query: Sha256::digest(query_string).into(),
            // query_hash: Sha256::digest(query).into(),
        };
    }

    panic!("QueryResults::Solutions expected");
}
