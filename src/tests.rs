use crate::tests_runner::RunResult;
use std::fmt::Debug;

#[macro_export]
macro_rules! test {
    (critical: $($cb:expr)*) => {
        Box::new(Test::new(crate::tests::Criticality::Critical, $($cb)*))
    };
    (normal: $($cb:expr)*) => {
        Box::new(Test::new(crate::tests::Criticality::Normal, $($cb)*))
    };
    ($($cb:expr)*) => {
        Box::new(Test::new(crate::tests::Criticality::Normal, $($cb)*))
    };
}

#[derive(Clone)]
/// A test
pub struct Test<T> {
    pub result: Option<RunResult>,
    pub criticality: Criticality,
    pub cb: fn(&T) -> TestResult,
}

impl<T> Test<T> {
    pub fn new(criticality: Criticality, cb: fn(&T) -> TestResult) -> Test<T> {
        Test {
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
            "Test {{ criticality: {:?}, result: {:?} }}",
            self.criticality, self.result
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestResult {
    pub status: TestStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Aborted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

pub trait TestTrait<T> {
    fn run(&mut self, data: &T) -> RunResult;
    fn criticality(&self) -> Criticality;
    fn set_status(&mut self, issue: TestStatus);
    fn status(&self) -> Option<TestStatus>;
    fn result(&self) -> Option<RunResult>;
    fn should_abort(&self) -> bool {
        self.criticality() == Criticality::Critical
            && (self.status() == Some(TestStatus::Failed)
                || self.status() == Some(TestStatus::Aborted))
    }
}

impl<T> TestTrait<T> for Test<T> {
    fn run(&mut self, data: &T) -> RunResult {
        let result = (self.cb)(data);

        self.result = Some(RunResult::TestResult(result.clone()));
        RunResult::TestResult(result)
    }

    /// Returns the Criticality of the test
    fn criticality(&self) -> Criticality {
        self.criticality
    }

    /// Sets the status of the test
    fn set_status(&mut self, status: TestStatus) {
        self.result = Some(RunResult::TestResult(TestResult { status }));
    }

    /// Gets the result of the test
    fn result(&self) -> Option<RunResult> {
        self.result.clone()
    }

    /// Gets the status of the test
    fn status(&self) -> Option<TestStatus> {
        self.result.as_ref().and_then(|r| r.status())
    }
}
