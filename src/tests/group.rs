use serde::Serialize;

/// Wraps a set of tests into a Test.
#[macro_export]
macro_rules! group {
    ($name:literal[critical] $(=> $test:expr),+) => {{
        let mut group = $crate::Group::new($name.into());
        group.set_criticality($crate::Criticality::Critical);

        $(
            group.add_test($test);
        )*

        group
    }};
    ($name:literal[normal] $(=> $test:expr),+) => {{
        let mut group = $crate::Group::new($name.into());
        group.set_criticality($crate::Criticality::Normal);

        $(
            group.add_test($test);
        )*

        group
    }};
    ($name:literal $(=> $test:expr),+) => {{
        group!($name[normal] $(=> $test),+)
    }};
}

use crate::{
    tests::{Criticality, Test, TestStatus, Testing},
    tests_runner::TestsRunner,
};

#[derive(Clone, Serialize)]
/// A group of tests.
pub struct Group<T: ?Sized> {
    pub name: String,
    pub criticality: Criticality,
    pub tests: Vec<Test<T>>,
}

impl<T> Group<T> {
    pub fn new(name: String) -> Self {
        Group {
            name,
            tests: Vec::new(),
            criticality: Criticality::Normal,
        }
    }

    pub fn set_criticality(&mut self, criticality: Criticality) {
        self.criticality = criticality;
    }

    pub fn add_test<X: Into<Test<T>>>(&mut self, test: X) {
        self.tests.push(test.into());
    }
}

impl<T> Testing<T> for Group<T>
where
    T: Clone + Serialize,
{
    fn run(&mut self, input: &T) -> &TestStatus {
        let mut test_runner: TestsRunner<T> = TestsRunner::new(input);

        for test in &self.tests {
            test_runner = test_runner.add_test(test.clone());
        }

        test_runner.run();

        self.tests = test_runner.tests;

        self.status()
    }

    fn status(&self) -> &TestStatus {
        let status: Vec<TestStatus> = self
            .tests
            .iter()
            .map(|test| test.status().clone())
            .collect();

        if status.iter().any(|s| *s == TestStatus::Init) {
            &TestStatus::Init
        } else if status.iter().all(|s| *s == TestStatus::Passed) {
            &TestStatus::Passed
        } else if status.iter().any(|s| *s == TestStatus::Failed) {
            &TestStatus::Failed
        } else if status.iter().any(|s| *s == TestStatus::Aborted) {
            &TestStatus::Aborted
        } else {
            &TestStatus::Failed
        }
    }

    fn criticality(&self) -> &Criticality {
        &self.criticality
    }

    fn abort(&mut self) {
        for test in &mut self.tests {
            test.abort();
        }
    }
}

#[cfg(test)]
mod group_test {
    use crate::group;
    use crate::test;
    use crate::Criticality;

    use crate::TestStatus;

    #[test]
    fn group_macro_should_create_critical_test_group() {
        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let group = group!("test_group"[critical]
            => test!("test_1"[critical] => success),
            => test!("test_2"[critical] => success)
        );

        assert!(group.criticality == Criticality::Critical);
        assert!(group.tests.len() == 2);
    }

    #[test]
    fn group_macro_should_create_normal_test_group() {
        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let group = group!("test_group"[normal]
            => test!("test_1"[normal] => success),
            => test!("test_2"[normal] => success)
        );

        assert!(group.criticality == Criticality::Normal);
        assert!(group.tests.len() == 2);
    }

    #[test]
    fn group_macro_should_create_group_without_explicit_criticality() {
        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let group = group!("test_group"
            => test!("test_1"[normal] => success),
            => test!("test_2"[normal] => success)
        );

        assert!(group.criticality == Criticality::Normal);
        assert!(group.tests.len() == 2);
    }
}
