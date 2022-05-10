mod tests;
mod tests_group;
mod tests_runner;

pub use crate::tests_runner::*;
pub use tests::*;
pub use tests_group::*;

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn tests_runner_should_run_tests() {
        let mut runner = TestRunner::new(&());

        let test = test!(critical, |_data| {
            TestResult {
                status: TestStatus::Passed,
            }
        });

        runner.add_test(test.clone());
        runner.add_test(test);

        let results = runner.run();

        let expected = vec![
            RunResult::TestResult(TestResult {
                status: TestStatus::Passed,
            }),
            RunResult::TestResult(TestResult {
                status: TestStatus::Passed,
            }),
        ];

        assert_eq!(results, expected);
    }

    #[test]
    fn should_abort_on_critical_test_failure() {
        let mut runner = TestRunner::new(&());

        let test_1 = test! {
            critical,
            |_data| TestResult {
                status: TestStatus::Failed,
            }
        };

        let test_2 = test! {
            critical,
            |_data| TestResult {
                status: TestStatus::Passed,
            }
        };

        runner.add_test(test_1);
        runner.add_test(test_2);

        let results = runner.run();

        let expected = vec![
            RunResult::TestResult(TestResult {
                status: TestStatus::Failed,
            }),
            RunResult::TestResult(TestResult {
                status: TestStatus::Aborted,
            })
        ];
            

        assert_eq!(results, expected);
    }

    #[test]
    fn should_run_tests_suite() {
        let suite = test_group! {
            Criticality::Critical,
            |_data| TestResult {
                status: TestStatus::Passed,
            },
            |_data| TestResult {
                status: TestStatus::Passed,
            }
        };

        let mut tests_runner = TestRunner::new(&());

        tests_runner.add_test(suite.clone());
        tests_runner.add_test(suite);

        let results = tests_runner.run();

        let expected = vec![
            RunResult::GroupResult(vec![
                TestResult {
                    status: TestStatus::Passed,
                },
                TestResult {
                    status: TestStatus::Passed,
                },
            ]),
            RunResult::GroupResult(vec![
                TestResult {
                    status: TestStatus::Passed,
                },
                TestResult {
                    status: TestStatus::Passed,
                },
            ]),
        ];

        assert_eq!(results, expected);
    }

    #[test]
    fn should_abort_on_critical_test_failure_in_suite() {
        let suite = test_group! {
            Criticality::Critical,
            |_data| TestResult {
                status: TestStatus::Failed,
            },
            |_data| TestResult {
                status: TestStatus::Passed,
            }
        };

        let mut tests_runner = TestRunner::new(&());

        tests_runner.add_test(suite.clone());
        tests_runner.add_test(suite);

        let results = tests_runner.run();

        let expected_results = vec![
            RunResult::GroupResult(vec![
                TestResult {
                    status: TestStatus::Failed,
                },
                TestResult {
                    status: TestStatus::Aborted,
                },
            ]),
            RunResult::GroupResult(vec![
                TestResult {
                    status: TestStatus::Aborted,
                },
                TestResult {
                    status: TestStatus::Aborted,
                },
            ]),
        ];

        assert_eq!(results, expected_results);
    }
}
