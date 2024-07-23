use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::ops::{Div, Neg};
use rust_decimal::MathematicalOps;
use rust_decimal_macros::dec;
use crate::console::ConsoleManager;
use crate::term::{Commutative, Equation,  Term};


pub struct Evaluator {
    pub(crate) formula:Term,
}

impl Evaluator {
    fn trans_pos(mut equation:Equation) -> Result<Term,Box<dyn Error>>{
        match equation {
            Equation(Some(mut left), Some(right)) => {
                match left {
                    Term::Chain(Commutative::Add,ref mut terms) => terms.push_back(Term::Opposite(right.into())),
                    _ => {
                        let mut deque = VecDeque::new();
                        deque.push_back(left);
                        deque.push_back(Term::Opposite(right.into()));
                        left = Term::Chain(Commutative::Add,deque);
                    },
                };
                Ok(left)
            }
            Equation(None, Some(right)) => Ok(right),
            _ => Err("Invalid Equation".into())
        }
    }
    fn process(equation:Equation) -> Result<Term,Box<dyn Error>>{
        let mut left = Self::trans_pos(equation)?;
        Self::const_expr(&mut left, None)?;
        return Ok(left);
    }
    pub fn new(equation:Equation) -> Result<Self,Box<dyn Error>> {
        Ok(Self {
            formula: Self::process(equation)?
        })
    }

    fn get(name:&str,context:&HashMap<String,Term>) -> Result<Term,Box<dyn Error>> {
        Ok(context.get(name).ok_or(format!("Unknown variable {}",name))?.clone())
    }

    fn const_expr(term:&mut Term, context:Option<&HashMap<String,Term>>) -> Result<Option<Term>,Box<dyn Error>> {
        match term {
            Term::Chain(com,terms) => {
                for term in terms.iter_mut() {
                    Self::const_expr(term, context)?;
                }
                let mut result = match com {
                    Commutative::Add => dec!(0),
                    Commutative::Multiply => dec!(1)
                };
                let mut count = 0;
                let len = terms.len();
                while let Some(term) = terms.pop_front() {
                    count += 1;
                    match term {
                        Term::Number(n) => {
                            match com {
                                Commutative::Add => result += n,
                                Commutative::Multiply => result *= n,
                            }
                        },
                        _ => terms.push_back(term)
                    }
                    if count >= len { break  }
                }
                if terms.is_empty() {
                    *term = Term::Number(result);
                    if context.is_some(){ return Ok(Some(Term::Number(result))); }
                } else if result != dec!(0) {
                    terms.push_back(Term::Number(result));
                }
            }
            Term::Power(Some( base),exp) => {
                Self::const_expr(base, context)?;
                Self::const_expr(exp, context)?;
                match (base.as_ref(),exp.as_ref()) {
                    (Term::Number(base),Term::Number(exp)) => {
                        *term = Term::Number(base.powd(*exp));
                    },
                    _ => {}
                }
            }
            Term::Opposite(b) => {
                Self::const_expr(b, context)?;
                match b.as_ref() {
                    Term::Number(n) => *term = Term::Number(n.neg()),
                    _ => {}
                }
            }
            Term::MulInverse(b) => {
                Self::const_expr(b, context)?;
                match b.as_ref() {
                    Term::Number(n) => *term = Term::Number(dec!(1).div(n)),
                    _ => {}
                }
            }
            Term::Unknown(name) => {
                if let Some(context) = context{
                    let Ok(num) = Self::get(name,context) else{ return Ok(None) };
                    *term = num;
                }
            }
            _ => {}
        }

        if context.is_none() { Ok(None) }
        else {
            return Ok(Some(term.clone()));
        }
    }

    pub fn inline(&mut self,context:&HashMap<String,Term>) -> Result<(),Box<dyn Error>> {
        Self::const_expr(&mut self.formula, Some(context))?;
        Ok(())
    }
    pub fn eval(&self,context:&HashMap<String,Term>) -> Result<Term,Box<dyn Error>> {
        let mut formula = self.formula.clone();
        Self::const_expr(&mut formula, Some(context))?.ok_or(format!("Failed to eval with {:?}", context).into())
    }

    /**
    fn get_unknown_expr(term:&Term) -> Result<Num,Box<dyn Error>> {
        let ok = match term {
            Term::Unknown(_) => dec!(1),
            Term::Opposite(b) => Self::get_unknown_expr(b)?,
            Term::MulInverse(b) => Self::get_unknown_expr(b)?.neg(),
            Term::Power(Some(base),exp) => {
                let base = Self::get_unknown_expr(base)?;
                let exp = Self::get_unknown_expr(exp)?;
                base * exp
            }
            Term::Chain(Commutative::Multiply,terms) => {
                let mut result = dec!(0);
                for term in terms {
                    result += Self::get_unknown_expr(term);
                }
                result
            }
            Term::Chain(Commutative::Add,_) => return Err("Unexpected Add Chain!".into()),
                _ => Num::Number(dec!(0))
        };
        Ok(ok)
    }

    fn not_contains_unknown(term:&Term) -> bool {
        match term {
            Term::Unknown(_) => false,
            Term::Opposite(b) => Self::not_contains_unknown(b),
            Term::MulInverse(b) => Self::not_contains_unknown(b),
            Term::Power(Some(base),exp) => Self::not_contains_unknown(base) && Self::not_contains_unknown(exp),
            Term::Chain(Commutative::Add,terms) => terms.iter().all(Self::not_contains_unknown),
            Term::Chain(Commutative::Multiply,terms) => terms.iter().all(Self::not_contains_unknown),
            _ => true
        }
    }

    fn combine(term:&mut Term, console: &mut ConsoleManager) -> Result<(),Box<dyn Error>> {
        match term {
            Term::Chain(Commutative::Add,terms) => {
                let mut unknowns:HashMap<Num,Term> = HashMap::new();
                for term in terms {
                    match term {
                        Term::Chain(Commutative::Add,terms_sub) => {
                            terms.append(terms_sub);
                        }
                        _ => {
                            let exp = Self::get_unknown_expr(term)?;
                            let Term::Chain(_,deque) = unknowns.entry(exp).or_insert(Term::Chain(Commutative::Multiply,VecDeque::new())) else { return Err("Invalid Term!".into()) };

                        }
                    }
                }
            }
        }
        Ok(())
    }
     */

    pub fn print(&mut self,console:&mut ConsoleManager) {
        console.println(self.formula.to_string())
    }

}

pub struct UnknownEvaluator {
    pub(crate) unknown:String,
    pub(crate) evaluator: Evaluator
}

impl UnknownEvaluator{
    fn process(equation:Equation) -> Result<(String,Term),Box<dyn Error>>{
        match equation {
            Equation(Some(Term::Unknown(name)), Some(mut right)) => {
                Evaluator::const_expr(&mut right, None)?;
                Ok((name,right))
            },
            _ => Err("Invalid Unknown Equation".into())
        }
    }
    pub fn new(equation:Equation) -> Result<Self,Box<dyn Error>> {
        let (unknown,formula) = Self::process(equation)?;
        Ok(Self {
            unknown,
            evaluator:Evaluator{ formula}
        })
    }

    pub fn inline(&mut self,context:&HashMap<String,Term>) -> Result<(),Box<dyn Error>> {
        self.evaluator.inline(context)?;
        Ok(())
    }

    pub fn eval(&self, context:&HashMap<String,Term>) -> Result<Term,Box<dyn Error>> {
        return self.evaluator.eval(context);
    }

    pub fn print(&mut self,console: &mut ConsoleManager) {
        console.print(format!("{} = ",self.unknown));
        self.evaluator.print(console)
    }

}
