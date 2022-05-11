use crate::{
    tests::{Criticality, Test, TestStatus, TestTrait},
    tests_runner,
};
use crate::{RunResult, TestResult};

#[macro_export]
macro_rules! test_group {
    (normal: $($test:expr),*) => {
        test_group!(crate::tests::Criticality::Normal => $($test),*)
    };

    (critical: $($test:expr),*) => {
        test_group!(crate::tests::Criticality::Critical => $($test),*);
    };

    ($criticality:expr => $($test:expr),*) => {
        {
            let mut group = crate::tests_group::TestGroup::new($criticality);

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
        RunResult::GroupResult(results)
    }

    fn criticality(&self) -> Criticality {
        self.criticality
    }

    fn set_status(&mut self, status: TestStatus) {
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

    fn status(&self) -> Option<TestStatus> {
        self.result.as_ref().and_then(|r| r.status())
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::test;

    #[test]
    fn test_single_group_run() {
        let group = test_group!(
            normal:
                test!(
                    critical:
                    |_data| {
                        TestResult {
                            status: TestStatus::Passed,
                        }
                    }
                ),
            test!(
                critical:
                |_data| {
                    TestResult {
                        status: TestStatus::Passed,
                    }
                }
            )
        );

        let mut tests_runner = tests_runner::TestRunner::new(&());
        tests_runner.add_test(group);

        let result = tests_runner.run();

        let expected = RunResult::GroupResult(vec![
            TestResult {
                status: TestStatus::Passed,
            },
            TestResult {
                status: TestStatus::Passed,
            },
        ]);

        assert_eq!(result, vec![expected]);
    }

    #[test]
    fn test_multiple_groups_run() {
        let group_1 = test_group!(
            normal:
                test!(
                    critical:
                    |_data| {
                        TestResult {
                            status: TestStatus::Passed,
                        }
                    }
                ),
            test!(
                critical:
                |_data| {
                    TestResult {
                        status: TestStatus::Passed,
                    }
                }
            )
        );

        let group_2 = group_1.clone();

        let mut tests_runner = tests_runner::TestRunner::new(&());
        tests_runner.add_test(group_1);
        tests_runner.add_test(group_2);

        let result = tests_runner.run();

        let expected_1 = RunResult::GroupResult(vec![
            TestResult {
                status: TestStatus::Passed,
            },
            TestResult {
                status: TestStatus::Passed,
            },
        ]);

        let expected_2 = RunResult::GroupResult(vec![
            TestResult {
                status: TestStatus::Passed,
            },
            TestResult {
                status: TestStatus::Passed,
            },
        ]);

        assert_eq!(result, vec![expected_1, expected_2]);
    }

    #[test]
    fn test_group_with_failed_critical_test() {
        let group = test_group!(
            critical:
                test!(
                    critical:
                    |_data| {
                        TestResult {
                            status: TestStatus::Failed,
                        }
                    }
                ),
            test!(
                critical:
                |_data| {
                    TestResult {
                        status: TestStatus::Passed,
                    }
                }
            )
        );

        let mut tests_runner = tests_runner::TestRunner::new(&());
        tests_runner.add_test(group);

        let result = tests_runner.run();

        let expected = RunResult::GroupResult(vec![
            TestResult {
                status: TestStatus::Failed,
            },
            TestResult {
                status: TestStatus::Aborted,
            },
        ]);

        assert_eq!(result, vec![expected]);
    }

    #[test]

    fn normal_multigroups_with_failed_critical_test() {
        let group_1 = test_group!(
            normal:
                test!(
                    critical:
                    |_data| {
                        TestResult {
                            status: TestStatus::Failed,
                        }
                    }
                ),
            test!(
                critical:
                |_data| {
                    TestResult {
                        status: TestStatus::Passed,
                    }
                }
            )
        );

        let group_2 = group_1.clone();

        let mut tests_runner = tests_runner::TestRunner::new(&());

        tests_runner.add_test(group_1);
        tests_runner.add_test(group_2);

        let result = tests_runner.run();

        let expected_1 = RunResult::GroupResult(vec![
            TestResult {
                status: TestStatus::Failed,
            },
            TestResult {
                status: TestStatus::Aborted,
            },
        ]);

        // The second group runs because the first group is normal
        let expected_2 = RunResult::GroupResult(vec![
            TestResult {
                status: TestStatus::Failed,
            },
            TestResult {
                status: TestStatus::Aborted,
            },
        ]);

        assert_eq!(result, vec![expected_1, expected_2]);
    }

    #[test]
    fn critical_multigroups_with_failed_critical_test() {
        let group_1 = test_group!(
            critical:
                test!(
                    critical:
                    |_data| {
                        TestResult {
                            status: TestStatus::Failed,
                        }
                    }
                ),
            test!(
                critical:
                |_data| {
                    TestResult {
                        status: TestStatus::Passed,
                    }
                }
            )
        );

        let group_2 = group_1.clone();

        let mut tests_runner = tests_runner::TestRunner::new(&());

        tests_runner.add_test(group_1);
        tests_runner.add_test(group_2);

        let result = tests_runner.run();

        let expected_1 = RunResult::GroupResult(vec![
            TestResult {
                status: TestStatus::Failed,
            },
            TestResult {
                status: TestStatus::Aborted,
            },
        ]);

        // The second group is not run because the first group is critical and failed
        let expected_2 = RunResult::GroupResult(vec![
            TestResult {
                status: TestStatus::Aborted,
            },
            TestResult {
                status: TestStatus::Aborted,
            },
        ]);

        assert_eq!(result, vec![expected_1, expected_2]);
    }
}
