use crate::{test, RunResult, TestResult};
use crate::{
    tests::{Criticality, Test, TestStatus, TestTrait},
    tests_runner,
};

#[macro_export]
macro_rules! test_group {
    ($criticality:expr, $($test:expr),*) => {
        {
            let mut group = TestGroup::new($criticality);

            $(
                group.add_test(*test!($test));
            )*

            Box::new(group)
        }
    };

    ($($test:expr)*) => {
        test_group!(Testcriticality::default() => $($test)*)
    };
}

#[derive(Debug, Clone)]
/// A group of tests
pub struct TestGroup<T> {
    tests: Vec<Test<T>>,
    criticality: Criticality,
    result: Option<RunResult>,
}

impl<T> TestGroup<T> {
    pub fn new(criticality: Criticality) -> TestGroup<T> {
        TestGroup {
            tests: Vec::new(),
            criticality,
            result: None,
        }
    }

    pub fn add_test(&mut self, test: Test<T>) {
        self.tests.push(test);
    }
}

impl<T: Clone + 'static> TestTrait<T> for TestGroup<T> {
    fn run(&mut self, data: &T) -> RunResult {
        let mut tests_runner = tests_runner::TestRunner::new(data);

        for test in &self.tests {
            let test = test.clone();
            tests_runner.add_test(Box::new(test));
        }

        let runner_results = tests_runner.run();

        let mut results = Vec::new();

        for runner_result in runner_results {
            match runner_result {
                RunResult::TestResult(result) => &results.push(result),
                RunResult::GroupResult(result) => &results.extend(result),
            };
        }

        self.result = Some(RunResult::GroupResult(results.clone()));
        return RunResult::GroupResult(results);
    }

    fn criticality(&self) -> Criticality {
        self.criticality.clone()
    }

    fn set_status(&mut self, issue: TestStatus) {
        for test in &mut self.tests {
            test.set_status(issue.clone());
        }
    }

    fn result(&self) -> Option<RunResult> {
        self.result.clone()
    }

    fn status(&self) -> Option<TestStatus> {
        self.result.as_ref().map(|r| r.status()).flatten()
    }
}
