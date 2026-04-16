use std::cell::RefCell;
use std::rc::Rc;
use crate::Cachable;
use crate::state::State;

/// **tranformer**: takes in a state and updates the state and returns it.
#[derive(Clone)]
pub struct Transformer<'a, V: Cachable + 'a>(Rc<RefCell<dyn FnMut(State<'a, V>) -> State<'a, V> + 'a>>);

impl<'a, V: Cachable + 'a> Transformer<'a, V> {
    pub fn new<F: FnMut(State<'a, V>) -> State<'a, V> + 'a>(f: F) -> Self {
        Transformer(Rc::new(RefCell::new(f)))
    }

    pub fn apply(&self, value: State<'a, V>) -> State<'a, V> {
        self.0.borrow_mut()(value)
    }
}

/// **continuation**: takes in a cachable value and returns a state transformer
#[derive(Clone)]
pub struct Continuation<'a, V: Cachable + 'a>(Rc<RefCell<dyn FnMut(Vec<V>) -> Transformer<'a, V> + 'a>>);

impl<'a, V: Cachable + 'a> Continuation<'a, V> {
    pub fn new(fun: impl FnMut(Vec<V>) -> Transformer<'a,V> + 'a) -> Continuation<'a, V> {
        Continuation(Rc::new(RefCell::new(fun)))
    }

    pub fn apply(&self, values: Vec<V>) -> Transformer<'a,V> {
        self.0.borrow_mut()(values)
    }
}

/// # Monad
/**
This is a continuation monad composed with a state monad.
The state is a memo table mapping keys to Values and Continuations

## Enables
- Nondeterminism: exploring multiple abstract values
- Memoization: caching results and detecting fixed points
- Demand-driven iteration: when new values appear, they delivered to all waiting continuations

## Type
`V`: a Cachable Value

`Fn(Fn(Vec<V>) -> Fn(State) -> State) -> Fn(State -> State)`
*/
#[derive(Clone)]
pub struct Monad<'a, V: Cachable + 'a>(Rc<RefCell<dyn FnMut(Continuation<'a, V>) -> Transformer<'a, V> + 'a>>);


impl<'a, V: Cachable + 'a> Monad<'a, V> {

    pub fn new(f: impl FnMut(Continuation<'a, V>) -> Transformer<'a, V> + 'a) -> Monad<'a, V> {
        Monad(Rc::new(RefCell::new(f)))
    }

    pub fn apply(&self, continuation: Continuation<'a, V>) -> Transformer<'a, V> {
        self.0.borrow_mut()(continuation)
    }

    /// **inject**: inject value into the monad
    /// This is what is commonly called `return` in other languages
    pub fn inject(value: V) -> Monad<'a, V> {
        Self::inject_values([value])
    }

    /// **inject_values**: inject values into the monad
    pub fn inject_values(values: impl IntoIterator<Item=V>) -> Monad<'a, V> {
        let values: Vec<V> = values.into_iter().collect::<Vec<_>>();
        Monad(Rc::new(RefCell::new(move |continuation: Continuation<'a, V>| {
            continuation.apply(values.clone())
        })))
    }

    /// **and_then**: sequence computations, spreading the value list to f via apply
    /// This is what is commonly called `bind` or `>>=` in Haskell
    pub fn and_then<F>(self, f: F) -> Monad<'a, V>
    where
        F: FnMut(Vec<V>) -> Monad<'a, V> + 'a
    {
        let f = Rc::new(RefCell::new(f));
        Monad(Rc::new(RefCell::new(move |continuation: Continuation<'a, V>| {
            let f_clone = f.clone();
            let inner_continuation = Continuation::new(move |vs: Vec<V>| {
                let monad = f_clone.borrow_mut()(vs);
                monad.0.borrow_mut()(continuation.clone())
            });
            self.0.borrow_mut()(inner_continuation)
        })))
    }

    /// **each**: nondeterministic choice — run all computations, threading state
    pub fn each(cs: impl IntoIterator<Item=Monad<'a, V>>) -> Monad<'a, V> {
        let cs = cs.into_iter().collect::<Vec<_>>();
        Monad(Rc::new(RefCell::new(move |continuation: Continuation<'a, V>| {
            let cs = cs.clone();
            let continuation = continuation.clone();
            Transformer::new(move |s: State<'a, V>| {
                let continuation = continuation.clone();
                cs.iter().fold(s.clone(), move |state: State<'a, V>, c: &Monad<'a, V>| {
                    let transformer = c.apply(continuation.clone());
                    transformer.apply(state)
                })
            })
        })))
    }

    // **fail**: no results (each with zero computations)
    pub fn fail() -> Monad<'a, V> {
        Self::each([])
    }

    /// **run**: runs the monad to completion
    pub fn run(self) -> State<'a, V> {
        self.apply(Continuation::new(|_: Vec<V>| {
            Transformer::new(|s: State<'a, V>| {
                s
            })
        })).apply(State::new())
    }
}

impl<V: Cachable> std::fmt::Display for Monad<'_, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "monad")
    }
}

impl<V: Cachable> std::fmt::Debug for Monad<'_, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "monad")
    }
}

