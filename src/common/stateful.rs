//! Trying to apply some inspirations from the state monad pattern,
//! without fully understanding it, in Rust.
//!
//! The implementation is a mix between the Environment monad and the State monad.
//! The state is mutable, but not by means of returning a new value.
//! So the signature isn't `(S) -> (A, S)`, but something like `(&mut S) -> A`.

use std::marker::PhantomData;

/// Represents something that can perform a computation based on a shared mutable state.
/// The result of the computation is expressed as a Result.
pub trait Stateful {
    type Output;
    type State;
    type Error;

    /// Produces the output, given the current state.
    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error>;

    /// Creates a new [Stateful] based on the computed value of this object.
    fn flat_map<F, N>(self, f: F) -> FlatMapState<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Output) -> N,
        N: Stateful<State = Self::State>,
    {
        FlatMapState::new(self, f)
    }

    /// Maps the computed value of the current [Stateful] into a new value, without using the state.
    fn map<F, N>(self, f: F) -> MapState<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Output) -> N,
    {
        MapState::new(self, f)
    }

    /// Uses a different state for computing this object. The state is derived
    /// from the current state using the given function.
    fn in_child_state<F, ParentState>(self, state_selector: F) -> InChildState<Self, F, ParentState>
    where
        Self: Sized,
        F: FnOnce(&mut ParentState) -> &mut Self::State,
    {
        InChildState::new(self, state_selector)
    }
}

/// A [Stateful] that returns its value without using the state.
pub struct Unit<A, S, E> {
    value: A,
    state: PhantomData<S>,
    error: PhantomData<E>,
}

impl<A, S, E> Unit<A, S, E> {
    pub fn new(value: A) -> Self {
        Self {
            value,
            state: PhantomData,
            error: PhantomData,
        }
    }
}

impl<A, S, E> Stateful for Unit<A, S, E> {
    type Output = A;
    type State = S;
    type Error = E;

    fn unwrap(self, _: &mut Self::State) -> Result<A, Self::Error> {
        Ok(self.value)
    }
}

/// Creates a new [Stateful] based on the computed value of this object.
pub struct FlatMapState<Decorated, F> {
    decorated: Decorated,
    mapper: F,
}

impl<Decorated, F> FlatMapState<Decorated, F> {
    pub fn new(decorated: Decorated, mapper: F) -> Self {
        Self { decorated, mapper }
    }
}

impl<Decorated, F, N> Stateful for FlatMapState<Decorated, F>
where
    Decorated: Stateful,
    F: FnOnce(Decorated::Output) -> N,
    N: Stateful<State = Decorated::State, Error = Decorated::Error>,
{
    type Output = N::Output;
    type State = Decorated::State;
    type Error = Decorated::Error;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let x = self.decorated.unwrap(state)?;
        let y = (self.mapper)(x);
        y.unwrap(state)
    }
}

/// Maps the computed value of the current [Stateful] into a new value, without using the state.
pub struct MapState<Decorated, F> {
    decorated: Decorated,
    mapper: F,
}

impl<Decorated, F> MapState<Decorated, F> {
    pub fn new(decorated: Decorated, mapper: F) -> Self {
        Self { decorated, mapper }
    }
}

impl<Decorated, F, N> Stateful for MapState<Decorated, F>
where
    Decorated: Stateful,
    F: FnOnce(Decorated::Output) -> N,
{
    type Output = N;
    type State = Decorated::State;
    type Error = Decorated::Error;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let x = self.decorated.unwrap(state)?;
        Ok((self.mapper)(x))
    }
}

// blanket implementation for tuple of 2

impl<A, B> Stateful for (A, B)
where
    A: Stateful,
    B: Stateful<State = A::State, Error = A::Error>,
{
    type Output = (A::Output, B::Output);
    type State = A::State;
    type Error = A::Error;

    fn unwrap(self, context: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let (left, right) = self;
        let left = left.unwrap(context)?;
        let right = right.unwrap(context)?;
        Ok((left, right))
    }
}

/// Wrapper for a [FnOnce].
pub struct FnStateful<F, A, S, E>(F, PhantomData<A>, PhantomData<S>, PhantomData<E>);

impl<F, A, S, E> FnStateful<F, A, S, E> {
    pub fn new(f: F) -> Self {
        Self(f, PhantomData, PhantomData, PhantomData)
    }
}

impl<F, A, S, E> Stateful for FnStateful<F, A, S, E>
where
    F: FnOnce(&mut S) -> Result<A, E>,
{
    type Output = A;
    type State = S;
    type Error = E;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        (self.0)(state)
    }
}

