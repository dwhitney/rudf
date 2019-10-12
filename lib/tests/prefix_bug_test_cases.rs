use rudf::{DatasetSyntax, Repository, RepositoryConnection, MemoryRepository};
use rudf::sparql::{PreparedQuery, QueryResult};

#[test]
fn prefix_bug_test() {
  let repository = MemoryRepository::default();
  let mut connection = repository.connection().unwrap();
  let ttl = br#"
   <http://example.com/bob> <http://xmlns.com/foaf/0.1/name> "Bob" .
   <http://example.com/alice> <http://xmlns.com/foaf/0.1/name> "Alice" .

   <http://example.com/bob> <http://xmlns.com/foaf/0.1/mbox> <mailto:bob@example.org>  .
   <http://example.com/alice> <http://xmlns.com/foaf/0.1/mbox> <mailto:alice@example.org> .
  "#;
  connection.load_dataset(ttl.as_ref(), DatasetSyntax::NQuads, None).unwrap();

  let bug_query = r#"
    PREFIX foaf: <http://xmlns.com/foaf/0.1/>
    SELECT ?name ?mbox
    WHERE
      { 
        ?s <foaf:name> ?name .
        ?s <foaf:mbox> ?mbox .
      }
    "#;


  let query = r#"
    SELECT ?name ?mbox
    WHERE
      { 
        ?s <http://xmlns.com/foaf/0.1/name> ?name .
        ?s <http://xmlns.com/foaf/0.1/mbox> ?mbox .
      }
    "#;


  let prepared_query = connection.prepare_query(query, None).unwrap();
  let results_query = prepared_query.exec().unwrap();

  let query_set = if let QueryResult::Bindings(results) = results_query {
    results.into_values_iter().map(move |b| b.unwrap()).collect::<Vec<_>>()
  } else {
    vec![]
  };


  let prepared_bug = connection.prepare_query(bug_query, None).unwrap();
  let results_bug = prepared_bug.exec().unwrap();

  let bug_set = if let QueryResult::Bindings(results) = results_bug {
    results.into_values_iter().map(move |b| b.unwrap()).collect::<Vec<_>>()
  } else {
    vec![]
  };

  assert_eq!(query_set, bug_set);
  
}

