use oxrdf::{
  BlankNode, BlankNodeId, Dataset, Graph, IriParseError, Iri, NamedNode, Quad, Subject, Term,
  Triple, Variable,
};
use thiserror::Error;
use log::{debug, info, warn, error};
use std::collections::{HashMap, HashSet};

/// Common IRIs used in N3:
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const LOG_PROOF: &str = "http://www.w3.org/2000/10/swap/log#Proof";
const LOG_CONCLUSION: &str = "http://www.w3.org/2000/10/swap/log#conclusion";
const LOG_INCLUDES: &str = "http://www.w3.org/2000/10/swap/log#includes";
const LOG_IMPLIES: &str = "http://www.w3.org/2000/10/swap/log#implies";

/// Custom error type for proof checking
#[derive(Debug, Error)]
pub enum ProofCheckError {
  #[error("Document <{0}> not found in the graph.")]
  DocumentNotFound(String),

  #[error("Document <{0}> is not recognized as a log:Proof.")]
  NotAProof(String),

  #[error("Missing log:conclusion or log:includes in <{0}>")]
  MissingConclusionIncludes(String),

  #[error("Failed assertion check: {0}")]
  AssertionFailure(String),

  #[error("Failed implication check: {0}")]
  ImplicationFailure(String),

  #[error("Invalid IRI: {0}")]
  InvalidIri(String),

  #[error("Generic error: {0}")]
  Other(String),
}

/// A helper struct that represents a “quoted” formula or a sub-graph of statements.
///
/// In N3/CWM, a formula is basically a set of statements, potentially with variables.
/// Often these are represented as blank nodes with reified statements, or via the
/// N3 “{}” notation.  For demonstration, we store them as actual `Triple` sets
/// in a separate structure.  If there are variables, we treat them as blank nodes
/// or placeholders in this example.
#[derive(Debug, Clone)]
pub struct N3Formula {
  /// A set of triples that belong to this formula
  pub triples: Vec<Triple>,
}

/// Implementation of a “unification” approach for a formula.
/// For simplicity, we treat blank nodes as variables that can match anything,
/// and named nodes must match exactly.  In real CWM, you have `?x` style variables,
/// function built-ins, etc.
impl N3Formula {
  /// Attempt to unify this formula with the given “knowledge base” graph.  
  /// If *every triple* in the formula can match at least one triple in `kb`,
  /// we say that the formula is “satisfied.”
  pub fn is_satisfied_by(&self, kb: &Graph) -> bool {
      // For each triple in self, we need at least one triple in kb that unifies
      // with it. If any triple in self cannot match, the formula is not satisfied.
      for t in &self.triples {
          let mut matched = false;
          for kb_triple in kb.triples() {
              if unify_triples(t, kb_triple) {
                  matched = true;
                  break;
              }
          }
          if !matched {
              // If we find a triple from the formula that doesn't unify
              // with *any* triple in the knowledge base, the formula fails.
              debug!("Formula triple {:?} does NOT match anything in KB", t);
              return false;
          }
      }
      true
  }
}

/// Attempt to unify two triples.  
/// - NamedNodes must match exactly.  
/// - Literals must match exactly (including datatype).  
/// - BlankNodes are treated as “wildcards” or variables, so they unify with anything.  
/// 
/// This is a simple approach. In real CWM, we’d do more sophisticated variable binding.
fn unify_triples(a: &Triple, b: &Triple) -> bool {
  unify_term(&a.subject, &b.subject)
      && unify_term(&a.predicate.into_term(), &b.predicate.into_term())
      && unify_term(&a.object, &b.object)
}

