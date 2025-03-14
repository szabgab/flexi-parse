use flexi_parse::group::Braces;
use flexi_parse::group::Group;
use flexi_parse::group::Parentheses;
use flexi_parse::parse;
use flexi_parse::parse_repeated;
use flexi_parse::parse_string;
use flexi_parse::peek2_any;
use flexi_parse::pretty_unwrap;
use flexi_parse::punctuated::Punctuated;
use flexi_parse::token::Ident;
use flexi_parse::token::LitFloat;
use flexi_parse::token::LitInt;
use flexi_parse::token::LitStrDoubleQuote as LitStr;
use flexi_parse::Parse;
use flexi_parse::ParseStream;
use flexi_parse::Parser;
use flexi_parse::Punct;
use flexi_parse::Result;

mod kw {
    use flexi_parse::keywords_prefixed;

    keywords_prefixed![
        "and", "class", "else", "false", "for", "fun", "if", "nil", "or", "print", "return",
        "super", "this", "true", "var", "while"
    ];
}

#[derive(Debug, Clone, PartialEq)]
enum Binary {
    Mul(Expr, Punct!["*"], Expr),
    Div(Expr, Punct!["/"], Expr),

    Add(Expr, Punct!["+"], Expr),
    Sub(Expr, Punct!["-"], Expr),

    Equal(Expr, Punct!["=="], Expr),
    NotEqual(Expr, Punct!["!="], Expr),

    Greater(Expr, Punct![">"], Expr),
    GreaterEqual(Expr, Punct![">="], Expr),
    Less(Expr, Punct!["<"], Expr),
    LessEqual(Expr, Punct!["<="], Expr),
}

#[derive(Debug, Clone, PartialEq)]
enum Literal {
    False(kw::keyword_false),
    Float(LitFloat),
    Int(LitInt),
    Nil(kw::keyword_nil),
    String(LitStr),
    True(kw::keyword_true),
}

#[derive(Debug, Clone, PartialEq)]
enum Logical {
    And(Expr, kw::keyword_and, Expr),
    Or(Expr, kw::keyword_or, Expr),
}

#[derive(Debug, Clone, PartialEq)]
enum Unary {
    Neg(Punct!["-"], Expr),
    Not(Punct!["!"], Expr),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Assign {
        name: Ident,
        value: Box<Expr>,
    },
    Binary(Box<Binary>),
    Call {
        callee: Box<Expr>,
        paren: Parentheses,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Ident,
    },
    Literal(Literal),
    Logical(Box<Logical>),
    Set {
        object: Box<Expr>,
        name: Ident,
        value: Box<Expr>,
    },
    Super {
        keyword: kw::keyword_super,
        method: Ident,
    },
    This(kw::keyword_this),
    Unary(Box<Unary>),
    Variable(Ident),
}

impl Expr {
    fn binary(binary: Binary) -> Self {
        Expr::Binary(Box::new(binary))
    }

    fn assignment(input: ParseStream<'_>) -> Result<Self> {
        let expr = Expr::or(input)?;

        if input.peek(Punct!["="]) {
            let equals: Punct!["="] = input.parse()?;
            let value = Box::new(Expr::assignment(input)?);

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign { name, value });
            } else if let Expr::Get { object, name } = expr {
                return Ok(Expr::Set {
                    object,
                    name,
                    value,
                });
            }

