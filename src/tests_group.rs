use crate::{
    tests::{Criticality, Status, Test, TestTrait},
    tests_runner,
};
use crate::{RunResult, TestResult};

#[macro_export]
macro_rules! test_group {
    (normal: $($test:expr),*) => {
        test_group!($crate::Criticality::Normal => $($test),*)
    };

    (critical: $($test:expr),*) => {
        test_group!($crate::Criticality::Critical => $($test),*)
    };

    ($criticality:expr => $($test:expr),*) => {
        {
            let mut group = $crate::TestGroup::new($criticality);

            $(
                group.add_test(*$test);
            )*

            Box::new(group)
        }
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

#[async_trait::async_trait]
impl<'a, T: Clone> TestTrait<'a, T> for TestGroup<T>
where
    T: Sync + Send,
    Self: Send + 'a,
{
    async fn run(&mut self, data: &mut T) -> RunResult {
        let mut tests_runner = tests_runner::TestRunner::new(data);

        for test in &self.tests {
            let test = test.clone();
            tests_runner.add_test(Box::new(test));
        }

        let runner_results = tests_runner.run().await;

        let mut results = Vec::new();

        for runner_result in runner_results {
            match runner_result {
                RunResult::TestResult(result) => &results.push(result),
                RunResult::GroupResult(result) => &results.extend(result),
            };
        }

        self.result = Some(RunResult::GroupResult(results.clone()));
        RunResult::GroupResult(results)
    }

    fn criticality(&self) -> Criticality {
        self.criticality
    }

    fn set_status(&mut self, status: Status) {
        self.result = Some(RunResult::GroupResult(
            self.tests
                .clone()
                .into_iter()
                .map(|_t| TestResult {
                    status: status.clone(),
                })
                .collect(),
        ))
    }

    fn result(&self) -> Option<RunResult> {
        self.result.clone()
    }

    fn status(&self) -> Option<Status> {
        self.result.as_ref().and_then(|r| r.status())
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::test;

    #[tokio::test]
    async fn test_single_group_run() {
        let group = test_group!(
            normal:
                test!(
                    "test_name"[critical]:
                    |_data| {
                        TestResult {
                            status: Status::Passed,
                        }
                    }
                ),
            test!(
                "test_name"[critical]:
                |_data| {
                    TestResult {
                        status: Status::Passed,
                    }
                }
            )
        );

        let mut data = ();
        let mut tests_runner = tests_runner::TestRunner::new(&mut data);
        tests_runner.add_test(group);
        let result = tests_runner.run().await;

        let expected = RunResult::GroupResult(vec![
            TestResult {
                status: Status::Passed,
            },
            TestResult {
                status: Status::Passed,
            },
        ]);

        assert_eq!(result, vec![expected]);
    }

    #[tokio::test]
    async fn test_multiple_groups_run() {
        let group_1 = test_group!(
            normal:
                test!(
                    "test_name"[critical]:
                    |_data| {
                        TestResult {
                            status: Status::Passed,
                        }
                    }
                ),
            test!(
                "test_name"[critical]:
                |_data| {
                    TestResult {
                        status: Status::Passed,
                    }
                }
            )
        );

        let group_2 = group_1.clone();

        let mut data = ();
        let mut tests_runner = tests_runner::TestRunner::new(&mut data);

        tests_runner.add_test(group_1);
        tests_runner.add_test(group_2);

        let result = tests_runner.run().await;

        let expected_1 = RunResult::GroupResult(vec![
            TestResult {
                status: Status::Passed,
            },
            TestResult {
                status: Status::Passed,
            },
        ]);

        let expected_2 = RunResult::GroupResult(vec![
            TestResult {
                status: Status::Passed,
            },
            TestResult {
                status: Status::Passed,
            },
        ]);

        assert_eq!(result, vec![expected_1, expected_2]);
    }

    #[tokio::test]
    async fn test_group_with_failed_critical_test() {
        let group = test_group!(
            critical:
                test!(
                    "test_name"[critical]:
                    |_data| {
                        TestResult {
                            status: Status::Failed,
                        }
                    }
                ),
            test!(
                "test_name"[critical]:
                |_data| {
                    TestResult {
                        status: Status::Passed,
                    }
                }
            )
        );

        let mut data = ();
        let mut tests_runner = tests_runner::TestRunner::new(&mut data);

        tests_runner.add_test(group);

        let result = tests_runner.run().await;

        let expected = RunResult::GroupResult(vec![
            TestResult {
                status: Status::Failed,
            },
            TestResult {
                status: Status::Aborted,
            },
        ]);

        assert_eq!(result, vec![expected]);
    }

    #[tokio::test]
    async fn normal_multigroups_with_failed_critical_test() {
        let group_1 = test_group!(
            normal:
                test!(
                    "test_name"[critical]:
                    |_data| {
                        TestResult {
                            status: Status::Failed,
                        }
                    }
                ),
            test!(
                "test_name"[critical]:
                |_data| {
                    TestResult {
                        status: Status::Passed,
                    }
                }
            )
        );

        let group_2 = group_1.clone();

        let mut data = ();
        let mut tests_runner = tests_runner::TestRunner::new(&mut data);

        tests_runner.add_test(group_1);
        tests_runner.add_test(group_2);

        let result = tests_runner.run().await;

        let expected_1 = RunResult::GroupResult(vec![
            TestResult {
                status: Status::Failed,
            },
            TestResult {
                status: Status::Aborted,
            },
        ]);

        // The second group runs because the first group is normal
        let expected_2 = RunResult::GroupResult(vec![
            TestResult {
                status: Status::Failed,
            },
            TestResult {
                status: Status::Aborted,
            },
        ]);

        assert_eq!(result, vec![expected_1, expected_2]);
    }

    #[tokio::test]
    async fn critical_multigroups_with_failed_critical_test() {
        let group_1 = test_group!(
            critical:
                test!(
                    "test_name"[critical]:
                    |_data| {
                        TestResult {
                            status: Status::Failed,
                        }
                    }
                ),
            test!(
                "test_name"[critical]:
                |_data| {
                    TestResult {
                        status: Status::Passed,
                    }
                }
            )
        );

        let group_2 = group_1.clone();

        let mut data = ();
        let mut tests_runner = tests_runner::TestRunner::new(&mut data);

        tests_runner.add_test(group_1);
        tests_runner.add_test(group_2);

        let result = tests_runner.run().await;

        let expected_1 = RunResult::GroupResult(vec![
            TestResult {
                status: Status::Failed,
            },
            TestResult {
                status: Status::Aborted,
            },
        ]);

        // The second group is not run because the first group is critical and failed
        let expected_2 = RunResult::GroupResult(vec![
            TestResult {
                status: Status::Aborted,
            },
            TestResult {
                status: Status::Aborted,
            },
        ]);

        assert_eq!(result, vec![expected_1, expected_2]);
    }
}
