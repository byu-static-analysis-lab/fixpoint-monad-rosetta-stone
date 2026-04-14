use std::hash::Hash;

mod monad;
mod state;
mod memo;

pub use monad::*;
pub use state::*;
pub use memo::*;

pub trait MakeKey {
    fn make_key<S: AsRef<str>>(&self, prefix: S) -> String {
        format!("{}:{}", prefix.as_ref(), self.show())
    }

    fn show(&self) -> String;
}


impl MakeKey for bool {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}

impl MakeKey for u8 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}

impl MakeKey for u16 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}

impl MakeKey for u32 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}
impl MakeKey for u64 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}
impl MakeKey for u128 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}
impl MakeKey for i8 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}
impl MakeKey for i16 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}
impl MakeKey for i32 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}
impl MakeKey for i64 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}
impl MakeKey for i128 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}
impl MakeKey for f32 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}
impl MakeKey for f64 {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}

impl MakeKey for str {
    fn show(&self) -> String {
        self.to_string()
    }
}

impl MakeKey for String {
    fn show(&self) -> String {
        format!("{}", *self)
    }
}

impl<T: MakeKey> MakeKey for Box<T> {
    fn show(&self) -> String {
        format!("{}", (**self).show())
    }
}

impl<T: MakeKey> MakeKey for Vec<T> {
    fn show(&self) -> String {
        let mut out = String::from("[");
        for (i, item) in self.iter().enumerate() {
            out.push_str(&item.show());
            if i < self.len() - 1 {
                out.push_str(", ");
            }
        }
        out.push_str("]");
        out
    }
}

pub trait Cachable: Hash + PartialEq + PartialOrd + Clone + Eq + Ord + Default {}

impl Cachable for bool {}
impl Cachable for u8 {}
impl Cachable for u16 {}
impl Cachable for u32 {}
impl Cachable for u64 {}
impl Cachable for u128 {}
impl Cachable for i8 {}
impl Cachable for i16 {}
impl Cachable for i32 {}
impl Cachable for i64 {}
impl Cachable for i128 {}