            input.add_error(input.new_error("Invalid assignment target".to_string(), &equals, 0))
        }

        Ok(expr)
    }

    fn or(input: ParseStream<'_>) -> Result<Self> {
        let mut expr = Expr::and(input)?;

        while input.peek(kw::keyword_or) {
            expr = Expr::Logical(Box::new(Logical::Or(
                expr,
                input.parse()?,
                Expr::and(input)?,
            )));
        }

        Ok(expr)
    }

    fn and(input: ParseStream<'_>) -> Result<Self> {
        let mut expr = Expr::equality(input)?;

        while input.peek(kw::keyword_and) {
            expr = Expr::Logical(Box::new(Logical::And(
                expr,
                input.parse()?,
                Expr::equality(input)?,
            )));
        }

        Ok(expr)
    }

    fn equality(input: ParseStream<'_>) -> Result<Self> {
        let mut expr = Expr::comparison(input)?;

        loop {
            if input.peek(Punct!["=="]) {
                expr = Expr::binary(Binary::Equal(
                    expr,
                    input.parse()?,
                    Expr::comparison(input)?,
                ));
            } else if input.peek(Punct!["!="]) {
                expr = Expr::binary(Binary::NotEqual(
                    expr,
                    input.parse()?,
                    Expr::comparison(input)?,
                ));
            } else {
                break Ok(expr);
            }
        }
    }

    fn comparison(input: ParseStream<'_>) -> Result<Self> {
        let mut expr = Expr::term(input)?;

        loop {
            if input.peek(Punct![">"]) {
                expr = Expr::binary(Binary::Greater(expr, input.parse()?, Expr::term(input)?));
            } else if input.peek(Punct![">="]) {
                expr = Expr::binary(Binary::GreaterEqual(
                    expr,
                    input.parse()?,
                    Expr::term(input)?,
                ));
            } else if input.peek(Punct!["<"]) {
                expr = Expr::binary(Binary::Less(expr, input.parse()?, Expr::term(input)?));
            } else if input.peek(Punct!["<="]) {
                expr = Expr::binary(Binary::LessEqual(expr, input.parse()?, Expr::term(input)?));
            } else {
                break Ok(expr);
            }
        }
    }

    fn term(input: ParseStream<'_>) -> Result<Self> {
        let mut expr = Expr::factor(input)?;

        loop {
            if input.peek(Punct!["+"]) {
                expr = Expr::binary(Binary::Add(expr, input.parse()?, Expr::factor(input)?));
            } else if input.peek(Punct!["-"]) {
                expr = Expr::binary(Binary::Sub(expr, input.parse()?, Expr::factor(input)?));
            } else {
                break Ok(expr);
            }
        }
    }

    fn factor(input: ParseStream<'_>) -> Result<Self> {
        let mut expr = Expr::unary(input)?;

        loop {
            if input.peek(Punct!["*"]) {
                expr = Expr::binary(Binary::Mul(expr, input.parse()?, Expr::unary(input)?));
            } else if input.peek(Punct!["/"]) {
                expr = Expr::binary(Binary::Div(expr, input.parse()?, Expr::unary(input)?));
            } else {
                break Ok(expr);
            }
        }
    }

    fn unary(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Punct!["-"]) {
            Ok(Expr::Unary(Box::new(Unary::Neg(
                input.parse()?,
                Expr::unary(input)?,
            ))))
        } else if input.peek(Punct!["!"]) {
            Ok(Expr::Unary(Box::new(Unary::Not(
                input.parse()?,
                Expr::unary(input)?,
            ))))
        } else {
            Expr::call(input)
        }
    }

    fn call(input: ParseStream<'_>) -> Result<Self> {
        let mut expr = Expr::primary(input)?;

        loop {
            if input.peek(Punct!["("]) {
                expr = Expr::finish_call(input, expr)?;
            } else if input.peek(Punct!["."]) {
                let _: Punct!["."] = input.parse()?;
                let name: Ident = input.parse()?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break Ok(expr);
            }
        }
    }

    fn finish_call(input: ParseStream<'_>, callee: Expr) -> Result<Self> {
        let mut contents: Group<Parentheses> = input.parse()?;
        contents.remove_whitespace();
        let paren = contents.delimiters();
        let arguments: Punctuated<Expr, Punct![","]> =
            Punctuated::parse_separated_trailing.parse(contents.into_token_stream())?;
        let arguments: Vec<_> = arguments.into_iter().collect();
        if arguments.len() >= 255 {
            input.add_error(input.new_error(
                "Can't have more than 254 arguments".to_string(),
                paren.0.clone(),
                1,
            ));
        }
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn primary(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead();
        if lookahead.peek(kw::keyword_false) {
            Ok(Expr::Literal(Literal::False(input.parse()?)))
        } else if lookahead.peek(kw::keyword_true) {
            Ok(Expr::Literal(Literal::True(input.parse()?)))
        } else if lookahead.peek(kw::keyword_nil) {
            Ok(Expr::Literal(Literal::Nil(input.parse()?)))
        } else if lookahead.peek(LitFloat) {
            Ok(Expr::Literal(Literal::Float(input.parse()?)))
        } else if lookahead.peek(LitInt) {
            Ok(Expr::Literal(Literal::Int(input.parse()?)))
        } else if lookahead.peek(LitStr) {
            Ok(Expr::Literal(Literal::String(input.parse()?)))
        } else if input.peek(kw::keyword_super) {
            Ok(Expr::Super {
                keyword: input.parse()?,
                method: input.parse()?,
            })
        } else if input.peek(kw::keyword_this) {
            Ok(Expr::This(input.parse()?))
        } else if lookahead.peek(Ident) {
            Ok(Expr::Variable(kw::ident(input)?))
        } else if lookahead.peek(Punct!["("]) {
            let group: Group<Parentheses> = input.parse()?;
            parse(group.into_token_stream())
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Expr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Expr::assignment(input)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
struct Function {
    name: Ident,
    params: Vec<Ident>,
    body: Vec<Stmt>,
}

impl Parse for Function {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name: Ident = input.parse()?;
        let mut contents: Group<Parentheses> = input.parse()?;
        contents.remove_whitespace();
        let Parentheses(span) = contents.delimiters();
        let params: Punctuated<Ident, Punct![","]> =
            Punctuated::parse_separated.parse(contents.into_token_stream())?;
        let params: Vec<_> = params.into_iter().collect();
        if params.len() >= 255 {
            input.add_error(input.new_error(
                "Can't have more than 254 parameters".to_string(),
                span,
                1,
            ));
        }
        let mut contents: Group<Braces> = input.parse()?;
        contents.remove_whitespace();
        let body = block.parse(contents.into_token_stream())?;
        Ok(Function { name, params, body })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum Stmt {
    Block(Vec<Stmt>),
    Class {
        name: Ident,
        superclass: Option<Ident>,
        methods: Vec<Function>,
    },
    Expr(Expr),
    Function(Function),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Return {
        keyword: kw::keyword_return,
        value: Expr,
    },
    Variable {
        name: Ident,
        initialiser: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

fn block(input: ParseStream<'_>) -> Result<Vec<Stmt>> {
    let mut statements = vec![];

    while !input.is_empty() {
        statements.push(Stmt::declaration(input)?);
    }

    Ok(statements)
}

impl Stmt {
    fn declaration(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(kw::keyword_class) {
            Stmt::class_declaration(input)
        } else if input.peek(kw::keyword_fun) {
            let _: kw::keyword_fun = input.parse()?;
            Ok(Stmt::Function(Function::parse(input)?))
        } else if input.peek(kw::keyword_var) {
            Stmt::var_declaration(input)
        } else {
            Stmt::statement(input)
        }
    }

    fn class_declaration(input: ParseStream<'_>) -> Result<Self> {
        let _: kw::keyword_class = input.parse()?;
        let name: Ident = input.parse()?;

        let superclass = if input.peek(Punct!["<"]) {
            let _: Punct!["<"] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };

        let mut contents: Group<Braces> = input.parse()?;
        contents.remove_whitespace();
        let methods = parse_repeated.parse(contents.into_token_stream())?;

        Ok(Stmt::Class {
            name,
            superclass,
            methods,
        })
    }

    fn var_declaration(input: ParseStream<'_>) -> Result<Self> {
        let _: kw::keyword_var = input.parse()?;

        let name = kw::ident(input)?;

        let initialiser = if input.peek(Punct!["="]) {
            let _: Punct!["="] = input.parse()?;
            input.parse()?
        } else {
            Expr::Literal(Literal::Nil(kw::keyword_nil::new(input)))
        };

        let _: Punct![";"] = input.parse()?;
        Ok(Stmt::Variable { name, initialiser })
    }

    fn statement(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(kw::keyword_if) {
            Stmt::if_statement(input)
        } else if input.peek(kw::keyword_for) {
            Stmt::for_statement(input)
        } else if input.peek(kw::keyword_print) {
            Stmt::print_statement(input)
        } else if input.peek(kw::keyword_return) {
            Stmt::return_statement(input)
        } else if input.peek(kw::keyword_while) {
            Stmt::while_statement(input)
        } else if input.peek(Punct!["{"]) {
            let mut group: Group<Braces> = input.parse()?;
            group.remove_whitespace();
            Ok(Stmt::Block(block.parse(group.into_token_stream())?))
        } else {
            Stmt::expression_statement(input)
        }
    }

    fn for_statement(input: ParseStream<'_>) -> Result<Self> {
        struct ForInner(Option<Stmt>, Expr, Option<Expr>);

        impl Parse for ForInner {
            fn parse(input: ParseStream<'_>) -> Result<Self> {
                let initialiser = if input.peek(Punct![";"]) {
                    let _: Punct![";"] = input.parse()?;
                    None
                } else if input.peek(kw::keyword_var) {
                    Some(Stmt::var_declaration(input)?)
                } else {
                    Some(Stmt::expression_statement(input)?)
                };

                let condition = if input.peek(Punct![";"]) {
                    Expr::Literal(Literal::True(kw::keyword_true::new(input)))
                } else {
                    Expr::parse(input)?
                };
                let _: Punct![";"] = input.parse()?;

                let increment = if input.is_empty() {
                    None
                } else {
                    Some(Expr::parse(input)?)
                };

                Ok(ForInner(initialiser, condition, increment))
            }
        }

        let _: kw::keyword_for = input.parse()?;

        let mut inner: Group<Parentheses> = input.parse()?;
        inner.remove_whitespace();
        let ForInner(initialiser, condition, increment) = parse(inner.into_token_stream())?;

        let mut body = Stmt::statement(input)?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expr(increment)]);
        }

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initialiser) = initialiser {
            body = Stmt::Block(vec![initialiser, body]);
        }

        Ok(body)
    }

    fn if_statement(input: ParseStream<'_>) -> Result<Self> {
        let _: kw::keyword_if = input.parse()?;
        let condition: Group<Parentheses> = input.parse()?;
        let condition = parse(condition.into_token_stream())?;

        let then_branch = Box::new(Stmt::statement(input)?);
        let else_branch = if input.peek(kw::keyword_else) {
            Some(Box::new(Stmt::statement(input)?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(input: ParseStream<'_>) -> Result<Self> {
        let _: kw::keyword_print = input.parse()?;
        let value = input.parse()?;
        let _: Punct![";"] = input.parse()?;
        Ok(Self::Print(value))
    }

    fn return_statement(input: ParseStream<'_>) -> Result<Self> {
        let keyword: kw::keyword_return = input.parse()?;
        let value = if input.peek(Punct![";"]) {
            Expr::Literal(Literal::Nil(kw::keyword_nil::new(input)))
        } else {
            input.parse()?
        };
        let _: Punct![";"] = input.parse()?;
        Ok(Stmt::Return { keyword, value })
    }

    fn while_statement(input: ParseStream<'_>) -> Result<Self> {
        let _: kw::keyword_while = input.parse()?;
        let condition: Group<Parentheses> = input.parse()?;
        let condition = parse(condition.into_token_stream())?;
        let body = Box::new(Stmt::statement(input)?);

        Ok(Stmt::While { condition, body })
    }

    fn expression_statement(input: ParseStream<'_>) -> Result<Self> {
        let expr = input.parse()?;
        let _: Punct![";"] = input.parse()?;
        Ok(Self::Expr(expr))
    }
}

impl Parse for Stmt {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Stmt::declaration(input)
    }
}

struct Ast(Vec<Stmt>);

impl Ast {
    fn synchronise(input: ParseStream<'_>) {
        input.synchronise(|input| {
            use kw::*;
            input.peek(Punct![";"])
                || peek2_any!(
                    input,
                    keyword_class,
                    keyword_for,
                    keyword_fun,
                    keyword_if,
                    keyword_print,
                    keyword_return,
                    keyword_var,
                    keyword_while
                )
        });
    }
}

impl Parse for Ast {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut stmts = vec![];
        let mut errors = vec![];

        while !input.is_empty() {
            match input.parse() {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => {
                    Ast::synchronise(input);
                    errors.push(err);
                }
            }
        }

        if let Some(error) = errors.into_iter().reduce(|acc, err| acc.with(err)) {
            Err(error)
        } else {
            Ok(Ast(stmts))
        }
    }
}

fn main() {
    let _ast: Ast = pretty_unwrap(parse_string(
        "
        var x = 5;
        var y = \"hello\";
        x = 6;
        y = 4.0;

        fun add(x, y) {
            return x + y;
        }
    "
        .to_string(),
    ));
}