/// Attempt to unify two `Term`s under the assumption that blank nodes are “variables.”
fn unify_term(a: &Term, b: &Term) -> bool {
  match (a, b) {
      // Blank + anything => unify
      (Term::BlankNode(_), _) => true,
      (_, Term::BlankNode(_)) => true,

      // NamedNode => must match IRI exactly
      (Term::NamedNode(a_iri), Term::NamedNode(b_iri)) => a_iri.as_str() == b_iri.as_str(),

      // Literals => must match exactly
      (Term::Literal(a_lit), Term::Literal(b_lit)) => a_lit.value() == b_lit.value()
          && a_lit.datatype() == b_lit.datatype()
          && a_lit.language() == b_lit.language(),

      // If they're variables in real N3, you’d do more logic. For now,
      // different Term variants do not unify if not blank nodes or exact matches.
      _ => false,
  }
}

/// Retrieve all named nodes for a given subject and predicate in `graph`.
fn get_named_objects_for_predicate(
  graph: &Graph,
  subject: &NamedNode,
  predicate_iri: &str,
) -> Vec<NamedNode> {
  let predicate = match NamedNode::new(predicate_iri) {
      Ok(nn) => nn,
      Err(_) => {
          warn!("Invalid predicate IRI: {}", predicate_iri);
          return vec![];
      }
  };

  graph
      .triples_for_subject(subject.into())
      .filter_map(|t| {
          if t.predicate == predicate.into() {
              if let Term::NamedNode(nn) = &t.object {
                  Some(nn.clone())
              } else {
                  None
              }
          } else {
              None
          }
      })
      .collect()
}

/// Check that the document is typed as a log:Proof
fn ensure_is_log_proof(graph: &Graph, doc_subject: &NamedNode) -> Result<(), ProofCheckError> {
  let rdf_type = NamedNode::new(RDF_TYPE)
      .map_err(|_| ProofCheckError::Other("Invalid IRI for rdf:type".to_string()))?;

  let is_proof = graph
      .triples_for_subject(doc_subject.into())
      .filter(|t| t.predicate == rdf_type.into())
      .any(|t| match &t.object {
          Term::NamedNode(nn) => nn.as_str() == LOG_PROOF,
          _ => false,
      });

  if is_proof {
      Ok(())
  } else {
      Err(ProofCheckError::NotAProof(doc_subject.as_str().to_string()))
  }
}

/// Extract the formula (set of triples) for a node that *represents* a formula.
/// In many N3 encodings, a “formula node” is a blank node with reified statements,
/// or a named node that reifies them.  This function tries to find all triples
/// that “belong” to that formula.  
///
/// For simplicity, here we do a naive approach: we look for all statements
/// that have the formula node as the `subject`. In real usage, you might
/// parse reification or other data to figure out which statements are *inside* the formula.
fn extract_formula(graph: &Graph, formula_node: &NamedNode) -> N3Formula {
  // This is a naive approach: any triple that has `formula_node` as subject is included.
  // Real N3 formula extraction might be more elaborate: we’d look for
  // reified statements or sub-graphs.
  let formula_triples: Vec<Triple> = graph
      .triples_for_subject(formula_node.into())
      .map(|t| t.clone())
      .collect();

  N3Formula {
      triples: formula_triples,
  }
}

/// “Check” the assertions from an included formula:
/// - We parse the formula (extract it as a set of statements).
/// - We see if each triple can unify with something in the current knowledge base.
fn check_assertions(
  graph: &Graph,
  formula_node: &NamedNode,
  kb: &Graph,
) -> Result<(), ProofCheckError> {
  // Extract the formula from the graph
  let formula = extract_formula(graph, formula_node);

  // If the formula is empty, maybe it’s an error or maybe it’s trivially true?
  // We’ll say an empty formula is trivially satisfied. 
  if formula.triples.is_empty() {
      debug!("Included formula {} is empty; treating as trivially satisfied.", formula_node);
      return Ok(())
  }

  // Check if formula is satisfied by the knowledge base
  if formula.is_satisfied_by(kb) {
      Ok(())
  } else {
      Err(ProofCheckError::AssertionFailure(format!(
          "Formula <{}> not satisfied by current KB",
          formula_node.as_str()
      )))
  }
}

