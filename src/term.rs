use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use rust_decimal::Decimal;

pub type Num = Decimal;
pub struct Equation(pub Option<Term>,pub Option<Term>);

impl Equation {
    pub fn to_string(&self) -> String {
        let left = self.0.as_ref().map(|it| it.to_string()).unwrap_or("".to_string());
        let right = self.1.as_ref().map(|it| it.to_string()).unwrap_or("".to_string());
        format!("{} = {}",left,right)
    }
}
#[derive(PartialEq,Eq,Clone)]
pub enum Commutative{
    Add,
    Multiply,
}

impl Commutative{
    pub fn to_string(&self) -> String {
        match self {
            Self::Add => "+".to_string(),
            Self::Multiply => "*".to_string(),
        }
    }
}
#[derive(PartialEq,Eq,Clone)]
pub enum Term {
    Number(Num),
    Chain(Commutative, VecDeque<Term>),
    Power(Option<Box<Term>>,Box<Term>),
    MulInverse(Box<Term>),
    Opposite(Box<Term>),
    Unknown(String)
}

impl Display for Term{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.to_string())
    }
}
impl Debug for Term{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.to_string())
    }
}
impl Term {
    pub fn to_string(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::Chain(com, terms) => format!("{} [ {} ]",com.to_string(),terms.iter().map(|it| it.to_string()).collect::<Vec<String>>().join(" , ")),
            Self::MulInverse(term) => format!("1/{}",term.to_string()),
            Self::Opposite(term) => format!("-{}",term.to_string()),
            Self::Unknown(c) => c.to_string(),
            Self::Power(base,exp) => {
                let base = base.as_ref().map(|it| it.to_string()).unwrap_or("none".to_string());
                let exp = exp.to_string();
                format!("{}^{}",base,exp)
            }
        }
    }
}

#[derive(PartialEq,Eq)]
pub enum Operator{
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Equals,
}

impl Operator{

    pub fn com(&self) -> bool {
        match self {
            Self::Add | Self::Multiply => true,
            _ => false,
        }
    }

    pub fn to_com(&self) -> Commutative {
        match self {
            Self::Add => Commutative::Add,
            Self::Subtract => Commutative::Add,
            Self::Multiply => Commutative::Multiply,
            Self::Divide => Commutative::Multiply,
            _ => panic!("Invalid Operator!"),
        }
    }
    pub fn to_com_term(&self, term:Term) -> Term {
        match self {
            Self::Subtract => Term::Opposite(term.into()),
            Self::Divide => Term::MulInverse(term.into()),
            _ => term,
        }
    }

    pub(crate) fn priority(&self) -> i32{
        match self {
            Self::Equals => 0,
            Self::Add => 1,
            Self::Subtract => 1,
            Self::Multiply => 2,
            Self::Divide => 2,
            Self::Power => 3,
        }
    }
    fn to_string(&self) -> String{
        match self {
            Self::Add => "+".to_string(),
            Self::Subtract => "-".to_string(),
            Self::Multiply => "*".to_string(),
            Self::Divide => "/".to_string(),
            Self::Power => "^".to_string(),
            Self::Equals => "=".to_string(),
        }
    }
}
impl From<char> for Operator{
    fn from(c:char) -> Self{
        match c {
            '+' => Self::Add,
            '-' => Self::Subtract,
            '*' => Self::Multiply,
            '/' => Self::Divide,
            '^' => Self::Power,
            '=' => Self::Equals,
            _ => panic!("Invalid Operator {c}!"),
        }
    }
}
impl Operator{
    pub(crate) fn from_char(c:char) -> Option<Self>{
        match c {
            '+' => Some(Self::Add),
            '-' => Some(Self::Subtract),
            '*' => Some(Self::Multiply),
            '/' => Some(Self::Divide),
            '^' => Some(Self::Power),
            '=' => Some(Self::Equals),
            _ => None,
        }
    }
}