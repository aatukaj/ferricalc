pub mod ast;
pub mod env;
pub mod math;
pub mod parser;

use self::ast::*;
use self::env::*;

use crate::scanner::TokenKind;
use rug::{ops::Pow, Float};
use std::{collections::HashMap, rc::Rc};

pub const PREC_BITS: u32 = 256;

pub struct Interpreter {
    pub env: Env,
    scope: Option<HashMap<String, Float>>,

    pub last_ans: Float,
    pub save_assignments: bool,
}
impl Interpreter {
    pub fn new() -> Self {
        let mut env = Env::new();
        math::insert_funcs(&mut env);
        Self {
            env,
            scope: None,
            last_ans: Float::new(PREC_BITS),
            save_assignments: true,
        }
    }
}

impl Visitor<Result<Float, String>> for Interpreter {
    fn visit_grouping_expr(&mut self, e: &GroupingExpr) -> Result<Float, String> {
        self.visit_expr(&e.0)
    }
    fn visit_var(&mut self, e: &Var) -> Result<Float, String> {
        let name = e.name.as_str();
        (name == "ans")
            .then_some(self.last_ans.clone())
            .or(self.scope.as_mut().and_then(|s| s.get(name).cloned()))
            .or(self.env.get_var(name).cloned())
            .ok_or(format!("Undeclared variable '{name}'"))
    }
    fn visit_binary_expr(&mut self, e: &BinaryExpr) -> Result<Float, String> {
        let lhs = self.visit_expr(&e.lhs)?;
        let rhs = self.visit_expr(&e.rhs)?;
        Ok(match e.operator.kind {
            TokenKind::Plus => lhs + rhs,
            TokenKind::Minus => lhs - rhs,
            TokenKind::Slash => lhs / rhs,
            TokenKind::Star => lhs * rhs,
            TokenKind::Exp => lhs.pow(&rhs),
            ref t => panic!("Unexpected Token {t:?}"),
        })
    }

    fn visit_unary_expr(&mut self, e: &UnaryExpr) -> Result<Float, String> {
        let rhs = self.visit_expr(&e.rhs)?;
        Ok(match e.operator.kind {
            TokenKind::Minus => -rhs,
            TokenKind::Plus => rhs,
            ref t => panic!("Unexpected Token {t:?}"),
        })
    }

    fn visit_literal(&mut self, e: &Literal) -> Result<Float, String> {
        Ok(match e {
            Literal::Number(n) => n.clone(),
        })
    }

    fn visit_func_call(&mut self, e: &FnCall) -> Result<Float, String> {
        let name = e.name.as_str();
        let args: Vec<Float> = e
            .arguments
            .iter()
            .map(|e| self.visit_expr(e))
            .collect::<Result<_, _>>()?;
        match self
            .env
            .get_func(name)
            .ok_or(format!("No function named '{name}'"))?
        {
            Func::BuiltinFn(f) => Ok(f(&args)),
            Func::UserFn(f) => {
                if !(f.arguments.len() == e.arguments.len()) {
                    return Err(format!(
                        "Function '{name}' takes {} args",
                        f.arguments.len()
                    ));
                }
                let f_args = f.arguments.clone();
                self.scope = Some(f_args.into_iter().zip(args).collect());
                let res = self.visit_expr(&Rc::clone(&f.expr));
                self.scope = None;
                res
            }
        }
    }
    fn visit_stmt_owned(&mut self, s: Stmt) -> Result<Float, String> {
        match s {
            Stmt::VarAssign(e) => {
                let res = self.visit_expr(&e.value)?;
                if self.save_assignments {
                    self.env.set_var(e.name, res.clone())?;
                }
                Ok(res)
            }
            Stmt::FnAssign(e) => {
                if self.save_assignments {
                    self.env.set_func(
                        e.name,
                        Func::UserFn(UserFn {
                            expr: Rc::new(e.expr),
                            arguments: e.arguments,
                        }),
                    )?;
                }
                Ok(Float::with_val(PREC_BITS, 1.0))
            }
            Stmt::Expr(e) => self.visit_expr(&e),
        }
    }
    fn visit_stmt(&mut self, _s: &Stmt) -> Result<Float, String> {
        unimplemented!()
    }
}
