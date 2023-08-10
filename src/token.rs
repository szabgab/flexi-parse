use crate::error::Error;
use crate::error::ErrorKind;
use crate::private::Sealed;
use crate::Parse;
use crate::ParseStream;
use crate::Result;
use crate::Span;
use crate::TokenTree;

use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;
use std::result;

pub trait Token: Parse + Sealed {}

impl<T: Token> Parse for Option<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        match input.try_parse() {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }
}

pub trait Punct: Token {
    #[doc(hidden)]
    fn display() -> String;
}

pub trait WhiteSpace: Punct {}

#[derive(Debug, Clone)]
pub struct Ident {
    pub(crate) string: String,
    pub(crate) span: Span,
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
    }
}

impl TryFrom<TokenTree> for Ident {
    type Error = TokenTree;

    fn try_from(value: TokenTree) -> result::Result<Self, Self::Error> {
        if let TokenTree::Ident(token) = value {
            Ok(token)
        } else {
            Err(value)
        }
    }
}

impl Parse for Ident {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(ident) = Self::try_from(input.next()?.to_owned()) {
            Ok(ident)
        } else {
            todo!()
        }
    }
}

impl Sealed for Ident {}

impl Token for Ident {}

#[derive(Debug, Clone)]
pub(crate) struct SingleCharPunct {
    pub(super) kind: PunctKind,
    pub(super) spacing: Spacing,
    pub(super) span: Span,
}

impl PartialEq for SingleCharPunct {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.spacing == other.spacing
    }
}

