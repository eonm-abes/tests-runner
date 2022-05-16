use crate::tests::{Criticality, TestStatus, Testing};

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Wraps a single test into a Test.
#[macro_export]
macro_rules! test {
    ($name:literal[critical] => $func:expr) => {{
        let mut test = $crate::SingleTest::new($name.into(), $func);
        test.set_criticality($crate::Criticality::Critical);
        test
    }};

    ($name:literal[normal] => $func:expr) => {
        $crate::SingleTest::new($name.into(), $func)
    };
    ($name:literal => $func:expr) => {
        test!($name[normal] => $func)
    };
}

/// A single test.
#[derive(Clone, Serialize)]
pub struct SingleTest<T: ?Sized> {
    name: String,
    criticality: Criticality,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    status: TestStatus,
    #[serde(skip)]
    cb: fn(&T) -> TestStatus,
}

impl<T> SingleTest<T> {
    pub fn new(name: String, cb: fn(&T) -> TestStatus) -> Self {
        SingleTest {
            criticality: Criticality::Normal,
            start: None,
            end: None,
            name,
            status: TestStatus::Init,
            cb,
        }
    }

    pub fn set_criticality(&mut self, criticality: Criticality) {
        self.criticality = criticality;
    }
}

impl<T> Testing<T> for SingleTest<T>
where
    T: Clone,
{
    fn run(&mut self, input: &T) -> &TestStatus {
        self.start = Some(Utc::now());
        self.status = TestStatus::Running;
        self.status = (self.cb)(input);
        self.end = Some(Utc::now());
        &self.status
    }

    fn abort(&mut self) {
        self.status = TestStatus::Aborted;
    }

    fn criticality(&self) -> &Criticality {
        &self.criticality
    }

    fn status(&self) -> &TestStatus {
        &self.status
    }
}

impl<T> std::fmt::Debug for SingleTest<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Test")
            .field("name", &self.name)
            .field("status", &self.status)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("criticality", &self.criticality)
            .finish()
    }
}

#[cfg(test)]
mod single_tests {
    use super::*;
    use crate::test;
    use crate::TestStatus;

    #[test]
    fn test_macro_should_create_critical_test() {
        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let test = test!("test"[critical] => success);

        assert_eq!(test.criticality(), &Criticality::Critical);
    }

    #[test]
    fn test_macro_should_create_normal_test() {
        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let test = test!("test"[normal] => success);

        assert_eq!(test.criticality(), &Criticality::Normal);
    }

    #[test]
    fn test_macro_should_create_test_without_explicit_criticality() {
        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let test = test!("test" => success);

        assert_eq!(test.criticality(), &Criticality::Normal);
    }

    #[test]
    fn set_test_criticality() {
        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let mut test = SingleTest::new("test".into(), success);
        test.set_criticality(Criticality::Critical);
        assert_eq!(test.criticality(), &Criticality::Critical);
    }

    #[test]
    fn run_test() {
        fn success(_data: &()) -> TestStatus {
            TestStatus::Passed
        }

        let mut test = test!("test"[normal] => success);
        test.run(&());

        assert_eq!(test.status(), &TestStatus::Passed);

        fn failure(_data: &()) -> TestStatus {
            TestStatus::Failed
        }

        let mut test = test!("test"[normal] => failure);
        test.run(&());

        assert_eq!(test.status(), &TestStatus::Failed);
    }
}
