use rudf::model::*;
use rudf::{GraphSyntax, Repository, RepositoryConnection, MemoryRepository, Result};
use rudf::sparql::{BindingsIterator, GraphPattern, PreparedQuery, ServiceHandler, NoneService, QueryResult, PlanBuilder};
use failure::format_err;

#[derive(Clone,Copy)]
struct Test1Handler;
impl ServiceHandler for Test1Handler {
    fn handle<'a>(&'a self, named_node: NamedNode) -> Option<(fn(GraphPattern) -> Result<BindingsIterator<'a>>)> {
      Some(handle_service1) 
    }
}

fn handle_service1<'a>(graph_pattern: GraphPattern) -> Result<BindingsIterator<'a>> {
  let repository = MemoryRepository::default();
  let mut connection = repository.connection().unwrap();
  let file = b"<http://example.com> <http://example.com> <http://example.com> .";
  connection.load_graph(file.as_ref(), GraphSyntax::NTriples, None, None).unwrap();
  let prepared_query = connection.prepare_query_from_pattern(&graph_pattern, None).unwrap();
  let result = prepared_query.exec(&Some(Test1Handler)).unwrap();
  match result {
    QueryResult::Bindings(iterator) => {
      let (variables, iter) = iterator.destruct();
      let cloned_iter = iter.collect::<Vec<_>>().into_iter();
      let new_iter = BindingsIterator::new(variables, Box::new(cloned_iter));
      Ok(new_iter)
    },
    _ => Err(format_err!("Excpected bindings but got another QueryResult"))
  }
}


#[test]
fn service_test() -> Result<()> {
  let repository = MemoryRepository::default();
  let connection = repository.connection().unwrap();

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
  let prepared_query = connection.prepare_query(query, None).unwrap();
  let service_handler = Some(Test1Handler);
  let results = prepared_query.exec(&service_handler).unwrap();
  if let QueryResult::Bindings(results) = results {
    let collected = results.into_values_iter().collect::<Vec<_>>();
    println!("Results: {:?}", collected);
  }
  Ok(())
}