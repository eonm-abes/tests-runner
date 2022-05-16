use crate::fsm::{ResourceState, State, Transition};
use crate::{Criticality, TestStatus, Testing, TestsRunner, TestsRunnerStatus};

use serde::Serialize;

use chrono::Utc;

/// Defines allowed transitions between states.
macro_rules! transitions {
    ($from:ident => $to:ident) => {
        impl $crate::fsm::TransitionTo<$to> for $from {}
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct InitState;

#[derive(Debug, Clone, PartialEq)]
pub struct TestingState;

#[derive(Debug, Clone, PartialEq)]
pub struct AbortingState;

#[derive(Debug, Clone, PartialEq)]
pub struct FinishState;

impl<'a, T> ResourceState<'a> for TestsRunner<'a, T>
where
    T: Clone,
{
    type Status = TestStatus;
}

transitions!(InitState => TestingState);
impl<'a, T> State<'a, TestsRunner<'a, T>> for InitState
where
    T: Clone + Serialize,
{
    fn next(self: Box<Self>, state: &mut TestsRunner<T>) -> Transition<'a, TestsRunner<'a, T>> {
        state.start = Some(Utc::now());
        state.status = TestsRunnerStatus::Init;

        Transition::next(self, TestingState)
    }
}

transitions!(TestingState => TestingState);
transitions!(TestingState => FinishState);
transitions!(TestingState => AbortingState);
impl<'a, T> State<'a, TestsRunner<'a, T>> for TestingState
where
    T: Clone + Serialize,
{
    fn next(self: Box<Self>, state: &mut TestsRunner<T>) -> Transition<'a, TestsRunner<'a, T>> {
        state.status = TestsRunnerStatus::Testing;

        let test = state
            .tests
            .iter_mut()
            .find(|test| test.status() == &TestStatus::Init);

        if let Some(test) = test {
            let criticality = test.criticality().clone();
            let result = test.run(&state.data);

            match (criticality, result) {
                (Criticality::Critical, TestStatus::Failed | TestStatus::Aborted) => {
                    Transition::next(self, AbortingState)
                }
                _ => Transition::next(self, TestingState),
            }
        } else {
            Transition::next(self, FinishState)
        }
    }
}

impl<'a, T> State<'a, TestsRunner<'a, T>> for FinishState
where
    T: Clone,
{
    fn next(self: Box<Self>, state: &mut TestsRunner<T>) -> Transition<'a, TestsRunner<'a, T>> {
        state.end = Some(Utc::now());
        state.status = TestsRunnerStatus::Completed;
        Transition::Complete(Ok(()))
    }
}

transitions!(AbortingState => AbortingState);
impl<'a, T> State<'a, TestsRunner<'a, T>> for AbortingState
where
    T: Clone + Serialize,
{
    fn next(self: Box<Self>, state: &mut TestsRunner<T>) -> Transition<'a, TestsRunner<'a, T>> {
        state.status = TestsRunnerStatus::Aborted;

        let test = state
            .tests
            .iter_mut()
            .find(|test| test.status() == &TestStatus::Init);

        if let Some(test) = test {
            test.abort();
            Transition::next(self, AbortingState)
        } else {
            state.end = Some(Utc::now());
            Transition::Complete(Ok(()))
        }
    }
}
