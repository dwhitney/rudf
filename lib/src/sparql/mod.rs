//! [SPARQL](https://www.w3.org/TR/sparql11-overview/) implementation.

mod algebra;
mod eval;
mod json_results;
mod model;
mod parser;
mod plan;
mod plan_builder;
mod xml_results;

use crate::model::NamedNode;
use crate::sparql::algebra::QueryVariants;
use crate::sparql::eval::SimpleEvaluator;
use crate::sparql::parser::read_sparql_query;
use crate::sparql::plan::TripleTemplate;
use crate::sparql::plan::{DatasetView, PlanNode};
use crate::sparql::plan_builder::PlanBuilder;
use crate::store::StoreConnection;
use crate::Result;
use std::fmt;

pub use crate::sparql::algebra::GraphPattern;
pub use crate::sparql::model::BindingsIterator;
pub use crate::sparql::model::QueryResult;
pub use crate::sparql::model::QueryResultSyntax;
pub use crate::sparql::model::Variable;

/// A prepared [SPARQL query](https://www.w3.org/TR/sparql11-query/)
pub trait PreparedQuery {
    /// Evaluates the query and returns its results
    fn exec(&self) -> Result<QueryResult>;
}

/// An implementation of `PreparedQuery` for internal use
pub struct SimplePreparedQuery<S: StoreConnection>(SimplePreparedQueryOptions<S>);

enum SimplePreparedQueryOptions<S: StoreConnection> {
    Select {
        plan: PlanNode,
        variables: Vec<Variable>,
        evaluator: SimpleEvaluator<S>,
    },
    Ask {
        plan: PlanNode,
        evaluator: SimpleEvaluator<S>,
    },
    Construct {
        plan: PlanNode,
        construct: Vec<TripleTemplate>,
        evaluator: SimpleEvaluator<S>,
    },
    Describe {
        plan: PlanNode,
        evaluator: SimpleEvaluator<S>,
    },
}

impl<S: StoreConnection> SimplePreparedQuery<S> {
    pub(crate) fn new<'a>(
        connection: S,
        query: &str,
        base_iri: Option<&str>,
        service_handler: Option<fn (NamedNode) -> (fn(GraphPattern) -> Result<BindingsIterator<'a>>)>
        ) -> Result<Self> {
        let dataset = DatasetView::new(connection);
        //TODO avoid inserting terms in the Repository StringStore
        Ok(Self(match read_sparql_query(query, base_iri)? {
            QueryVariants::Select {
                algebra,
                dataset: _,
                base_iri,
            } => {
                let (plan, variables) = PlanBuilder::build(dataset.encoder(), &algebra)?;
                SimplePreparedQueryOptions::Select {
                    plan,
                    variables,
                    evaluator: SimpleEvaluator::new(dataset, base_iri, service_handler),
                }
            }
            QueryVariants::Ask {
                algebra,
                dataset: _,
                base_iri,
            } => {
                let (plan, _) = PlanBuilder::build(dataset.encoder(), &algebra)?;
                SimplePreparedQueryOptions::Ask {
                    plan,
                    evaluator: SimpleEvaluator::new(dataset, base_iri, service_handler),
                }
            }
            QueryVariants::Construct {
                construct,
                algebra,
                dataset: _,
                base_iri,
            } => {
                let (plan, variables) = PlanBuilder::build(dataset.encoder(), &algebra)?;
                SimplePreparedQueryOptions::Construct {
                    plan,
                    construct: PlanBuilder::build_graph_template(
                        dataset.encoder(),
                        &construct,
                        variables,
                    )?,
                    evaluator: SimpleEvaluator::new(dataset, base_iri, service_handler),
                }
            }
            QueryVariants::Describe {
                algebra,
                dataset: _,
                base_iri,
            } => {
                let (plan, _) = PlanBuilder::build(dataset.encoder(), &algebra)?;
                SimplePreparedQueryOptions::Describe {
                    plan,
                    evaluator: SimpleEvaluator::new(dataset, base_iri, service_handler),
                }
            }
        }))
    }
}

impl<S: StoreConnection> PreparedQuery for SimplePreparedQuery<S> {
    fn exec(&self) -> Result<QueryResult<'_>> {
        match &self.0 {
            SimplePreparedQueryOptions::Select {
                plan,
                variables,
                evaluator,
            } => evaluator.evaluate_select_plan(&plan, &variables),
            SimplePreparedQueryOptions::Ask { plan, evaluator } => {
                evaluator.evaluate_ask_plan(&plan)
            }
            SimplePreparedQueryOptions::Construct {
                plan,
                construct,
                evaluator,
            } => evaluator.evaluate_construct_plan(&plan, &construct),
            SimplePreparedQueryOptions::Describe { plan, evaluator } => {
                evaluator.evaluate_describe_plan(&plan)
            }
        }
    }
}

/// A parsed [SPARQL query](https://www.w3.org/TR/sparql11-query/)
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct Query(QueryVariants);

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Query {
    /// Parses a SPARQL query
    pub fn parse(query: &str, base_iri: Option<&str>) -> Result<Self> {
        Ok(Query(read_sparql_query(query, base_iri)?))
    }
}
