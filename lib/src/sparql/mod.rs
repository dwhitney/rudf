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
use crate::sparql::plan::PlanNode;
use crate::store::StoreConnection;
use crate::Result;

use std::fmt;
use rio_api::iri::{Iri};

pub use crate::sparql::plan_builder::PlanBuilder;
pub use crate::sparql::plan::DatasetView;
pub use crate::sparql::algebra::GraphPattern;
pub use crate::sparql::model::BindingsIterator;
pub use crate::sparql::model::QueryResult;
pub use crate::sparql::model::QueryResultSyntax;
pub use crate::sparql::model::Variable;


//pub type ServiceHandler<'a> = Option<fn (NamedNode) -> Option<(fn(GraphPattern) -> Result<BindingsIterator<'a>>)>>;
// Box<dyn Iterator<Item = Result<Quad>> + 'a>

pub trait ServiceHandler : Copy {
    //fn handle<'a>(&'a self, node: NamedNode) -> Option<(fn(GraphPattern) -> Result<Box<dyn Iterator<Item = Result<Quad>> + 'a>>)>;
    fn handle<'a>(&'a self, node: NamedNode) -> Option<(fn(GraphPattern) -> Result<BindingsIterator<'a>>)>;
}

#[derive(Copy, Clone)]
pub struct NoneService;

impl ServiceHandler for NoneService {
    fn handle<'a>(&'a self, _: NamedNode) -> Option<(fn(GraphPattern) -> Result<BindingsIterator<'a>>)> {
        None
    }
}

/// A prepared [SPARQL query](https://www.w3.org/TR/sparql11-query/)
pub trait PreparedQuery {
    /// Evaluates the query and returns its results
    fn exec<'a, H: ServiceHandler + 'a>(&'a self, service_handler: &'a Option<H>) -> Result<QueryResult>;
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
        variables: Vec<Variable>,
        evaluator: SimpleEvaluator<S>,
    },
    Construct {
        plan: PlanNode,
        variables: Vec<Variable>,
        construct: Vec<TripleTemplate>,
        evaluator: SimpleEvaluator<S>,
    },
    Describe {
        plan: PlanNode,
        variables: Vec<Variable>,
        evaluator: SimpleEvaluator<S>,
    },
}

impl<S: StoreConnection> SimplePreparedQuery<S> {

    pub(crate) fn new<'a>(
        connection: S,
        query: &str,
        base_iri: Option<&str>
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
                    evaluator: SimpleEvaluator::new(dataset, base_iri),
                }
            }
            QueryVariants::Ask {
                algebra,
                dataset: _,
                base_iri,
            } => {
                let (plan, variables) = PlanBuilder::build(dataset.encoder(), &algebra)?;
                SimplePreparedQueryOptions::Ask {
                    plan,
                    variables,
                    evaluator: SimpleEvaluator::new(dataset, base_iri),
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
                    variables: variables.clone(),
                    construct: PlanBuilder::build_graph_template(
                        dataset.encoder(),
                        &construct,
                        variables,
                    )?,
                    evaluator: SimpleEvaluator::new(dataset, base_iri),
                }
            }
            QueryVariants::Describe {
                algebra,
                dataset: _,
                base_iri,
            } => {
                let (plan, variables) = PlanBuilder::build(dataset.encoder(), &algebra)?;
                SimplePreparedQueryOptions::Describe {
                    plan,
                    variables,
                    evaluator: SimpleEvaluator::new(dataset, base_iri),
                }
            }
        }))
    }

    pub(crate) fn new_from_pattern<'a>(
        connection: S,
        pattern: &GraphPattern,
        base_iri: Option<&str>
    ) -> Result<Self> {
        let dataset = DatasetView::new(connection);
        let (plan, variables) = PlanBuilder::build(dataset.encoder(), pattern)?;
        let iri = base_iri.map(|i| Iri::parse(i.to_string()).unwrap());
        Ok(Self(SimplePreparedQueryOptions::Select {
            plan,
            variables,
            evaluator: SimpleEvaluator::new(dataset, iri),
        }))
    }
}

impl<S: StoreConnection> PreparedQuery for SimplePreparedQuery<S> {
    fn exec<'a, H: ServiceHandler + 'a>(&'a self, service_handler: &'a Option<H>) -> Result<QueryResult<'_>> {
        match &self.0 {
            SimplePreparedQueryOptions::Select {
                plan,
                variables,
                evaluator,
            } => evaluator.evaluate_select_plan(&plan, &variables, service_handler),
            SimplePreparedQueryOptions::Ask { plan, variables, evaluator } => {
                evaluator.evaluate_ask_plan(&plan, &variables, service_handler)
            }
            SimplePreparedQueryOptions::Construct {
                plan,
                variables,
                construct,
                evaluator,
            } => evaluator.evaluate_construct_plan(&plan, &construct, &variables, service_handler),
            SimplePreparedQueryOptions::Describe { plan, variables, evaluator } => {
                evaluator.evaluate_describe_plan(&plan, &variables, service_handler)
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
