use crate::tests_runner::RunResult;
use std::fmt::Debug;

#[macro_export]
macro_rules! test {
    ($name:literal[critical]: $($cb:expr)*) => {
        Box::new(Test::new($name,$crate::Criticality::Critical, $($cb)*))
    };
    ($name:literal[normal]: $($cb:expr)*) => {
        Box::new(Test::new($name,$crate::Criticality::Normal, $($cb)*))
    };
    ($name:literal, $($cb:expr)*) => {
        Box::new(Test::new($name, $crate::Criticality::Normal, $($cb)*))
    };
}

#[derive(Clone)]
/// A test
pub struct Test<T> {
    pub result: Option<RunResult>,
    pub name: String,
    pub criticality: Criticality,
    pub cb: fn(&mut T) -> Status,
}

impl<T> Test<T> {
    pub fn new<S: Into<String>>(
        name: S,
        criticality: Criticality,
        cb: fn(&mut T) -> Status,
    ) -> Test<T> {
        Test {
            name: name.into(),
            result: None,
            criticality,
            cb,
        }
    }
}

impl<T> Debug for Test<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Test {{ name: {}, criticality: {:?}, result: {:?} }}",
            self.name, self.criticality, self.result
        )
    }
}

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
/// A test result
pub struct TestResult {
    pub criticality: Criticality,
    pub name: String,
    pub status: Status,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Represents the status of a test or a group of tests
pub enum Status {
    Passed,
    Failed,
    Aborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
/// Represents the Criticality of a test or a group of tests
pub enum Criticality {
    // If a critical test fails, the test suite is aborted
    Critical,
    Normal,
}

impl Default for Criticality {
    fn default() -> Self {
        Criticality::Normal
    }
}

#[async_trait::async_trait]
pub trait TestTrait<'a, T>
where
    T: Sync + Send,
    Self: Send + 'a,
{
    async fn run(&mut self, data: &mut T) -> RunResult;
    fn criticality(&self) -> Criticality;
    fn set_status(&mut self, issue: Status);
    fn status(&self) -> Option<Status>;
    fn result(&self) -> Option<RunResult>;
    fn should_abort(&self) -> bool {
        self.criticality() == Criticality::Critical
            && (self.status() == Some(Status::Failed) || self.status() == Some(Status::Aborted))
    }
}

#[async_trait::async_trait]
impl<'a, T> TestTrait<'a, T> for Test<T>
where
    T: Sync + Send,
    Self: Send + 'a,
{
    async fn run(&mut self, data: &mut T) -> RunResult {
        let status = (self.cb)(data);

        let test_result = RunResult::TestResult(TestResult {
            criticality: self.criticality,
            name: self.name.clone(),
            status,
        });

        self.result = Some(test_result.clone());
        test_result
    }

    /// Returns the Criticality of the test
    fn criticality(&self) -> Criticality {
        self.criticality
    }

    /// Sets the status of the test
    fn set_status(&mut self, status: Status) {
        self.result = Some(RunResult::TestResult(TestResult {
            criticality: self.criticality,
            name: self.name.clone(),
            status,
        }));
    }

    /// Gets the result of the test
    fn result(&self) -> Option<RunResult> {
        self.result.clone()
    }

    /// Gets the status of the test
    fn status(&self) -> Option<Status> {
        self.result.as_ref().and_then(|r| r.status())
    }
}
