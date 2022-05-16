use crate::fsm::{StateHolder, Transition};
use crate::tests::{Test, Testing};
mod states;
use states::*;

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Serialize)]
/// Runs tests and groups of tests.
pub struct TestsRunner<'a, T: ?Sized> {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    status: TestsRunnerStatus,
    pub tests: Vec<Test<T>>,
    #[serde(skip)]
    data: &'a T,
}

impl<'a, T: ?Sized> TestsRunner<'a, T>
where
    T: Clone + Serialize,
{
    /// Creates a new `TestsRunner`
    pub fn new(data: &'a T) -> Self {
        TestsRunner {
            start: None,
            end: None,
            status: TestsRunnerStatus::Init,
            tests: Vec::new(),
            data,
        }
    }

    /// Updates the data of the tests runner.
    pub fn set_data(mut self, data: &'a T) -> Self {
        self.data = data;
        self.try_update_status(TestsRunnerStatus::Init);

        self
    }

    fn try_update_status(&mut self, status: TestsRunnerStatus) {
        if self.status != TestsRunnerStatus::Aborted {
            self.status = status;
        }
    }

    /// Adds a test or a group of tests to the tests runner.
    pub fn add_test<X: Into<Test<T>>>(mut self, test: X) -> Self {
        if self.status == TestsRunnerStatus::Aborted {
            let mut test = test.into();
            test.abort();
            self.tests.push(test);
        } else {
            self.try_update_status(TestsRunnerStatus::Init);
            self.tests.push(test.into());
        }

        self
    }

    pub fn add_group<X: Into<Test<T>>>(self, group: X) -> Self {
        self.add_test(group)
    }

    /// Runs the tests.
    pub fn run(&mut self) -> TestsRunnerStatus {
        if self.status == TestsRunnerStatus::Init {
            self.end = None;

            let mut fsm = Box::new(StateHolder {
                state: Box::new(InitState),
            });

            while let Transition::Next(next) = fsm.state.next(self) {
                fsm = Box::new(next);
            }
        }

        self.status.clone()
    }

    pub fn repport(&self) -> String {
        serde_json::to_string_pretty(&self).expect("Failed to serialize tests runner data to json")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Represents the Status of a tests runner.
pub enum TestsRunnerStatus {
    /// The tests runner is in the initial state.
    Init,
    /// The tests runner is in the testing state.
    Testing,
    /// The tests runner is in the aborting state.
    Aborted,
    /// The tests runner is in the completed state.
    Completed,
}

#[cfg(test)]

mod tests_runner_tests {
    use super::*;
    use crate::group;
    use crate::test;
    use crate::tests::TestStatus;

    #[test]
    fn run_tests() {
        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let mut tr = TestsRunner::new(&()).add_test(test!("test1"[normal] => success));

        assert_eq!(tr.run(), TestsRunnerStatus::Completed);
        assert_eq!(tr.tests[0].status(), &TestStatus::Passed);
        assert_eq!(tr.status, TestsRunnerStatus::Completed);
        assert!(tr.start.is_some());
        assert!(tr.end.is_some());
    }

    #[test]

    fn test_string() {
        fn success(_data: &String) -> TestStatus {
            TestStatus::Passed
        }

        let data = "".to_string();
        let mut tr = TestsRunner::new(&data).add_test(test!("test1"[normal] => success));

        assert_eq!(tr.run(), TestsRunnerStatus::Completed);
    }

    #[test]
    fn tr_should_abort_on_critical_test_failure() {
        fn fail(_data: &()) -> TestStatus {
            TestStatus::Failed
        }

        let mut tr = TestsRunner::new(&()).add_test(test!("test1"[critical] => fail));
        tr.run();

        assert_eq!(tr.status, TestsRunnerStatus::Aborted);
        assert_eq!(tr.tests[0].status(), &TestStatus::Failed);
        assert!(tr.start.is_some());
        assert!(tr.end.is_some());
    }

    #[test]
    fn aborted_tr_should_remains_aborted() {
        fn fail(_data: &()) -> TestStatus {
            TestStatus::Failed
        }

        let mut tr = TestsRunner::new(&()).add_test(test!("test1"[critical] => fail));

        tr.run();

        assert_eq!(tr.status, TestsRunnerStatus::Aborted);

        tr = tr.add_test(test!("test1"[critical] => fail));
        tr.run();

        assert_eq!(tr.status, TestsRunnerStatus::Aborted);
    }

    #[test]
    fn adding_test_to_non_aborted_tr_shoud_change_status_to_init() {
        fn fail(_data: &()) -> TestStatus {
            TestStatus::Failed
        }

        let mut tr = TestsRunner::new(&()).add_test(test!("test1"[normal] => fail));

        tr.run();

        assert_eq!(tr.status, TestsRunnerStatus::Completed);

        tr = tr.add_test(test!("test1"[normal] => fail));
        assert_eq!(tr.status, TestsRunnerStatus::Init);

        tr.run();
        assert_eq!(tr.status, TestsRunnerStatus::Completed);
    }

    #[test]
    fn critical_test_failure_should_leads_to_tr_abortion() {
        fn fail(_data: &()) -> TestStatus {
            TestStatus::Failed
        }

        let mut tr = TestsRunner::new(&())
            .add_test(test!("test1"[critical] => fail))
            .add_test(test!("test1"[critical] => fail));

        tr.run();

        assert_eq!(tr.status, TestsRunnerStatus::Aborted);

        let tests_status = tr
            .tests
            .iter()
            .map(|test| test.status().clone())
            .collect::<Vec<TestStatus>>();

        assert_eq!(tests_status, vec![TestStatus::Failed, TestStatus::Aborted]);
    }

    #[test]
    fn non_critical_failure_should_not_leads_to_tr_abortion() {
        fn fail(_data: &()) -> TestStatus {
            TestStatus::Failed
        }

        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let mut tr = TestsRunner::new(&())
            .add_test(test!("test1"[normal] => fail))
            .add_test(test!("test1"[normal] => success));

        tr.run();

        assert_eq!(tr.status, TestsRunnerStatus::Completed);

        let tests_status = tr
            .tests
            .iter()
            .map(|test| test.status().clone())
            .collect::<Vec<TestStatus>>();

        assert_eq!(tests_status, vec![TestStatus::Failed, TestStatus::Passed]);
    }

    #[test]
    fn test_group_passing() {
        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let mut tr = TestsRunner::new(&()).add_test(group!(
            "test_group"[critical]
                => test!("test_1"[normal] => success),
                => test!("test_2"[normal] => success)
        ));

        tr.run();

        assert_eq!(tr.status, TestsRunnerStatus::Completed);

        let tests_status = tr
            .tests
            .iter()
            .map(|test| test.status().clone())
            .collect::<Vec<TestStatus>>();

        assert_eq!(tests_status, vec![TestStatus::Passed]);
    }

    #[test]
    fn test_group_failing() {
        fn fail(_data: &()) -> TestStatus {
            TestStatus::Failed
        }

        let mut tr = TestsRunner::new(&()).add_test(group!(
            "test_group"[normal]
                => test!("test_1"[normal] => fail),
                => test!("test_2"[normal] => fail)
        ));

        tr.run();

        assert_eq!(tr.status, TestsRunnerStatus::Completed);

        let tests_status = tr
            .tests
            .iter()
            .map(|test| test.status().clone())
            .collect::<Vec<TestStatus>>();

        assert_eq!(tests_status, vec![TestStatus::Failed]);
    }

    #[test]
    fn test_group_aborted() {
        fn fail(_data: &()) -> TestStatus {
            TestStatus::Failed
        }

        let mut tr = TestsRunner::new(&()).add_test(group!(
            "test_group"[critical]
                => test!("test_1"[normal] => fail),
                => test!("test_2"[normal] => fail)
        ));

        tr.run();

        assert_eq!(tr.status, TestsRunnerStatus::Aborted);

        let tests_status = tr
            .tests
            .iter()
            .map(|test| test.status().clone())
            .collect::<Vec<TestStatus>>();

        assert_eq!(tests_status, vec![TestStatus::Failed]);
    }

    #[test]

    fn critical_group_failure_abortion() {
        fn fail(_data: &()) -> TestStatus {
            TestStatus::Failed
        }

        let mut tr = TestsRunner::new(&())
            .add_test(test!("test"[normal] => fail))
            .add_test(group!(
                "group"[critical]
                    => test!("test_1"[normal] => fail),
                    => test!("test_2"[normal] => fail)
            ))
            .add_test(test!("test"[normal] => fail));
        tr.run();

        assert_eq!(tr.status, TestsRunnerStatus::Aborted);

        let tests_status = tr
            .tests
            .iter()
            .map(|test| test.status().clone())
            .collect::<Vec<TestStatus>>();

        assert_eq!(
            tests_status,
            vec![TestStatus::Failed, TestStatus::Failed, TestStatus::Aborted]
        );
    }
}
