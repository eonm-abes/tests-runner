mod tests;
mod tests_runner;
mod tests_suite;

pub use crate::tests_runner::*;
pub use tests::*;
pub use tests_suite::*;

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn tests_runner_should_run_tests() {
        let mut runner = TestRunner::new(());

        let test = test!(TestLevel::Critical, |_data| {
            TestResult {
                issue: TestIssue::Passed,
            }
        });

        runner.add_test(test.clone());
        runner.add_test(test);

        let results = runner.run();

        let expected = vec![
            Some(TestResult {
                issue: TestIssue::Passed,
            }),
            Some(TestResult {
                issue: TestIssue::Passed,
            }),
        ];

        assert_eq!(results, expected);
    }

    #[test]
    fn should_abort_on_critical_test_failure() {
        let mut runner = TestRunner::new(());

        let test_1 = test! {
            TestLevel::Critical,
            |_data| TestResult {
                issue: TestIssue::Failed,
            }
        };

        let test_2 = test! {
            TestLevel::Normal,
            |_data| TestResult {
                issue: TestIssue::Passed,
            }
        };

        runner.add_test(test_1);
        runner.add_test(test_2);

        let results = runner.run();

        let expected_results = vec![
            Some(TestResult {
                issue: TestIssue::Failed,
            }),
            Some(TestResult {
                issue: TestIssue::Aborted,
            }),
        ];

        assert_eq!(results, expected_results);
    }

    #[test]
    fn should_run_tests_suite() {
        let suite = test_suite! {
            TestLevel::Critical,
            |_data| TestResult {
                issue: TestIssue::Passed,
            },
            |_data| TestResult {
                issue: TestIssue::Passed,
            }
        };

        let mut tests_runner = TestRunner::new(());

        tests_runner.add_test(suite.clone());
        tests_runner.add_test(suite);

        let results = tests_runner.run();

        let expected_results = vec![
            Some(TestResult {
                issue: TestIssue::Passed,
            }),
            Some(TestResult {
                issue: TestIssue::Passed,
            }),
        ];

        assert_eq!(results, expected_results);
    }

    #[test]
    fn should_abort_on_critical_test_failure_in_suite() {
        let suite = test_suite! {
            TestLevel::Critical,
            |_data| TestResult {
                issue: TestIssue::Failed,
            },
            |_data| TestResult {
                issue: TestIssue::Passed,
            }
        };

        let mut tests_runner = TestRunner::new(());

        tests_runner.add_test(suite);
        //tests_runner.add_test(Box::new(suite));

        let results = tests_runner.run();

        let expected_results = vec![
            Some(TestResult {
                issue: TestIssue::Failed,
            }),
            Some(TestResult {
                issue: TestIssue::Aborted,
            }),
        ];

        assert_eq!(results, expected_results);
    }
}
