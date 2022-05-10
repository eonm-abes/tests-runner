use std::fmt::Debug;

#[macro_export]
macro_rules! test {
    ($level:expr, $($cb:expr)*) => {
        Box::new(Test::new($level, $($cb)*))
    };
    ($cb:expr) => {
        Box::new(Test::new(TestLevel::default(), $cb))
    };
}

#[derive(Clone)]
/// A test
pub struct Test<T> {
    pub result: Option<TestResult>,
    pub level: TestLevel,
    pub cb: fn(&T) -> TestResult,
}

impl<T> Test<T> {
    pub fn new(level: TestLevel, cb: fn(&T) -> TestResult) -> Test<T> {
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
    pub issue: TestIssue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestIssue {
    Passed,
    Failed,
    Skipped,
    Aborted,
}

#[derive(Debug, Clone)]
pub enum TestLevel {
    // If a critical test fails, the test suite is aborted
    Critical,
    Normal,
}

impl Default for TestLevel {
    fn default() -> Self {
        TestLevel::Normal
    }
}

pub trait TestTrait<T> {
    fn run(&mut self, data: &T) -> TestResult;
    fn level(&self) -> TestLevel;
    fn set_issue(&mut self, issue: TestIssue);
    fn result(&self) -> Option<TestResult>;
}

impl<T> TestTrait<T> for Test<T> {
    fn run(&mut self, data: &T) -> TestResult {
        let result = (self.cb)(data);

        self.result = Some(result.clone());
        result
    }

    fn level(&self) -> TestLevel {
        self.level.clone()
    }

    fn set_issue(&mut self, issue: TestIssue) {
        self.result = Some(TestResult { issue });
    }

    fn result(&self) -> Option<TestResult> {
        self.result.clone()
    }
}
