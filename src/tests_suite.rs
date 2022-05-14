use crate::tests::{Test, TestIssue, TestLevel, TestResult, TestTrait};

#[macro_export]
macro_rules! test_suite {
    ($level:expr, $($test:expr),*) => {
        {
            let mut suite = TestSuite::new($level);

            $(w
                suite.add_test(*test!($test));
            )*

            Box::new(suite)
        }
    };

    ($($test:expr)*) => {
        test_suite!(TestLevel::default() => $($test)*)
    };
}

#[derive(Debug, Clone)]
pub struct TestSuite<T> {
    tests: Vec<Test<T>>,
    level: TestLevel,
    result: Option<TestResult>,
}

impl<T> TestSuite<T> {
    pub fn new(level: TestLevel) -> TestSuite<T> {
        TestSuite {
            tests: Vec::new(),
            level,
            result: None,
        }
    }

    pub fn add_test(&mut self, test: Test<T>) {
        self.tests.push(test);
    }
}

impl<T> TestTrait<T> for TestSuite<T> {
    fn run(&mut self, data: &T) -> TestResult {
        for test in &mut self.tests {
            let test_result = test.run(data);

            match (&test_result.issue, &self.level) {
                (TestIssue::Failed, TestLevel::Critical) => {
                    self.result = Some(test_result.clone());
                    return test_result;
                }
                (TestIssue::Failed | TestIssue::Skipped | TestIssue::Aborted, _) => {
                    self.result = Some(test_result.clone());
                    return test_result;
                }
                _ => {}
            }
        }

        TestResult {
            issue: TestIssue::Passed,
        }
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
