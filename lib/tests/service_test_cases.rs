use rudf::model::*;
use rudf::{Repository, RepositoryConnection, MemoryRepository, Result};
use rudf::sparql::{NoneService, PreparedQuery};
use rudf::sparql::QueryResult;
/// 

#[test]
fn service_test() -> Result<()> {
  let repository = MemoryRepository::default();
  let mut connection = repository.connection().unwrap();

  let query = r#"
  SELECT ?s ?p ?o
  WHERE
    { 
      SERVICE <http://service1.org>
      { ?s ?p ?o
      }

      SERVICE <http://service2.org>
      { ?s ?p ?o
      }
    }
  "#;
  let prepared_query = connection.prepare_query(query, None, Some(NoneService)).unwrap();
  let results = prepared_query.exec().unwrap();
  if let QueryResult::Bindings(results) = results {
    let collected = results.into_values_iter().collect::<Vec<_>>();
    println!("Results: {:?}", collected);
  }
  Ok(())
}