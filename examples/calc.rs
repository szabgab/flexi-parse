use flexi_parse::parse;
use flexi_parse::parse_string;
use flexi_parse::pretty_unwrap;
use flexi_parse::token;
use flexi_parse::token::Group;
use flexi_parse::token::Parenthesis;
use flexi_parse::Parse;
use flexi_parse::ParseStream;
use flexi_parse::Result;

#[derive(Debug, Clone)]
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
        if input.parse::<Option<token::Plus>>()?.is_some() {
            expr = Expr::Add(Box::new(expr), Box::new(factor(input)?));
        } else if input.parse::<Option<token::Dash>>()?.is_some() {
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
        if input.parse::<Option<token::Asterisk>>()?.is_some() {
            expr = Expr::Mul(Box::new(expr), Box::new(unary(input)?));
        } else if input.parse::<Option<token::Slash>>()?.is_some() {
            expr = Expr::Div(Box::new(expr), Box::new(unary(input)?));
        } else if input.parse::<Option<token::Percent>>()?.is_some() {
            expr = Expr::Mod(Box::new(expr), Box::new(unary(input)?));
        } else {
            break;
        }
    }
    Ok(expr)
}

fn unary(input: ParseStream<'_>) -> Result<Expr> {
    if input.parse::<Option<token::Dash>>()?.is_some() {
        Ok(Expr::Neg(Box::new(unary(input)?)))
    } else {
        primary(input)
    }
}

fn primary(input: ParseStream<'_>) -> Result<Expr> {
    if let Some(f) = input.parse::<Option<token::LitFloat>>()? {
        Ok(Expr::Num(f.value()))
    } else if let Some(f) = input.parse::<Option<token::LitInt>>()? {
        Ok(Expr::Num(f.value() as f64))
    } else {
        let group: Group<Parenthesis> = input.parse()?;
        parse(&mut group.token_stream())
    }
}

fn main() {
    let expr: Expr = pretty_unwrap(parse_string("(3 + 5) / 2".to_string()));
    println!("{}", expr.eval());
}

#[cfg(test)]
mod tests {
    use super::Expr;

    use flexi_parse::parse_string;
    use flexi_parse::pretty_unwrap;

    #[test]
    fn basic_use() {
        let expr: Expr = pretty_unwrap(parse_string("(3 + 5) / 2".to_string()));
        assert_eq!(expr.eval(), 4.0);
    }
}