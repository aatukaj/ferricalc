use std::fmt::Display;

use rug::Float;

use crate::scanner::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Binary(Box<BinaryExpr>),
    Unary(Box<UnaryExpr>),
    Grouping(Box<GroupingExpr>),
    Var(Var),
    FnCall(FnCall),
}
pub enum Stmt {
    VarAssign(VarAssign),
    FnAssign(FnAssign),
    Expr(Expr),
}



#[derive(Clone, Debug)]
pub struct VarAssign {
    pub name: String,
    pub value: Expr,
}

#[derive(Clone, Debug)]
pub struct Var {
    pub name: String,
}
#[derive(Debug, Clone)]
pub struct FnCall {
    pub name: String,
    pub arguments: Vec<Expr>,
}
#[derive(Clone, Debug)]
pub struct FnAssign {
    pub name: String,
    pub arguments: Vec<String>,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(Float),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            match self {
                Self::Number(n) => n,
            }
        ))
    }
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub lhs: Expr,
    pub operator: Token,
    pub rhs: Expr,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub operator: Token,
    pub rhs: Expr,
}

#[derive(Debug, Clone)]
pub struct GroupingExpr(pub Expr);


pub trait Visitor<T> {
    fn visit_grouping_expr(&mut self, e: &GroupingExpr) -> T;
    fn visit_binary_expr(&mut self, e: &BinaryExpr) -> T;
    fn visit_unary_expr(&mut self, e: &UnaryExpr) -> T;
    fn visit_literal(&mut self, e: &Literal) -> T;
    fn visit_var(&mut self, e: &Var) -> T;
    fn visit_func_call(&mut self, e: &FnCall) -> T;
    fn visit_expr(&mut self, e: &Expr) -> T {
        match e {
            Expr::Literal(e) => self.visit_literal(e),
            Expr::Binary(e) => self.visit_binary_expr(e),
            Expr::Unary(e) => self.visit_unary_expr(e),
            Expr::Grouping(e) => self.visit_grouping_expr(e),
            Expr::Var(e) => self.visit_var(e),
            Expr::FnCall(e) => self.visit_func_call(e),
        }
    }
    fn visit_stmt_owned(&mut self, s: Stmt) -> T {
        self.visit_stmt(&s)
    }
    fn visit_stmt(&mut self, s: &Stmt) -> T;
}