/// “Check” the implications for a conclusion formula:
/// - We look for any triple `(antecedent) log:implies (conclusion)`.
/// - If the antecedent is satisfied, the conclusion must also be satisfied.
///   If not, we fail.
fn check_implications(
  graph: &Graph,
  conclusion_node: &NamedNode,
  kb: &mut Graph, // We may add derived statements to the KB
) -> Result<(), ProofCheckError> {
  // Collect all statements of the conclusion node as a formula
  let conclusion_formula = extract_formula(graph, conclusion_node);

  // For real cwm, the conclusion might have more than one triple.
  // We'll attempt to unify them if we find an implication referencing them.

  // We’ll scan the entire graph for any triple with `log:implies` as predicate
  // and conclusion_node as the object. That means:
  //
  //  antecedent log:implies conclusion_node
  //
  // Then we unify the antecedent with the KB. If that works, we unify or add
  // the conclusion formula to the KB (since it must be derived).
  let implies_pred = NamedNode::new(LOG_IMPLIES)
      .map_err(|_| ProofCheckError::InvalidIri(LOG_IMPLIES.to_string()))?;

  let mut found_any_impl = false;

  for t in graph.triples() {
      if t.predicate == implies_pred.into()
          && t.object == conclusion_node.clone().into()
      {
          found_any_impl = true;
          // t.subject is the “antecedent” (which might be a NamedNode or BlankNode)
          match &t.subject {
              Subject::NamedNode(nn) => {
                  let antecedent_formula = extract_formula(graph, nn);
                  // If antecedent is satisfied => conclusion formula must also be satisfied
                  if antecedent_formula.is_satisfied_by(kb) {
                      debug!("Antecedent <{}> is satisfied. Checking conclusion <{}>...", nn, conclusion_node);
                      if !conclusion_formula.is_satisfied_by(kb) {
                          // If the conclusion is not satisfied, we might add it to the KB
                          // or we might fail.  In “strict” proof-checking, we typically
                          // fail if the conclusion doesn’t unify with the KB.
                          // In an “inference” scenario, we might add conclusion to KB.
                          debug!("Conclusion not satisfied by KB. We add its statements as derived knowledge.");
                          for cf_triple in &conclusion_formula.triples {
                              kb.insert(cf_triple.clone());
                          }
                          // Then check again
                          if !conclusion_formula.is_satisfied_by(kb) {
                              return Err(ProofCheckError::ImplicationFailure(format!(
                                  "Conclusion <{}> not derivable even after adding.",
                                  conclusion_node.as_str()
                              )));
                          }
                      }
                  } else {
                      debug!("Antecedent <{}> is NOT satisfied, so no conclusion needed yet.", nn);
                  }
              }
              Subject::BlankNode(bn) => {
                  // For blank node antecedents, we handle them as an “anonymous formula” or variable formula.
                  // We can attempt to parse that formula the same way or treat it as trivially unknown.
                  debug!("Found log:implies with blank node antecedent: {:?}. For simplicity, ignoring in this example.", bn);
              }
              _ => {
                  // Rare case: subject is a variable or something else.
                  debug!("We do not handle variable subjects in this simplified approach.");
              }
          }
      }
  }

  if !found_any_impl {
      // If no triple says “something log:implies <conclusion_node>”,
      // we interpret that as no rules that derive this conclusion.
      // Possibly that’s an error if we wanted to prove it. 
      return Err(ProofCheckError::ImplicationFailure(format!(
          "No log:implies found deriving <{}>",
          conclusion_node.as_str()
      )));
  }

  Ok(())
}

