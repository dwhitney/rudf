use rudf::model::*;
use rudf::{DatasetSyntax, Repository, RepositoryConnection, MemoryRepository, Result};
use rudf::sparql::{BindingsIterator, GraphPattern, PreparedQuery, NoneService, QueryResult};
use failure::format_err;

fn ex(id: String) -> Term {
  Term::NamedNode(NamedNode::parse(format!("http://example.com/{}", &id)).unwrap())
}

fn foaf(id: String) -> Term {
  Term::NamedNode(NamedNode::parse(format!("http://xmlns.com/foaf/0.1/{}", &id)).unwrap())
}

fn mailto(id: String) -> Term {
  Term::NamedNode(NamedNode::parse(format!("mailto:{}", &id)).unwrap())
}

#[test]
fn simple_graph_test() {
  let repository = MemoryRepository::default();
  let mut connection = repository.connection().unwrap();
  let ttl = br#"
   <http://example.com/bob> <http://xmlns.com/foaf/0.1/name> "Bob" <http://service1.org> .
   <http://example.com/alice> <http://xmlns.com/foaf/0.1/name> "Alice" <http://service1.org>.

   <http://example.com/bob> <http://xmlns.com/foaf/0.1/mbox> <mailto:bob@example.org> <http://service2.org> .
   <http://example.com/alice> <http://xmlns.com/foaf/0.1/mbox> <mailto:alice@example.org> <http://service2.org> .
  "#;
  connection.load_dataset(ttl.as_ref(), DatasetSyntax::NQuads, None);

  let query = r#"
    PREFIX foaf: <http://xmlns.com/foaf/0.1/>
    SELECT ?name ?mbox
    WHERE
      { 
        GRAPH <http://service1.org>
        { ?s <foaf:name> ?name 
        }
        GRAPH <http://service2.org>
        { ?s <foaf:mbox> ?mbox 
        }
      }
    "#;


  let prepared_query = connection.prepare_query(query, None).unwrap();
  let service_handler = Some(NoneService);
  let results = prepared_query.exec(&service_handler).unwrap();
  if let QueryResult::Bindings(results) = results {
    let collected = results.into_values_iter().map(move |b| b.unwrap()).collect::<Vec<_>>();
    let solution = vec![
      vec![ Some(ex(String::from("s"))), Some(ex(String::from("p"))), Some(ex(String::from("o"))) ],
    ];

    println!("Results: {:?}", collected);
    //assert_eq!(collected, solution);
  } else {
    //assert_eq!(true, false);
  }
}