/// A specialization of [Stateful], for when the computed value is an Option.
pub trait OptStateful<O>: Stateful<Output = Option<O>> + Sized {
    /// A variation of the flat map operation, that only applies the mapper when a value is present.
    fn opt_flat_map<F, N>(self, mapper: F) -> OptFlatMap<Self, F>
    where
        F: FnOnce(O) -> N,
        N: Stateful<State = Self::State, Error = Self::Error>,
    {
        OptFlatMap {
            decorated: self,
            mapper,
        }
    }
}

impl<S, O> OptStateful<O> for S where S: Stateful<Output = Option<O>> {}

/// A variation of the flat map operation, that only applies the mapper when a value is present.
pub struct OptFlatMap<Decorated, F> {
    decorated: Decorated,
    mapper: F,
}

impl<Decorated, F, O, N> Stateful for OptFlatMap<Decorated, F>
where
    Decorated: Stateful<Output = Option<O>>,
    F: FnOnce(O) -> N,
    N: Stateful<State = Decorated::State, Error = Decorated::Error>,
{
    type Output = Option<N::Output>;
    type State = Decorated::State;
    type Error = Decorated::Error;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let opt_value = self.decorated.unwrap(state)?;
        match opt_value {
            Some(value) => {
                let next_stateful = (self.mapper)(value);
                let next = next_stateful.unwrap(state)?;
                Ok(Some(next))
            }
            None => Ok(None),
        }
    }
}

/// A specialization of [Stateful], for when the computed value is a Vec.
pub trait VecStateful<O>: Stateful<Output = Vec<O>> + Sized {
    /// A variation of the flat map operation, that applies the mapper to each element.
    fn vec_flat_map<F, N>(self, mapper: F) -> VecFlatMap<Self, F>
    where
        F: Fn(O) -> N,
        N: Stateful<State = Self::State, Error = Self::Error>,
    {
        VecFlatMap {
            decorated: self,
            mapper,
        }
    }

    /// Filters elements of the computed vector by a predicate.
    fn vec_filter<F>(self, predicate: F) -> VecFilter<Self, F>
    where
        F: Fn(&O) -> bool,
    {
        VecFilter {
            decorated: self,
            predicate,
        }
    }
}

impl<S, O> VecStateful<O> for S where S: Stateful<Output = Vec<O>> {}

/// A variation of the flat map operation, that applies the mapper to each element.
pub struct VecFlatMap<Decorated, F> {
    decorated: Decorated,
    mapper: F,
}

impl<Decorated, F, O, N> Stateful for VecFlatMap<Decorated, F>
where
    Decorated: Stateful<Output = Vec<O>>,
    F: Fn(O) -> N,
    N: Stateful<State = Decorated::State, Error = Decorated::Error>,
{
    type Output = Vec<N::Output>;
    type State = Decorated::State;
    type Error = Decorated::Error;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let items = self.decorated.unwrap(state)?;
        let mut result: Vec<N::Output> = vec![];
        for item in items {
            let next_stateful = (self.mapper)(item);
            let next = next_stateful.unwrap(state)?;
            result.push(next);
        }
        Ok(result)
    }
}

/// Filters elements of the computed vector by a predicate.
pub struct VecFilter<Decorated, F> {
    decorated: Decorated,
    predicate: F,
}

impl<Decorated, F, O> Stateful for VecFilter<Decorated, F>
where
    Decorated: Stateful<Output = Vec<O>>,
    F: Fn(&O) -> bool,
{
    type Output = Decorated::Output;
    type State = Decorated::State;
    type Error = Decorated::Error;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let Self {
            decorated,
            predicate,
        } = self;
        let items = decorated.unwrap(state)?;
        // TODO instead of VecStateful, try an IterStateful
        Ok(items.into_iter().filter(predicate).collect())
    }
}

/// Uses a different state to compute a [Stateful].
pub struct InChildState<Decorated, F, ParentState> {
    decorated: Decorated,
    state_selector: F,
    parent_state: PhantomData<ParentState>,
}

impl<Decorated, F, ParentState> InChildState<Decorated, F, ParentState> {
    pub fn new(decorated: Decorated, state_selector: F) -> Self {
        Self {
            decorated,
            state_selector,
            parent_state: PhantomData,
        }
    }
}

impl<Decorated, F, ParentState> Stateful for InChildState<Decorated, F, ParentState>
where
    Decorated: Stateful,
    F: FnOnce(&mut ParentState) -> &mut Decorated::State,
{
    type Output = Decorated::Output;
    type State = ParentState;
    type Error = Decorated::Error;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let child_state = (self.state_selector)(state);
        self.decorated.unwrap(child_state)
    }
}
