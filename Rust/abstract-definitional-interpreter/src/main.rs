use fixpoint_monad::{Cachable, MakeKey, Memo, State};
use crate::ast::Expr;
use crate::eval::eval;


mod ast;
mod eval;

#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Value {
    Boolean(bool),
    Number(u64),
    Function {
        args: Vec<String>,
        body: Expr,
    }
}

impl MakeKey for Value {
    fn show(&self) -> String {
        format!("{}", self)
    }
}

impl Cachable for Value {}

impl Default for Value {
    fn default() -> Self {
        Value::Number(0)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::Function { args, body } => {
                write!(f, "(λ{}. {})", args.join(" "), body)
            }
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

fn run<'a>(expr: Expr) -> State<'a, Value> {
    let sigma = Memo::sigma();
    eval(sigma).call(expr).run()
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter a lambda calculus expression with let and conditionals\n(ex: `\\x.x 3`, `(λx. x λy. y) 3`, `if x then 3 else 4`, `let x = 4 in x`, let x = λy. y in (x 3)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let expr = Expr::parse(&input)?;
    println!("{}\n", expr);

    let state = run(expr);
    println!("{}", state);

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    fn check_run(expr: Expr, expected: Option<Value>) -> Result<String, String> {
        let key = expr.make_key("eval");
        let state = run(expr);

        if let Some(expected) = expected {
            if state.contains_value(&key, &expected) {
                Ok(format!("{state}"))
            } else {
                Err(format!("{state}"))
            }
        } else {
            if state.key_empty(&key) {
                Ok(format!("{state}"))
            } else {
                Err(format!("{state}"))
            }
        }
    }

    fn check_run_string(expr: &str, expected: Option<Value>) -> Result<String, Box<dyn Error>> {
        let expr = Expr::parse(expr)?;
        let result = check_run(expr, expected)?;
        Ok(result)
    }

    #[test]
    fn let_binding() {
        let input = "let x = 5 in x";
        let result = check_run_string(input, Some(Value::Number(5)));
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[test]
    fn identity_function() {
        let input = "let f = λx. x in (f 42)";
        let result = check_run_string(input, Some(Value::Number(42)));
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[test]
    fn multiple_values() {
        let input = "let f = λx. x in let a = (f 1) in let b = (f 2) in a";
        let result = check_run_string(input, Some(Value::Number(1)));
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[test]
    fn conditional_true() {
        let input = "if true then let x = 1 in x else let x = 2 in x";
        let result = check_run_string(input, Some(Value::Number(1)));
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[test]
    fn conditional_unknown() {
        let input = "let b = λx. x in let c = (b true) in let d = (b false) in if c then let r = 1 in r else let r = 2 in r";
        let result = check_run_string(input, Some(Value::Number(1)));
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[test]
    fn higher_order() {
        let input = "let apply = λf. (f 99) in let g = λz. z in (apply g)";
        let result = check_run_string(input, Some(Value::Number(99)));
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[test]
    fn self_application() {
        let input = "let ω = λx. (x x) in (ω ω)";
        let result = check_run_string(input, None);
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[test]
    fn multiple_arguments() {
        let input = "let ω = λx y. (x y y) in let id = λ i j. i in (ω id 4)";
        let result = check_run_string(input, Some(Value::Number(4)));
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }
}