use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]

/// The status of a test or a group of tests.
pub enum TestStatus {
    /// The test has not run yey.
    Init,
    /// The test is running.
    Running,
    /// The test has passed.
    Passed,
    /// The test has failed.
    Failed,
    /// The data are unsuficient to run the test.
    NotApplicable,
    /// The test has been aborted by the tests runner.
    Aborted,
}
