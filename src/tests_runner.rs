use crate::tests::{TestIssue, TestLevel, TestResult, TestTrait};

/// Peut traiter un test ou une suite de tests. S'applique à un type T
pub struct TestRunner<T> {
    tests: Vec<Box<dyn TestTrait<T>>>,
    data: T,
}

impl<T> TestRunner<T> {
    pub fn new(data: T) -> TestRunner<T> {
        TestRunner {
            tests: Vec::new(),
            data,
        }
    }

    pub fn add_test(&mut self, test: Box<dyn TestTrait<T>>) {
        self.tests.push(test);
    }

    pub fn run(&mut self) -> Vec<Option<TestResult>> {
        let mut abort = false;

        for test in &mut self.tests {
            if !abort {
                let result = test.run(&self.data);

                if let (TestIssue::Failed, TestLevel::Critical) = (result.issue, test.level()) {
                    abort = true;
                }
            } else {
                test.set_issue(TestIssue::Aborted);
            }
        }

        self.tests.iter().map(|test| test.result()).collect()
    }
}
