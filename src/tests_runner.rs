use crate::tests::{Criticality, TestResult, TestStatus, TestTrait};

/// Peut traiter un test ou une suite de tests. S'applique à un type T
pub struct TestRunner<'a, T> {
    tests: Vec<Box<dyn TestTrait<T>>>,
    data: &'a T,
}

impl<'a, T> TestRunner<'a, T> {
    pub fn new(data: &T) -> TestRunner<T> {
        TestRunner {
            tests: Vec::new(),
            data,
        }
    }

    pub fn add_test(&mut self, test: Box<dyn TestTrait<T>>) {
        self.tests.push(test);
    }

    pub fn run(&mut self) -> Vec<RunResult> {
        let mut abort = false;

        for test in &mut self.tests {
            if !abort {
                let result = test.run(&self.data);

                if let (Some(TestStatus::Failed) | Some(TestStatus::Aborted), Criticality::Critical) =
                    (result.status(), test.criticality())
                {
                    abort = true;
                }
            } else {
                test.set_status(TestStatus::Aborted);
            }
        }

        self.tests.iter().flat_map(|test| test.result()).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunResult {
    TestResult(TestResult),
    GroupResult(Vec<TestResult>),
}

impl RunResult {
    pub fn status(&self) -> Option<TestStatus> {
        match self {
            RunResult::TestResult(result) => Some(result.status.clone()),
            RunResult::GroupResult(results) => {
                if results.iter().all(|r| r.status == TestStatus::Passed) {
                    Some(TestStatus::Passed)
                } else if results.iter().all(|r| r.status == TestStatus::Failed) {
                    Some(TestStatus::Failed)
                } else if results.iter().any(|r| r.status == TestStatus::Aborted) {
                    Some(TestStatus::Aborted)
                } else {
                    None
                }
            }
        }
    }
}
