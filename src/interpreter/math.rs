use rug::Float;

use crate::PREC_BITS;

use super::env::*;

fn sum(args: &[Float]) -> Float {
    args.into_iter()
        .fold(Float::new(PREC_BITS), |acc, f| acc + f)
}
fn sqrt(args: &[Float]) -> Float {
    args[0].clone().sqrt()
}
fn avg(args: &[Float]) -> Float {
    sum(args) / args.len() as u32
}
fn max(args: &[Float]) -> Float {
    args.iter().max_by_key(|f| f.as_ord()).unwrap().clone()
}
fn min(args: &[Float]) -> Float {
    args.iter().min_by_key(|f| f.as_ord()).unwrap().clone()
}
fn sin(args: &[Float]) -> Float {
    args[0].clone().sin()
}

pub (super) fn insert_funcs(env: &mut Env) {
    macro_rules! insert_funcs {
        ($( $func:ident ),+) => {
            $(
                env.members.insert(stringify!($func).into(), EnvMember::Fn(Func::BuiltinFn($func)));
            )*
        };
    }
    insert_funcs!(sum, sqrt, avg, min, max, sin);
}