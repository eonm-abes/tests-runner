pub mod criticality;
pub mod group;
pub mod single;
pub mod status;

pub use criticality::Criticality;
pub use group::Group;
pub use single::SingleTest;
pub use status::TestStatus;

use serde::Serialize;

/// An interface for testing.
pub trait Testing<T> {
    /// Runs a test.
    fn run(&mut self, input: &T) -> &TestStatus;
    /// Returns the status of a test.
    fn status(&self) -> &TestStatus;
    /// Returns the criticality of a test.
    fn criticality(&self) -> &Criticality;
    /// Aborts the test.
    fn abort(&mut self);
}

#[derive(Clone, Serialize)]
/// A test or a group of tests.
pub enum Test<T: ?Sized> {
    Single(SingleTest<T>),
    Group(Group<T>),
}

impl<T> Testing<T> for Test<T>
where
    T: Clone + Serialize,
{
    fn run(&mut self, input: &T) -> &TestStatus {
        match self {
            Test::Single(test) => test.run(input),
            Test::Group(test_group) => test_group.run(input),
        }
    }

    fn status(&self) -> &TestStatus {
        match self {
            Test::Single(test) => test.status(),
            Test::Group(test_group) => test_group.status(),
        }
    }

    fn criticality(&self) -> &Criticality {
        match self {
            Test::Single(test) => test.criticality(),
            Test::Group(test_group) => test_group.criticality(),
        }
    }

    fn abort(&mut self) {
        match self {
            Test::Single(test) => test.abort(),
            Test::Group(test_group) => test_group.abort(),
        }
    }
}

impl<T> From<SingleTest<T>> for Test<T> {
    fn from(test: SingleTest<T>) -> Self {
        Test::Single(test)
    }
}

impl<T> From<Group<T>> for Test<T> {
    fn from(test_group: Group<T>) -> Self {
        Test::Group(test_group)
    }
}
