use std::cmp::PartialEq;
use std::collections::VecDeque;
use std::error::Error;

use crate::console::ConsoleManager;
use crate::term::*;


struct Parser {
    terms: VecDeque<Term>,
    mode: ParseMode,
    last_priority: i32,
    paren: i32,
    builder:Vec<char>,
    equation: Equation,
}

impl Parser {
    fn new() -> Self {
        Self{
            terms: VecDeque::new(),
            mode: ParseMode::None,
            last_priority:-1,
            paren:0,
            builder:Vec::new(),
            equation: Equation(None,None)
        }
    }

    fn print(&mut self,console:&mut ConsoleManager){
        console.print("Terms: ");
        console.println(format!("{}",self.terms.len()));
        self.terms.iter().for_each(|t| console.println(t.to_string()));
    }
    fn builder_end(&mut self)-> Result<(),Box<dyn Error>>{
        match self.mode {
            ParseMode::Number => {
                let num_str = self.builder.iter().rev().collect::<String>();
                if num_str.len() > 1 && num_str.starts_with("0") && num_str.get(1..=1).ne(&Some(".")) {
                    return Err("Invalid Number! Cant start with 0 !".into());
                }
                if num_str.matches("\\.").count() > 1 {
                    return Err("Invalid Number! Multiple Decimals!".into());
                }
                let number = num_str.parse::<Num>()?;
                self.builder.clear();
                self.mode = ParseMode::None;
                self.terms.push_back(Term::Number(number));
            }
            ParseMode::Literal => {
                let literal = self.builder.iter().rev().collect::<String>();
                self.builder.clear();
                self.mode = ParseMode::None;
                self.terms.push_back(Term::Unknown(literal));
            }
            _ => {}
        }
        Ok(())
    }
    fn end(&mut self)-> Result<(),Box<dyn Error>>{
        self.builder_end()?;
        if self.paren != 0 {
            return Err("Parentheses not closed!".into());
        }
        let mut count = 0;
        let len = self.terms.len();
        while self.terms.len() >= 2 {
            count += 1;
            let mut term = self.terms.pop_back().ok_or("No Terms Found!")?;
            let complex =  self.terms.back_mut().ok_or("No Chain / Power Found!")?;
            match (&mut term,complex) {
                (Term::Chain(com_pop,terms_pop),Term::Chain(com_last,terms_last)) =>
                    match (com_last,com_pop) {
                        (Commutative::Multiply,Commutative::Multiply) |
                        (Commutative::Add,Commutative::Add) => terms_last.append(terms_pop),
                        (Commutative::Multiply,Commutative::Add) =>
                            if terms_pop.len() == 1 {
                                terms_last.push_back(terms_pop.pop_back().unwrap());
                            } else {
                                terms_last.push_back(term);
                            }
                        (Commutative::Add,Commutative::Multiply) => { self.terms.push_front(term);count += 1; },
                    }
                (_,Term::Power(base,_)) => {
                    if base.is_some() {
                        return Err("Power already has a base!".into());
                    }
                    *base = Some(term.into());
                }
                (_,Term::Chain(_,terms)) => {
                    terms.push_back(term);
                }
                _ => return Err("Invalid Expr!".into())
            }
            if count >= len { break; }
        }
        if self.terms.len() > 1 {
            let mut deque = VecDeque::new();
            deque.append(&mut self.terms);
            self.terms.push_back(Term::Chain(Commutative::Add,deque));
        }
        match self.equation {
            Equation(_,None) =>  self.equation.1 = Some(self.terms.pop_back().ok_or("No Terms Found!")?),
            _ => self.equation.0 = Some(self.terms.pop_back().ok_or("No Terms Found!")?),
        }
        Ok(())
    }

    fn parse(&mut self, c:char) -> Result<(),Box<dyn Error>> {
        match c {
            'A'..='Z'| 'a'..='z' => match self.mode {
                ParseMode::None | ParseMode::Literal => {
                    self.mode = ParseMode::Literal;
                    self.builder.push(c);
                    return Ok(());
                }
                _ => self.builder_end()?
            }
            '0'..='9' => match self.mode {
                ParseMode::None | ParseMode::Number => {
                    self.mode = ParseMode::Number;
                    self.builder.push(c);
                    return Ok(());
                }
                _ => self.builder_end()?
            }
            '.' => match self.mode {
                ParseMode::Number => {
                    self.builder.push(c);
                    return Ok(());
                }
                _ => self.builder_end()?
            }
            _ => self.builder_end()?
        }

        match c {
            '=' => {
                if let Equation(_,Some(_)) = self.equation {
                    return Err("Equation already has two sides!".into());
                }
                self.last_priority = 0;
                self.end()?
            }
            '+' | '-' | '*' | '/' | '^' => {
                let oper = Operator::from_char(c).ok_or("Invalid Operator!")?;
                let priority = oper.priority();
                if priority <= self.last_priority {
                    let same = priority == self.last_priority;
                    let last = self.terms.pop_back().ok_or("No Terms Found!")?;
                    let last = if same { oper.to_com_term(last) } else { last };
                    match self.terms.back_mut().ok_or("No Chain / Power Found!")? {
                        Term::Chain(_, ref mut terms) =>  terms.push_back(last),
                        Term::Power(base,_) => {
                            *base = Some(last.into());
                        }
                        _ => return Err("Invalid Chain / Power!".into())
                    }
                    if same {
                        return Ok(());
                    }
                }
                let last = self.terms.pop_back().ok_or("No Terms Found!")?;
                let term = match oper {
                    Operator::Power => Term::Power(None,last.into()),
                    _ => {
                        let mut deque = VecDeque::new();
                        deque.push_back(oper.to_com_term(last));
                        Term::Chain(oper.to_com(), deque)
                    }
                };
                self.terms.push_back(term);
                self.last_priority = priority;
            },

            '(' => {
                self.paren -= 1;
                self.last_priority = 999;
            }

            ')' => {
                self.paren += 1;
                self.last_priority = 0;
            }
            ' ' => {}

            _ => {
                return Err(format!("Invalid Character '{}'",c).into());
            }
        }

        return Ok(());
    }
}

#[derive(PartialEq)]
enum ParseMode {
    Number,
    Literal,
    None
}

pub fn parse(s:&str) -> Result<Equation,Box<dyn Error>>{
    let mut chars = s.chars().into_iter().rev();
    let mut parser = Parser::new();
    for char in chars {
        parser.parse(char)?;
    }
    parser.end()?;
    return Ok(parser.equation);
}