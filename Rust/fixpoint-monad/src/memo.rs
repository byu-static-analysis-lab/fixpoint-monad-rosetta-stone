use std::cell::RefCell;
use std::rc::Rc;
use crate::monad::{Continuation, Monad, Transformer};
use crate::state::{State, StateValue};
use crate::{MakeKey, Cachable};

/// **memo**: memoize a function with demand-driven fixed point computation
/**   
- First call: register continuation, compute, deliver results
- Later calls: register continuation, deliver cached results
- When new values arrive via deliver, all waiting continuations receive them
*/
#[derive(Clone)]
pub struct Memo<'a, V: Cachable + 'a, Args: Clone + MakeKey + 'a> {
    tag: String,
    f: Rc<RefCell<dyn FnMut(Args) -> Monad<'a, V> + 'a>>
}

impl<'a, V: Cachable + 'a, Args: Clone + MakeKey + 'a> Memo<'a, V, Args> {
    pub fn new<F>(tag: &str, f: F) -> Memo<'a, V, Args>
    where
        F: FnMut(Args) -> Monad<'a, V> + 'a
    {
        let tag = tag.to_owned();
        Memo {
            tag,
            f: Rc::new(RefCell::new(f))
        }
    }

    pub fn call(&self, args: Args) -> Monad<'a, V> {
        let tag = args.make_key(&self.tag);
        let f = self.f.clone();
        let args = args.clone();

        Monad::new(move |continuation: Continuation<'a, V>| {
            let tag = tag.clone();
            let f = f.clone();
            let args = args.clone();

            Transformer::new(move |s: State<'a, V>| {
                match s.get(&tag) {
                    None => {
                        let new_s = s.insert(tag.clone(), StateValue::new(continuation.clone()));
                        let c = f.borrow_mut()(args.clone());
                        let del_cont = deliver(&tag.clone());
                        let transformer = c.apply(del_cont);
                        transformer.apply(new_s)
                    }
                    Some(state_value) => {
                        let state_value = state_value.add_waiter(continuation.clone());

                        let mut new_s = s.insert(tag.clone(), state_value.clone());

                        for value in state_value.values {
                            let transformer = continuation.apply(vec![value]);
                            new_s = transformer.apply(new_s);
                        }
                        new_s
                    }
                }
            })
        })
    }
}

impl<'a, V: Cachable + 'a> Memo<'a, V, String> {
    pub fn sigma() -> Memo<'a, V, String> {
        Memo::new("σ", |_| Monad::fail())
    }
}

/// **deliver**: propagate a new value to all waiting continuations
/**
- Once given a key, (deliver key) is itself a continuation
- If value already seen, do nothing (fixed point for this value)
- Otherwise, add to cache and notify all waiters
*/
pub fn deliver<'a, V: Cachable + 'a>(key: &str) -> Continuation<'a, V> {
    let key = key.to_string();
    Continuation::new(move |vs: Vec<V>| {
        let key = key.clone();
        let vs = vs.clone();
        Transformer::new(move |s: State<'a, V>| {
            let key = key.clone();
            let vs = vs.clone();

            // Get the current state value or create default
            let mut state_value = s.get(&key).unwrap_or(StateValue::default());

            // Check if we've already seen these values
            for value in &vs {
                if state_value.contains(value) {
                    return s;
                }
            }

            // Add new values to the set
            state_value = state_value.add_values(vs.clone());

            // Update the state
            let mut s_prime = s.insert(key.clone(), state_value.clone());

            // Notify all waiting continuations
            for cont in state_value.cont_iter() {
                let transformer = cont.apply(vs.clone());
                s_prime = transformer.apply(s_prime.clone());
            }

            s_prime
        })
    })
}

pub fn bind<'a, V: Cachable + 'a>(x: &str, v: V) -> Monad<'a, V> {
    let x = x.to_string();
    Monad::new(move |continuation: Continuation<'a, V>| {
        let x = x.clone();
        let v = v.clone();

        Transformer::new(move |s: State<'a, V>| {
            let x = x.clone();

            // First, deliver the value to the variable's address
            let key = x.make_key("σ");
            let deliver_cont = deliver(&key);
            let deliver_transformer = deliver_cont.apply(vec![v.clone()]);
            let s_prime = deliver_transformer.apply(s);

            // Then, call the continuation with an empty list
            let result_transformer = continuation.apply(vec![]);
            result_transformer.apply(s_prime)
        })
    })
}

pub fn bind_each<'a, V: Cachable + 'a>(x: &[String], vs: &[V]) -> Monad<'a, V> {
    if x.len() != vs.len() {
        return Monad::fail()
    }

    let monads = x.into_iter().zip(vs.into_iter())
        .map(|(var, value)| {
            bind(var, value.clone())
        }).collect::<Vec<Monad<_>>>();

    Monad::each(monads)
}
