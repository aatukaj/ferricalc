use super::ast::*;
use crate::scanner::{Token, TokenKind};
use std::mem::discriminant;

pub struct Parser<'a> {
    current: usize,
    tokens: &'a [Token],
    source: &'a str,
}

#[allow(unused)]
impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token], source: &'a str) -> Self {
        Self {
            current: 0,
            tokens,
            source,
        }
    }
    fn match_tokens(&mut self, tokens: &[TokenKind]) -> bool {
        for t in tokens {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        discriminant(kind) == discriminant(&self.peek().kind)
    }
    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.term()
    }
    fn stmt(&mut self) -> Result<Stmt, String> {
        let mut expr = self.expression()?;

        if self.match_tokens(&[TokenKind::Equal]) {
            let equals = self.previous().clone();
            let value = self.expression()?;
            return match expr {
                Expr::Var(v) => Ok(Stmt::VarAssign(VarAssign {
                    name: v.name,
                    value,
                })),
                Expr::FnCall(f) => {
                    let args = f
                        .arguments
                        .into_iter()
                        .map(|arg| match arg {
                            Expr::Var(v) => Ok(v.name),
                            _ => Err("Invalid function args"),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(Stmt::FnAssign(FnAssign {
                        name: f.name,
                        arguments: args,
                        expr: value,
                    }))
                }
                _ => Err("Expected function or variable assignment".to_string()),
            };
        }
        Ok(Stmt::Expr(expr))
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;
        while self.match_tokens(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let rhs = self.factor()?;
            expr = Expr::Binary(Box::new(BinaryExpr {
                lhs: expr,
                operator,
                rhs,
            }))
        }
        Ok(expr)
    }
    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        while self.match_tokens(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous().clone();
            let rhs = self.unary()?;
            expr = Expr::Binary(Box::new(BinaryExpr {
                lhs: expr,
                operator,
                rhs,
            }))
        }
        Ok(expr)
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().clone();
            let rhs = self.unary()?;
            return Ok(Expr::Unary(Box::new(UnaryExpr { operator, rhs })));
        }
        self.exp()
    }
    fn exp(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;
        while self.match_tokens(&[TokenKind::Exp]) {
            let operator = self.previous().clone();
            let rhs = self.unary()?;
            expr = Expr::Binary(Box::new(BinaryExpr {
                lhs: expr,
                operator,
                rhs,
            }))
        }
        Ok(expr)
    }
    fn primary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenKind::Number]) {
            return Ok(Expr::Literal(self.previous().literal.clone().unwrap()));
        }
        if self.match_tokens(&[TokenKind::LParen]) {
            let expr = self.expression()?;
            self.consume(&TokenKind::RParen, "Expect ')' after expression.".into())?;
            return Ok(Expr::Grouping(Box::new(GroupingExpr(expr))));
        }
        if self.match_tokens(&[TokenKind::Indentifier]) {
            let name = self.source[self.previous().clone().span()].to_string();
            if self.match_tokens(&[TokenKind::LParen]) {
                let mut arguments = Vec::new();
                loop {
                    arguments.push(self.expression()?);
                    if !self.match_tokens(&[TokenKind::Comma]) {
                        break;
                    }
                }
                self.consume(&TokenKind::RParen, "Expect ')' after function call.".into())?;
                return Ok(Expr::FnCall(FnCall { name, arguments }));
            } else {
                return Ok(Expr::Var(Var { name }));
            }
        }
        Err("Expected expression".to_string())?
    }
    fn consume(&mut self, kind: &TokenKind, error: String) -> Result<&Token, String> {
        if self.check(kind) {
            return Ok(self.advance());
        }
        Err(error)
    }
    pub fn parse(&mut self) -> Result<Stmt, String> {
        let res = self.stmt()?;
        if !self.is_at_end() {
            Err("Expected EOF".to_string())?
        }
        Ok(res)
    }
    pub fn error(&self, msg: String, token: &Token) -> String {
        format!("{}\n{}^ {msg}", self.source, " ".repeat(token.span().start))
    }
}

pub struct AstPrinter<'a> {
    pub source: &'a str,
}

impl Visitor<String> for AstPrinter<'_> {
    fn visit_grouping_expr(&mut self, e: &GroupingExpr) -> String {
        format!("(group {})", self.visit_expr(&e.0))
    }
    fn visit_var(&mut self, e: &Var) -> String {
        e.name.to_string()
    }
    fn visit_binary_expr(&mut self, e: &BinaryExpr) -> String {
        format!(
            "({} {} {})",
            &self.source[e.operator.span()],
            self.visit_expr(&e.lhs),
            self.visit_expr(&e.rhs)
        )
    }

    fn visit_unary_expr(&mut self, e: &UnaryExpr) -> String {
        format!(
            "({} {})",
            &self.source[e.operator.span()],
            self.visit_expr(&e.rhs)
        )
    }

    fn visit_literal(&mut self, e: &Literal) -> String {
        format!("{}", e)
    }
    fn visit_func_call(&mut self, e: &FnCall) -> String {
        let mut s = format!("({}", e.name);
        for arg in &e.arguments {
            s.push_str(&format!(" {}", self.visit_expr(arg)))
        }
        s.push(')');
        s
    }
    fn visit_stmt(&mut self, s: &Stmt) -> String {
        match s {
            Stmt::VarAssign(e) => format!("{} = {}", e.name, self.visit_expr(&e.value)),
            Stmt::FnAssign(e) => format!(
                "{} = {}",
                self.visit_func_call(&FnCall {
                    name: e.name.clone(),
                    arguments: e
                        .arguments
                        .iter()
                        .cloned()
                        .map(|t| Expr::Var(Var { name: t }))
                        .collect()
                }),
                self.visit_expr(&e.expr)
            )
        ,
            Stmt::Expr(e) => self.visit_expr(e),
        }
    }
}
