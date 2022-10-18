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
pub struct FlatMapState<S, F> {
    current: S,
    mapper: F,
}

impl<S, F> FlatMapState<S, F> {
    pub fn new(current: S, mapper: F) -> Self {
        Self { current, mapper }
    }
}

impl<S, F, N> Stateful for FlatMapState<S, F>
where
    S: Stateful,
    F: FnOnce(S::Output) -> N,
    N: Stateful<State = S::State, Error = S::Error>,
{
    type Output = N::Output;
    type State = S::State;
    type Error = S::Error;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let x = self.current.unwrap(state)?;
        let y = (self.mapper)(x);
        y.unwrap(state)
    }
}

/// Maps the computed value of the current [Stateful] into a new value, without using the state.
pub struct MapState<S, F> {
    current: S,
    mapper: F,
}

impl<S, F> MapState<S, F> {
    pub fn new(current: S, mapper: F) -> Self {
        Self { current, mapper }
    }
}

impl<S, F, N> Stateful for MapState<S, F>
where
    S: Stateful,
    F: FnOnce(S::Output) -> N,
{
    type Output = N;
    type State = S::State;
    type Error = S::Error;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let x = self.current.unwrap(state)?;
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
            current: self,
            mapper,
        }
    }
}

impl<S, O> OptStateful<O> for S where S: Stateful<Output = Option<O>> {}

/// A variation of the flat map operation, that only applies the mapper when a value is present.
pub struct OptFlatMap<S, F> {
    current: S,
    mapper: F,
}

impl<S, F, O, N> Stateful for OptFlatMap<S, F>
where
    S: Stateful<Output = Option<O>>,
    F: FnOnce(O) -> N,
    N: Stateful<State = S::State, Error = S::Error>,
{
    type Output = Option<N::Output>;
    type State = S::State;
    type Error = S::Error;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let opt_value = self.current.unwrap(state)?;
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
            current: self,
            mapper,
        }
    }
}

impl<S, O> VecStateful<O> for S where S: Stateful<Output = Vec<O>> {}

/// A variation of the flat map operation, that applies the mapper to each element.
pub struct VecFlatMap<S, F> {
    current: S,
    mapper: F,
}

impl<S, F, O, N> Stateful for VecFlatMap<S, F>
where
    S: Stateful<Output = Vec<O>>,
    F: Fn(O) -> N,
    N: Stateful<State = S::State, Error = S::Error>,
{
    type Output = Vec<N::Output>;
    type State = S::State;
    type Error = S::Error;

    fn unwrap(self, state: &mut Self::State) -> Result<Self::Output, Self::Error> {
        let items = self.current.unwrap(state)?;
        let mut result: Vec<N::Output> = vec![];
        for item in items {
            let next_stateful = (self.mapper)(item);
            let next = next_stateful.unwrap(state)?;
            result.push(next);
        }
        Ok(result)
    }
}
