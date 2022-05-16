use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Represents the criticality of a test or a group of tests.
pub enum Criticality {
    Critical,
    Normal,
}
