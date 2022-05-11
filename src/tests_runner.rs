use crate::tests::{TestResult, Status, TestTrait};

/// Run tests or tests groups
pub struct TestRunner<'a, T>
where T: Sync + Send,
Self: Send + 'a
{
    tests: Vec<Box<dyn TestTrait<'a, T>>>,
    data: &'a mut T,
}

impl<'a, T> TestRunner<'a, T>
where T: Sync + Send,
Self: Send + 'a
{
    pub fn new(data: &'a mut T) -> TestRunner<T> {
        TestRunner {
            tests: Vec::new(),
            data,
        }
    }

    pub fn add_test(&mut self, test: Box<dyn TestTrait<'a, T>>) {
        self.tests.push(test);
    }

    pub async fn run(&mut self) -> Vec<RunResult> {
        let mut abort = false;

        for test in &mut self.tests {
            if !abort {
                test.run(self.data).await;
                abort = test.should_abort();
            } else {
                test.set_status(Status::Aborted);
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
    pub fn status(&self) -> Option<Status> {
        match self {
            RunResult::TestResult(result) => Some(result.status.clone()),
            RunResult::GroupResult(results) => {
                if results.iter().all(|r| r.status == Status::Passed) {
                    Some(Status::Passed)
                } else if results.iter().any(|r| r.status == Status::Aborted) {
                    Some(Status::Aborted)
                } else if results.iter().any(|r| r.status == Status::Failed) {
                    Some(Status::Failed)
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
    

    #[tokio::test]
    async fn test_runner_should_run_tests() {
        let mut data = ();
        let mut runner = TestRunner::new(&mut data);

        let t = test! {
            critical:
            |_| {
                TestResult {
                    status: Status::Passed,
                }
            }
        };
        runner.add_test(t);

        let result = runner.run().await;

        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_runner_should_run_multiple_tests() {
        let mut data = ();
        let mut runner = TestRunner::new(&mut data);

        let t1 = test! {
            critical:
            |_| {
                TestResult {
                    status: Status::Passed,
                }
            }
        };

        let t2 = test! {
            critical:
            |_| {
                TestResult {
                    status: Status::Passed,
                }
            }
        };

        runner.add_test(t1);
        runner.add_test(t2);

        let result = runner.run().await;

        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_runner_should_run_group() {
        let mut data = ();
        let mut runner = TestRunner::new(&mut data);

        let group = test_group!(
            normal:
                test!(
                    critical:
                    |_| {
                        TestResult {
                            status: Status::Passed,
                        }
                    }
                ),
            test!(
                critical:
                |_| {
                    TestResult {
                        status: Status::Passed,
                    }
                }
            )
        );

        runner.add_test(group);

        let result = runner.run().await;
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_runner_should_run_multiple_groups() {
        let mut data = ();
        let mut runner = TestRunner::new(&mut data);

        let group = test_group!(
            normal:
                test!(
                    critical:
                    |_| {
                        TestResult {
                            status: Status::Passed,
                        }
                    }
                ),
            test!(
                critical:
                |_| {
                    TestResult {
                        status: Status::Passed,
                    }
                }
            )
        );

        runner.add_test(group.clone());
        runner.add_test(group);

        let result = runner.run().await;
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_should_run_mix_of_tests_and_groups() {
        let mut data = ();
        let mut runner = TestRunner::new(&mut data);

        let group = test_group!(
            normal:
                test!(
                    critical:
                    |_| {
                        TestResult {
                            status: Status::Passed,
                        }
                    }
                ),
            test!(
                critical:
                |_| {
                    TestResult {
                        status: Status::Passed,
                    }
                }
            )
        );

        let t = test! {
            critical:
            |_| {
                TestResult {
                    status: Status::Passed,
                }
            }
        };

        runner.add_test(group);
        runner.add_test(t);

        let result = runner.run().await;
        assert_eq!(result.len(), 2);
    }
}
