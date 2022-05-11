use crate::tests::{TestResult, TestStatus, TestTrait};

/// Run tests or tests groups
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
                let _result = test.run(self.data);
                abort = test.should_abort();
            } else {
                eprintln!("aborting test {:?}", test.status());

                test.set_status(TestStatus::Aborted);
                eprintln!("aborting test {:?}", test.status());
            }
        }

        eprintln!(
            "{:?}",
            self.tests
                .iter()
                .map(|test| test.result())
                .collect::<Vec<Option<RunResult>>>()
        );

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
                } else if results.iter().any(|r| r.status == TestStatus::Aborted) {
                    Some(TestStatus::Aborted)
                } else if results.iter().any(|r| r.status == TestStatus::Failed) {
                    Some(TestStatus::Failed)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]

mod tests_runner {
    use super::*;
    use crate::test;
    use crate::test_group;
    use crate::tests::Test;
    

    #[test]
    fn test_runner_should_run_tests() {
        let mut runner = TestRunner::new(&());

        let t = test! {
            critical:
            |_| {
                TestResult {
                    status: TestStatus::Passed,
                }
            }
        };
        runner.add_test(t);

        let result = runner.run();

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_runner_should_run_multiple_tests() {
        let mut runner = TestRunner::new(&());

        let t1 = test! {
            critical:
            |_| {
                TestResult {
                    status: TestStatus::Passed,
                }
            }
        };

        let t2 = test! {
            critical:
            |_| {
                TestResult {
                    status: TestStatus::Passed,
                }
            }
        };

        runner.add_test(t1);
        runner.add_test(t2);

        let result = runner.run();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_runner_should_run_group() {
        let mut runner = TestRunner::new(&());

        let group = test_group!(
            normal:
                test!(
                    critical:
                    |_| {
                        TestResult {
                            status: TestStatus::Passed,
                        }
                    }
                ),
            test!(
                critical:
                |_| {
                    TestResult {
                        status: TestStatus::Passed,
                    }
                }
            )
        );

        runner.add_test(group);

        let result = runner.run();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_runner_should_run_multiple_groups() {
        let mut runner = TestRunner::new(&());

        let group = test_group!(
            normal:
                test!(
                    critical:
                    |_| {
                        TestResult {
                            status: TestStatus::Passed,
                        }
                    }
                ),
            test!(
                critical:
                |_| {
                    TestResult {
                        status: TestStatus::Passed,
                    }
                }
            )
        );

        runner.add_test(group.clone());
        runner.add_test(group);

        let result = runner.run();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_should_run_mix_of_tests_and_groups() {
        let mut runner = TestRunner::new(&());

        let group = test_group!(
            normal:
                test!(
                    critical:
                    |_| {
                        TestResult {
                            status: TestStatus::Passed,
                        }
                    }
                ),
            test!(
                critical:
                |_| {
                    TestResult {
                        status: TestStatus::Passed,
                    }
                }
            )
        );

        let t = test! {
            critical:
            |_| {
                TestResult {
                    status: TestStatus::Passed,
                }
            }
        };

        runner.add_test(group);
        runner.add_test(t);

        let result = runner.run();
        assert_eq!(result.len(), 2);
    }
}