impl<'a> TryFrom<&'a TokenTree> for &'a SingleCharPunct {
    type Error = ();

    fn try_from(value: &'a TokenTree) -> result::Result<Self, Self::Error> {
        if let TokenTree::Punct(token) = value {
            Ok(token)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Spacing {
    Alone,
    Joint,
}

impl From<char> for Spacing {
    fn from(value: char) -> Self {
        if PunctKind::try_from(value).is_ok() {
            Spacing::Joint
        } else {
            Spacing::Alone
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Literal {
    pub(crate) value: LiteralValue,
    pub(crate) span: Span,
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<'a> TryFrom<&'a TokenTree> for &'a Literal {
    type Error = &'a TokenTree;

    fn try_from(value: &'a TokenTree) -> result::Result<Self, Self::Error> {
        if let TokenTree::Literal(token) = value {
            Ok(token)
        } else {
            Err(value)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
}

macro_rules! tokens {
    {
        {
            $( ($t1:ident, $name1:literal, $doc1:literal) )+
        }
        {
            $( ($t2:ident, $t21:ident, $t22:ident, $name2:literal, $doc2:literal) )+
        }
        {
            $( ($t3:ident, $t31:ident, $t32:ident, $t33:ident, $name3:literal, $doc3:literal) )+
        }
    } => {
        $(
            #[derive(Debug, Clone)]
            #[doc = $doc1]
            pub struct $t1(Span);

            impl $t1 {
                pub fn span(&self) -> &Span {
                    &self.0
                }
            }

            impl Parse for $t1 {
                fn parse(input: ParseStream<'_>) -> Result<Self> {
                    let token = input.next()?.to_owned();
                    if let TokenTree::Punct(SingleCharPunct { kind: PunctKind::$t1, span, .. }) = token {
                        Ok(Self(span))
                    } else {
                        Err(Error::new(Rc::clone(&input.source), ErrorKind::UnexpectedToken {
                            expected: HashSet::from_iter(vec![$name1.to_string()]),
                            span: token.span().clone(),
                        }))
                    }
                }
            }

            impl Sealed for $t1 {}

            impl Token for $t1 {}

            impl fmt::Display for $t1 {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", $name1)
                }
            }

            impl Punct for $t1 {
                fn display() -> String {
                    $name1.to_string()
                }
            }
        )+

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum PunctKind {
            $( $t1 ),+
        }

        impl TryFrom<char> for PunctKind {
            type Error = char;

            fn try_from(value: char) -> result::Result<Self, Self::Error> {
                match value {
                    $( $name1 => Ok(PunctKind::$t1), )+
                    _ => Err(value),
                }
            }
        }

        $(
            #[derive(Debug, Clone)]
            #[doc = $doc2]
            pub struct $t2(Span);

            impl $t2 {
                pub fn span(&self) -> &Span {
                    &self.0
                }

                fn from_tokens_impl(input: ParseStream<'_>) -> Result<Self> {
                    if let TokenTree::Punct(SingleCharPunct { spacing: Spacing::Joint, .. }) = input.current()? {

                    } else {
                        return Err(Error::empty());
                    }

                    let start: $t21 = input.parse()?;
                    let end: $t22 = input.parse()?;
                    Ok(Self(Span::new(start.0.start, end.0.end, start.0.source)))
                }
            }

            impl Parse for $t2 {
                fn parse(input: ParseStream<'_>) -> Result<Self> {
                    let span = input.current()?.span();
                    Self::from_tokens_impl(input).map_err(|_| {
                        Error::new(Rc::clone(&input.source), ErrorKind::UnexpectedToken {
                            expected: HashSet::from_iter(vec![$name2.to_string()]),
                            span: span.clone(),
                        })
                    })
                }
            }

            impl Sealed for $t2 {}

            impl Token for $t2 {}

            impl fmt::Display for $t2 {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, $name2)
                }
            }

            impl Punct for $t2 {
                fn display() -> String {
                    $name2.to_string()
                }
            }
        )+

        $(
            #[derive(Debug, Clone)]
            #[doc = $doc3]
            pub struct $t3(Span);

            impl $t3 {
                pub fn span(&self) -> &Span {
                    &self.0
                }

                fn from_tokens_impl(input: ParseStream<'_>) -> Result<Self> {
                    if let TokenTree::Punct(SingleCharPunct { spacing: Spacing::Joint, .. }) = input.current()? {

                    } else {
                        return Err(Error::empty());
                    }
                    if let TokenTree::Punct(SingleCharPunct { spacing: Spacing::Joint, .. }) = input.current()? {

                    } else {
                        return Err(Error::empty());
                    }
                    let p1: $t31 = input.parse()?;
                    let _p2: $t32 = input.parse()?;
                    let p3: $t33 = input.parse()?;
                    Ok(Self(Span::new(p1.0.start, p3.0.end, p1.0.source)))
                }
            }

            impl Parse for $t3 {
                fn parse(input: ParseStream<'_>) -> Result<Self> {
                    let span = input.current()?.span();
                    Self::from_tokens_impl(input).map_err(|_| {
                        Error::new(Rc::clone(&input.source), ErrorKind::UnexpectedToken {
                            expected: HashSet::from_iter(vec![$name3.to_string()]),
                            span: span.clone(),
                        })
                    })
                }
            }

            impl Sealed for $t3 {}

            impl Token for $t3 {}

            impl fmt::Display for $t3 {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, $name3)
                }
            }

            impl Punct for $t3 {
                fn display() -> String {
                    $name3.to_string()
                }
            }
        )+
    };
}

tokens! {
    {
        (Bang, '!', "`!`")
        (Colon, ':', "`:`")
        (Equal, '=', "`=`")
        (SemiColon, ';', "`;`")
        (LAngle, '<', "`<`")
        (RAngle, '>', "`>`")
        (Plus, '+', "`+`")
        (Dash, '-', "`-`")
        (Asterisk, '*', "`*`")
        (Slash, '/', "`/`")
        (Percent, '%', "`%`")
        (Dot, '.', "`.`")
        (Comma, ',', "`,`")
        (LeftParen, '(', "`(`")
        (RightParen, ')', "`)`")
        (LeftBracket, '[', "`[`")
        (RightBracket, ']', "`]`")
        (LeftBrace, '{', "`{`")
        (RightBrace, '}', "`}`")
        (At, '@', "`@`")
        (Caret, '^', "`^`")
        (BackTick, '`', "`` ` ``")
        (Pipe, '|', "`|`")
        (Ampersand, '&', "`&`")
        (Tilde, '~', "`~`")
        (Tilde2, '¬', "`¬`")
        (Backslash, '\\', "`\\`")
        (Question, '?', "`?`")
        (Hash, '#', "`#`")
        (Pound, '£', "`£`")
        (Dollar, '$', "`$`")
        (Space, ' ', "` `")
        (UnderScore, '_', "`_`")
        (NewLine, '\n', "`\\n`")
    }
    {
        (BangEqual, Bang, Equal, "!=", "`!=`")
        (GreaterEqual, RAngle, Equal, ">=", "`>=`")
        (LessEqual, LAngle, Equal, "<=", "`<=`")
        (PlusEqual, Plus, Equal, "+=", "`+=`")
        (DashEqual, Dash, Equal, "-=", "`-=`")
        (AsteriskEqual, Asterisk, Equal, "*=", "`*=`")
        (SlashEqual, Slash, Equal, "/=", "`/=`")
        (PercentEqual, Percent, Equal, "%=", "`%=`")
        (LAngleLAngle, LAngle, LAngle, "<<", "`<<`")
        (RAngleRAngle, RAngle, RAngle, ">>", "`>>`")
        (LThinArrow, LAngle, Dash, "<-", "`<-`")
        (RThinArrow, Dash, RAngle, "->", "`->`")
        (FatArrow, Equal, RAngle, "=>", "`=>`")
        (SlashSlash, Slash, Slash, "//", "`//`")
        (ColonColon, Colon, Colon, "::", "`::`")
        (HashHash, Hash, Hash, "##", "`##`")
        (LogicalAnd, Ampersand, Ampersand, "&&", "`&&`")
        (LogicalOr, Pipe, Pipe, "||", "`||`")
    }
    {
        (HashHashHash, Hash, Hash, Hash, "###", "`###`")
        (SlashSlashEqual, Slash, Slash, Equal, "//=", "`//=`")
        (LAngleLAngleEqual, LAngle, LAngle, Equal, "<<=", "`<<=`")
        (RAngleRAngleEqual, RAngle, RAngle, Equal, ">>=", "`>>=`")
        (ColonColonEqual, Colon, Colon, Equal, "::=", "`::=`")
    }
}

impl WhiteSpace for Space {}

fn parse_joint_impl<T1: Punct, T2: Punct>(input: ParseStream<'_>) -> Result<(T1, T2)> {
    let t1 = input.parse()?;
    if let TokenTree::Punct(SingleCharPunct {
        spacing: Spacing::Joint,
        ..
    }) = input.get(-1)?
    {
    } else {
        return Err(Error::empty());
    }
    let t2 = input.parse()?;
    Ok((t1, t2))
}

impl<T1: Punct, T2: Punct> Parse for (T1, T2) {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let span = input.current()?.span();
        parse_joint_impl(input).map_err(|_| {
            Error::new(
                Rc::clone(&input.source),
                ErrorKind::UnexpectedToken {
                    expected: HashSet::from_iter(vec![<(T1, T2)>::display()]),
                    span: span.clone(),
                },
            )
        })
    }
}

impl<T1: Punct, T2: Punct> Sealed for (T1, T2) {}

impl<T1: Punct, T2: Punct> Token for (T1, T2) {}

impl<T1: Punct, T2: Punct> Punct for (T1, T2) {
    fn display() -> String {
        T1::display() + &T2::display()
    }
}

/// `  `
#[derive(Debug, Clone)]
pub struct Space2(Span);

impl Parse for Space2 {
    fn parse(input: ParseStream) -> Result<Self> {
        let (s1, s2): (Space, Space) = input.parse()?;
        Ok(Space2(Span::new(
            s1.span().start,
            s2.span().end,
            Rc::clone(&input.source),
        )))
    }
}

impl Sealed for Space2 {}

impl Token for Space2 {}

impl Punct for Space2 {
    fn display() -> String {
        "  ".to_string()
    }
}

impl WhiteSpace for Space2 {}

/// `    `
#[derive(Debug, Clone)]
pub struct Space4(Span);

impl Parse for Space4 {
    fn parse(input: ParseStream) -> Result<Self> {
        let (s1, s2): (Space2, Space2) = input.parse()?;
        Ok(Space4(Span::new(
            s1.0.start,
            s2.0.end,
            Rc::clone(&input.source),
        )))
    }
}

impl Sealed for Space4 {}

impl Token for Space4 {}

impl Punct for Space4 {
    fn display() -> String {
        "    ".to_string()
    }
}

impl WhiteSpace for Space4 {}
