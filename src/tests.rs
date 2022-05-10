use std::fmt::Debug;
use crate::tests_runner::RunResult;

#[macro_export]
macro_rules! test {
    (critical, $($cb:expr)*) => {
        Box::new(Test::new(Criticality::Critical, $($cb)*))
    };
    (normal, $($cb:expr)*) => {
        Box::new(Test::new(Criticality::Normal, $($cb)*))
    };
    ($cb:expr) => {
        Box::new(Test::new(Criticality::default(), $cb))
    };
}

#[derive(Clone)]
/// A test
pub struct Test<T> {
    pub result: Option<RunResult>,
    pub level: Criticality,
    pub cb: fn(&T) -> TestResult,
}

impl<T> Test<T> {
    pub fn new(level: Criticality, cb: fn(&T) -> TestResult) -> Test<T> {
        Test {
            result: None,
            level,
            cb,
        }
    }
}

impl<T> Debug for Test<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Test {{ level: {:?}, result: {:?} }}",
            self.level, self.result
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
    Skipped,
    Aborted,
}

#[derive(Debug, Clone, Copy)]
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
}

impl<T> TestTrait<T> for Test<T> {
    fn run(&mut self, data: &T) -> RunResult {
        let result = (self.cb)(data);

        self.result = Some(RunResult::TestResult(result.clone()));
        RunResult::TestResult(result)
    }

    fn criticality(&self) -> Criticality {
        self.level
    }

    fn set_status(&mut self, issue: TestStatus) {
        self.result = Some(RunResult::TestResult(TestResult { status: issue }));
    }

    fn result(&self) -> Option<RunResult> {
        self.result.clone()
    }

    fn status(&self) -> Option<TestStatus> {
        self.result
            .as_ref()
            .map(|r| r.status())
            .flatten()
    }
    
}
