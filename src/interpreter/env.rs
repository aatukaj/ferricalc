use std::{
    borrow::{Borrow, Cow},
    collections::{btree_map::Entry, BTreeMap},
    rc::Rc,
};

use rug::Float;

use crate::ast::Expr;

#[derive(Debug, Clone)]
pub struct UserFn {
    pub expr: Rc<Expr>,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Func {
    UserFn(UserFn),
    BuiltinFn(fn(&[Float]) -> Float),
}

#[derive(Debug)]
pub enum EnvMember {
    Var(Float),
    Fn(Func),
}
impl EnvMember {
    pub fn to_kind(&self) -> EnvMemberKind {
        match self {
            EnvMember::Fn(_) => EnvMemberKind::Fn,
            EnvMember::Var(_) => EnvMemberKind::Var,
        }
    }
}

pub enum EnvMemberKind {
    Var,
    Fn,

}




#[derive(Debug)]
pub struct Env {
    pub(super) members: BTreeMap<Cow<'static, str>, EnvMember>,
}
impl<'q, 'env: 'q> Env {
    pub fn new() -> Self {
        Self {
            members: <_>::default(),
        }
    }
    pub fn set_func(&mut self, k: String, v: Func) -> Result<(), String> {
        match self.members.entry(k.into()) {
            Entry::Occupied(mut e) => {e.insert(EnvMember::Fn(v));},
            Entry::Vacant(e) => {e.insert(EnvMember::Fn(v));}
        }
        Ok(())
    }
    pub fn get_func(&'env self, q: &'q str) -> Option<&'q Func> {
        self.members.get(q.into()).and_then(|e| match e {
            EnvMember::Fn(v) => Some(v),
            _ => None,
        })
    }

    pub fn set_var<K: Into<Cow<'static, str>>>(&mut self, k: K, v: Float) -> Result<(), String> {
        self.members
            .entry(k.into())
            .and_modify(|t| match t {
                EnvMember::Var(f) => *f = v.clone(),
                _ => (),
            })
            .or_insert(EnvMember::Var(v));
        Ok(())
    }
    pub fn get_var(&'env self, q: &'q str) -> Option<&'q Float> {
        self.members.get(q.into()).and_then(|e| match e {
            EnvMember::Var(v) => Some(v),
            _ => None,
        })
    }
    pub fn search(&'env self, q: &'q str) -> impl Iterator<Item = (&'q str, &'q EnvMember)> {
        let r = <Cow<_>>::from(q);
        self.members
            .range(r..)
            .take_while(move |(k, _)| k.len() >= q.len() && q == &k[0..q.len()])
            .map(|(k, e)| ((*k).borrow(), e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn env() {
        let mut e = Env::new();
        e.set_var("sin", Float::new(1)).unwrap();
        e.set_var("sum", Float::new(1)).unwrap();
        e.set_var("sqrt", Float::new(1)).unwrap();
        for (s, _) in e.members.range(<Cow<_>>::from("sq")..) {
            println!("{s}")
        }
    }
}