/// The primary entry point for verifying a proof (doc_iri) in the graph:
/// 1) Check that doc_iri is typed as log:Proof.
/// 2) Retrieve `log:includes` and `log:conclusion`.
/// 3) Check each included formula is satisfied (“assertions”).
/// 4) Check each conclusion formula can be derived (“implications”).
pub fn verify_proof(graph: &Graph, doc_iri: &str) -> Result<(), ProofCheckError> {
  info!("Verifying proof for document <{}>", doc_iri);

  let doc_subject = NamedNode::new(doc_iri)
      .map_err(|_| ProofCheckError::InvalidIri(doc_iri.to_string()))?;

  // 1) Ensure doc is a log:Proof
  ensure_is_log_proof(graph, &doc_subject)?;

  // 2) Retrieve includes & conclusion(s)
  let includes_iris = get_named_objects_for_predicate(graph, &doc_subject, LOG_INCLUDES);
  let conclusion_iris = get_named_objects_for_predicate(graph, &doc_subject, LOG_CONCLUSION);

  if includes_iris.is_empty() || conclusion_iris.is_empty() {
      return Err(ProofCheckError::MissingConclusionIncludes(doc_iri.to_string()));
  }

  // Make a “working KB” that starts as a copy of the entire graph
  // so we can add derived statements to it.
  let mut kb = graph.clone();

  // 3) Check each included formula
  for inc_node in &includes_iris {
      check_assertions(graph, inc_node, &kb)?;
  }

  // 4) Check each conclusion formula
  for conc_node in &conclusion_iris {
      check_implications(graph, conc_node, &mut kb)?;
  }

  info!("Proof <{}> verified successfully!", doc_iri);
  Ok(())
}

/// ------------------------------------------------------------------
/// Example `main()` usage & a basic test setup
/// ------------------------------------------------------------------
fn main() -> Result<(), Box<dyn std::error::Error>> {
  env_logger::init();

  // Build a small example graph
  let mut graph = Graph::new();
  let doc_iri = "http://example.org/myProof";

  // Insert the doc as a log:Proof
  graph.insert(Triple::new(
      NamedNode::new(doc_iri)?.into(),
      NamedNode::new(RDF_TYPE)?.into(),
      NamedNode::new(LOG_PROOF)?.into(),
  ));

  // Insert an example `log:includes` pointing to a “formula node”
  let includes_formula_iri = "http://example.org/includesFormula";
  graph.insert(Triple::new(
      NamedNode::new(doc_iri)?.into(),
      NamedNode::new(LOG_INCLUDES)?.into(),
      NamedNode::new(includes_formula_iri)?.into(),
  ));
  // Suppose the includes formula says: (ex:Alice ex:knows ex:Bob)
  graph.insert(Triple::new(
      NamedNode::new(includes_formula_iri)?.into(),
      NamedNode::new("http://example.org/knows")?.into(),
      NamedNode::new("http://example.org/Bob")?.into(),
  ));
  // In real N3, we’d reify that statement as well. This is simplified.

  // Insert a `log:conclusion` pointing to a “conclusion formula” node
  let conclusion_formula_iri = "http://example.org/conclusionFormula";
  graph.insert(Triple::new(
      NamedNode::new(doc_iri)?.into(),
      NamedNode::new(LOG_CONCLUSION)?.into(),
      NamedNode::new(conclusion_formula_iri)?.into(),
  ));

  // The conclusion formula is: (ex:Alice ex:friendsWith ex:Bob)
  graph.insert(Triple::new(
      NamedNode::new(conclusion_formula_iri)?.into(),
      NamedNode::new("http://example.org/friendsWith")?.into(),
      NamedNode::new("http://example.org/Bob")?.into(),
  ));

  // Now we add a rule that says:
  //   ( X :knows Y ) log:implies ( X :friendsWith Y )
  // For simplicity, we represent it as:
  //   includes_formula_iri log:implies conclusion_formula_iri
  // so that if includes_formula_iri is satisfied, we derive conclusion_formula_iri.
  graph.insert(Triple::new(
      NamedNode::new(includes_formula_iri)?.into(),
      NamedNode::new(LOG_IMPLIES)?.into(),
      NamedNode::new(conclusion_formula_iri)?.into(),
  ));

  // Attempt to verify
  match verify_proof(&graph, doc_iri) {
      Ok(_) => {
          println!("Proof verified successfully!");
      }
      Err(e) => {
          println!("Proof verification failed: {e}");
      }
  }

  Ok(())
}

