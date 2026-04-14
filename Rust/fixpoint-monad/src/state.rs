use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use crate::Cachable;
use crate::monad::Continuation;

#[derive(Clone, Default)]
pub struct StateValue<'a, V: Cachable + 'a> {
    pub(crate) values: HashSet<V>,
    continuations: Vec<Continuation<'a, V>>
}

impl<'a, V: Cachable + 'a> StateValue<'a, V> {
    pub fn new(continuation: Continuation<'a, V>) -> StateValue<'a, V> {
        StateValue {
            values: HashSet::new(),
            continuations: vec![continuation]
        }
    }

    pub fn add_waiter(&self, continuation: Continuation<'a, V>) -> StateValue<'a, V> {
        let mut clone = self.clone();
        clone.continuations.push(continuation);
        clone
    }

    pub fn add_value(&self, value: V) -> StateValue<'a, V> {
        let mut clone = self.clone();
        clone.values.insert(value);
        clone
    }

    pub fn add_values(&self, values: impl IntoIterator<Item=V>) -> StateValue<'a, V> {
        let mut clone = self.clone();
        clone.values.extend(values);
        clone
    }

    pub fn contains(&self, value: &V) -> bool {
        self.values.contains(value)
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn cont_iter(&self) -> impl Iterator<Item=&Continuation<'a, V>> {
        self.continuations.iter()
    }
}

impl<V: Cachable + Display> Display for StateValue<'_, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{ ")?;

        for (i, item) in self.values.iter().enumerate() {
            write!(f, "{}", item)?;
            if i < self.values.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, " }}")
    }
}


#[derive(Clone, Default)]
pub struct State<'a, V: Cachable + 'a> {
    map: HashMap<String, StateValue<'a, V>>
}

impl<'a, V: Cachable + 'a> State<'a, V> {
    pub fn new() -> State<'a, V> {
        State {
            map: HashMap::new()
        }
    }

    pub fn get(&self, key: &str) -> Option<StateValue<'a, V>> {
        self.map.get(key).cloned()
    }

    pub fn insert(&self, key: String, value: StateValue<'a, V>) -> State<'a, V> {
        let mut clone = self.clone();
        clone.map.insert(key, value);
        clone
    }

    pub fn contains_value(&self, key: &str, expected: &V) -> bool {
        self.get(key).map_or(false, |v| v.contains(expected))
    }

    pub fn key_empty(&self, key: &str) -> bool {
        self.map.get(key).map_or(true, |v| v.is_empty())
    }
}


impl<V: Cachable + Display> Display for State<'_, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (k, v) in &self.map {
            write!(f, "{} ↦ {}\n", k, v)?;
        }
        Ok(())
    }
}
