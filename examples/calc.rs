use flexi_parse::group::Group;
use flexi_parse::group::Parentheses;
use flexi_parse::parse;
use flexi_parse::parse_string;
use flexi_parse::pretty_unwrap;
use flexi_parse::token;
use flexi_parse::Parse;
use flexi_parse::ParseStream;
use flexi_parse::Punct;
use flexi_parse::Result;

use std::env;

enum Expr {
    Num(f64),
    Neg(Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn eval(&self) -> f64 {
        match self {
            Expr::Num(num) => *num,
            Expr::Neg(expr) => -expr.eval(),
            Expr::Mul(left, right) => left.eval() * right.eval(),
            Expr::Div(left, right) => left.eval() / right.eval(),
            Expr::Mod(left, right) => left.eval() % right.eval(),
            Expr::Add(left, right) => left.eval() + right.eval(),
            Expr::Sub(left, right) => left.eval() - right.eval(),
        }
    }
}

impl Parse for Expr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        addition(input)
    }
}

fn addition(input: ParseStream<'_>) -> Result<Expr> {
    let mut expr = factor(input)?;
    loop {
        if input.parse::<Option<Punct!["+"]>>()?.is_some() {
            expr = Expr::Add(Box::new(expr), Box::new(factor(input)?));
        } else if input.parse::<Option<Punct!["-"]>>()?.is_some() {
            expr = Expr::Sub(Box::new(expr), Box::new(factor(input)?));
        } else {
            break;
        }
    }
    Ok(expr)
}

fn factor(input: ParseStream<'_>) -> Result<Expr> {
    let mut expr: Expr = unary(input)?;
    loop {
        if input.parse::<Option<Punct!["*"]>>()?.is_some() {
            expr = Expr::Mul(Box::new(expr), Box::new(unary(input)?));
        } else if input.parse::<Option<Punct!["/"]>>()?.is_some() {
            expr = Expr::Div(Box::new(expr), Box::new(unary(input)?));
        } else if input.parse::<Option<Punct!["%"]>>()?.is_some() {
            expr = Expr::Mod(Box::new(expr), Box::new(unary(input)?));
        } else {
            break;
        }
    }
    Ok(expr)
}

fn unary(input: ParseStream<'_>) -> Result<Expr> {
    if input.parse::<Option<Punct!["-"]>>()?.is_some() {
        Ok(Expr::Neg(Box::new(unary(input)?)))
    } else {
        primary(input)
    }
}

fn primary(input: ParseStream<'_>) -> Result<Expr> {
    let lookahead = input.lookahead();
    if lookahead.peek(token::LitFloat) {
        Ok(Expr::Num(input.parse::<token::LitFloat>()?.value()))
    } else if lookahead.peek(token::LitInt) {
        Ok(Expr::Num(input.parse::<token::LitInt>()?.value() as f64))
    } else if lookahead.peek(token::LeftParen) {
        let group: Group<Parentheses> = input.parse()?;
        parse(group.into_token_stream())
    } else {
        Err(lookahead.error())
    }
}

fn main() {
    let expr: Expr = pretty_unwrap(parse_string(env::args().nth(1).expect("expect expression")));
    println!("{}", expr.eval());
}

#[cfg(test)]
mod tests {
    use super::Expr;

    use flexi_parse::parse_string;
    use flexi_parse::pretty_unwrap;

    #[test]
    fn test_features() {
        let expr: Expr = pretty_unwrap(parse_string(
            "
                (
                    (
                        (4 - 1) + 5
                    ) / (2.5 + -0.5) * 2
                ) % 3
            "
            .to_string(),
        ));
        assert_eq!(expr.eval(), 2.0);
    }
}