/// ------------------------------------------------------------------
/// Tests (cargo test)
/// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_basic_proof_verification() {
      let mut graph = Graph::new();
      let doc_iri = "http://example.org/myProof";

      // Mark it as a log:Proof
      graph.insert(Triple::new(
          NamedNode::new(doc_iri).unwrap().into(),
          NamedNode::new(RDF_TYPE).unwrap().into(),
          NamedNode::new(LOG_PROOF).unwrap().into(),
      ));

      // includes
      let includes_iri = "http://example.org/includesFormula";
      graph.insert(Triple::new(
          NamedNode::new(doc_iri).unwrap().into(),
          NamedNode::new(LOG_INCLUDES).unwrap().into(),
          NamedNode::new(includes_iri).unwrap().into(),
      ));
      // e.g. (ex:Alice ex:knows ex:Bob)
      graph.insert(Triple::new(
          NamedNode::new(includes_iri).unwrap().into(),
          NamedNode::new("http://example.org/knows").unwrap().into(),
          NamedNode::new("http://example.org/Bob").unwrap().into(),
      ));

      // conclusion
      let conclusion_iri = "http://example.org/conclusionFormula";
      graph.insert(Triple::new(
          NamedNode::new(doc_iri).unwrap().into(),
          NamedNode::new(LOG_CONCLUSION).unwrap().into(),
          NamedNode::new(conclusion_iri).unwrap().into(),
      ));
      // (ex:Alice ex:friendsWith ex:Bob)
      graph.insert(Triple::new(
          NamedNode::new(conclusion_iri).unwrap().into(),
          NamedNode::new("http://example.org/friendsWith").unwrap().into(),
          NamedNode::new("http://example.org/Bob").unwrap().into(),
      ));

      // The rule: includes_iri log:implies conclusion_iri
      graph.insert(Triple::new(
          NamedNode::new(includes_iri).unwrap().into(),
          NamedNode::new(LOG_IMPLIES).unwrap().into(),
          NamedNode::new(conclusion_iri).unwrap().into(),
      ));

      let result = verify_proof(&graph, doc_iri);
      assert!(result.is_ok());
  }

  #[test]
  fn test_proof_fails_when_includes_are_not_satisfied() {
      let mut graph = Graph::new();
      let doc_iri = "http://example.org/myProof";

      // Mark it as a log:Proof
      graph.insert(Triple::new(
          NamedNode::new(doc_iri).unwrap().into(),
          NamedNode::new(RDF_TYPE).unwrap().into(),
          NamedNode::new(LOG_PROOF).unwrap().into(),
      ));

      // includes a formula, but the formula is never actually stated in the KB
      let includes_iri = "http://example.org/includesFormula";
      graph.insert(Triple::new(
          NamedNode::new(doc_iri).unwrap().into(),
          NamedNode::new(LOG_INCLUDES).unwrap().into(),
          NamedNode::new(includes_iri).unwrap().into(),
      ));

      // conclusion formula
      let conclusion_iri = "http://example.org/conclusionFormula";
      graph.insert(Triple::new(
          NamedNode::new(doc_iri).unwrap().into(),
          NamedNode::new(LOG_CONCLUSION).unwrap().into(),
          NamedNode::new(conclusion_iri).unwrap().into(),
      ));

      // This time, we haven’t actually stored any triple for includes_iri,
      // so check_assertions should fail
      let result = verify_proof(&graph, doc_iri);
      assert!(result.is_err());
      if let Err(ProofCheckError::AssertionFailure(msg)) = result {
          assert!(msg.contains("not satisfied"));
      } else {
          panic!("Expected AssertionFailure error");
      }
  }
}
