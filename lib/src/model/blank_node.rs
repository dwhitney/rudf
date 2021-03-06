use rand::random;
use rio_api::model as rio;
use std::fmt;
use std::io::Write;
use std::str;

/// A RDF [blank node](https://www.w3.org/TR/rdf11-concepts/#dfn-blank-node).
///
/// This implementation enforces that the blank node id is a uniquely generated ID to easily ensure
/// that it is not possible for two blank nodes to share an id.
///
/// The common way to create a new blank node is to use the `Default::default` trait method.
///
/// The default string formatter is returning a N-Triples, Turtle and SPARQL compatible representation.
/// `BlankNode::default().to_string()` should return something like `_:00112233445566778899aabbccddeeff`
///
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Hash)]
pub struct BlankNode {
    id: u128,
    str: [u8; 32],
}

impl BlankNode {
    /// Creates a blank node from a unique id
    pub(crate) fn new_from_unique_id(id: u128) -> Self {
        let mut str = [0; 32];
        write!(&mut str[..], "{:x}", id).unwrap();
        Self { id, str }
    }

    /// Returns the underlying ID of this blank node
    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.str).unwrap()
    }

    /// Returns the underlying ID of this blank node
    pub(crate) fn id(&self) -> u128 {
        self.id
    }
}

impl fmt::Display for BlankNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        rio::BlankNode::from(self).fmt(f)
    }
}

impl Default for BlankNode {
    /// Builds a new RDF [blank node](https://www.w3.org/TR/rdf11-concepts/#dfn-blank-node) with a unique id
    fn default() -> Self {
        Self::new_from_unique_id(random::<u128>())
    }
}

impl<'a> From<&'a BlankNode> for rio::BlankNode<'a> {
    fn from(node: &'a BlankNode) -> Self {
        rio::BlankNode { id: node.as_str() }
    }
}
