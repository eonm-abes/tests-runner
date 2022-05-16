/// Defines allowed transitions between states.
pub trait TransitionTo<S> {}

impl<'a, S: ResourceState<'a>> Transition<'a, S> {
    #[allow(clippy::boxed_local)]
    pub fn next<I: State<'a, S>, O: 'static + State<'a, S>>(_i: Box<I>, o: O) -> Transition<'a, S>
    where
        I: TransitionTo<O>,
    {
        Transition::Next(StateHolder { state: Box::new(o) })
    }
}

pub trait ResourceState<'a> {
    type Status;
}

pub trait State<'a, S: ResourceState<'a>> {
    fn next(self: Box<Self>, state: &mut S) -> Transition<'a, S>;
}

pub enum Transition<'a, S: ResourceState<'a>> {
    /// Transition to new state.
    Next(StateHolder<'a, S>),
    /// Stop executing the state machine and report the result of the execution.
    Complete(Result<(), Box<dyn std::error::Error>>),
}

pub struct StateHolder<'a, S: ResourceState<'a>> {
    pub state: Box<dyn State<'a, S>>,
}
