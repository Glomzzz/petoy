mod console;
mod eval;
mod term;
mod parser;

use std::collections::HashMap;
use std::error::Error;
use crate::console::ConsoleManager;
use crate::eval::{Evaluator, UnknownEvaluator};

fn main() {
    let mut console = ConsoleManager::new();
    console.println("Welcome to Glom's Calculator!");
    while let Err(err) = start(&mut console) {
        console.println(&format!("Error: {}", err));
    }
}

fn start(console: &mut ConsoleManager) -> Result<(),Box<dyn Error>>{
    loop {
        let input = console.input()?;
        if input == "exit" {
            break;
        }
        console.println("Formula:");
        console.println(&input);
        let eq = parser::parse(&input)?;
        console.println(eq.to_string());
        console.println("Compile...");
        let mut evaluator = Evaluator::new(eq)?;
        evaluator.print(console);
        console.println("As you will:");
        let mut context = HashMap::new();
        while let Ok(input) = console.input() {
            match input.as_str() {
                "inline" => {
                    evaluator.inline(&context)?;
                    context.clear();
                    evaluator.print(console);
                    console.println("As you will:");
                    continue
                }
                "" => break,
                _ => {}
            }
            let unknown_eq = parser::parse(&input)?;
            match UnknownEvaluator::new(unknown_eq) {
                Ok(mut unknown_eq) => {
                    let origin = unknown_eq.evaluator.formula.to_string();
                    unknown_eq.inline(&context)?;
                    let current = unknown_eq.evaluator.formula.to_string();
                    console.println(format!("{} = {} := {}",unknown_eq.unknown,origin,current));
                    context.insert(unknown_eq.unknown, unknown_eq.evaluator.formula);
                }
                Err(err) => {
                    console.println(&format!("Error: {}", err));
                }
            }
        }
        console.println("Result:");
        console.println(evaluator.eval(&context)?.to_string());
    }
    Ok(())
}
