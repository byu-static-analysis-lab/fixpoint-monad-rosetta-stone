use std::collections::HashMap;
use std::iter::Peekable;
use std::str::CharIndices;
use crate::MakeKey;

#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Expr {
    Function {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Application {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    Let {
        var: String,
        val: Box<Expr>,
        body: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        body: Box<Expr>,
        els: Box<Expr>,
    },
    Variable(String),
    Number(u64),
    Boolean(bool),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Function { params, body } => {
                write!(f, "λ{}. {}", params.join(" "), body)
            }
            Expr::Application { func, args } => {
                let args = args.iter().map(|a| a.to_string()).collect::<Vec<_>>();
                write!(f, "({} {})", func, args.join(" "))
            }
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Number(n) => write!(f, "{}", n),
            Expr::Let { var, val, body } => {
                write!(f, "let {} = {} in {}", var, val, body)
            }
            Expr::If { cond, body, els } => {
                write!(f, "if {} then\n", cond)?;
                write!(f, "    {}\nelse\n    {}", body, els)
            }
            Expr::Boolean(bool) => write!(f, "{}", bool),
        }
    }
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Variable(name) => write!(f, "\"{}\"", name),
            x => write!(f, "{}", x),
        }
    }
}

impl MakeKey for Expr {
    fn show(&self) -> String {
        format!("{}", self)
    }
}


impl Expr {
    pub fn parse(input: &str) -> Result<Expr, String> {
        let tokenizer = ExprLexer::new(input);
        let tokens = tokenizer.into_iter().collect::<Vec<Token>>();
        let mut expr_parser = ExprParser::new(&tokens);
        expr_parser.parse_general()
    }

}


struct Scope {
    hash_map: HashMap<String, u64>,
}

impl Scope {
    fn new() -> Scope {
        Self {
            hash_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, val: u64) {
        self.hash_map.insert(key, val);
    }

    pub fn get(&self, key: &str) -> Option<&u64> {
        self.hash_map.get(key)
    }
}


struct ExprParser<'a> {
    tokens: &'a [Token],
    index: usize,
    scope: Vec<Scope>,
}

impl<'a> ExprParser<'a> {
    fn new(tokens: &'a [Token]) -> ExprParser<'a> {
        Self {
            tokens,
            index: 0,
            scope: vec![]
        }
    }
}

impl ExprParser<'_> {

    pub fn lookup_var(&self, name: &str) -> Option<u64> {
        for scope in self.scope.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(*val);
            }
        }
        None
    }

    pub fn increment_var(&mut self, name: &str) -> u64 {
        let old = self.lookup_var(name);
        if let Some(val) = old {
            self.scope.last_mut().unwrap().insert(name.to_string(), val + 1);
            val + 1
        } else {
            self.scope.last_mut().unwrap().insert(name.to_string(), 1);
            1
        }
    }

    pub fn push_scope(&mut self) {
        self.scope.push(Scope::new());
    }

    fn parse_general(&mut self) -> Result<Expr, String> {
        match &self.tokens[self.index] {
            Token::Let => {
                self.index += 1;
                let Token::Identifier(name) = &self.tokens[self.index] else {
                    return Err(String::from("Expected a variable name after a `let` keyword"));
                };
                self.push_scope();
                let var_index = self.increment_var(name);
                let name = format!("{name}{var_index}");
                self.index += 1;
                let Token::Equals = self.tokens[self.index] else {
                    return Err(format!("Expected `=` after `let {name}`"));
                };
                self.index += 1;
                let value = self.parse_app()?;
                let Token::In = &self.tokens[self.index] else {
                    return Err(format!("Expected `in` after `let {name} = `"));
                };
                self.index += 1;
                let expr = self.parse_general()?;

                Ok(Expr::Let {
                    var: name,
                    val: Box::new(value),
                    body: Box::new(expr),
                })
            }
            Token::If => {
                self.index += 1;
                let condition = self.parse_body()?;
                let Token::Then = self.tokens[self.index] else {
                    return Err(format!("Expected `then` after `if {condition}`"));
                };
                self.index += 1;
                let then = self.parse_general()?;
                let Token::Else = self.tokens[self.index] else {
                    return Err(format!("Expected `else` after `if {condition} then {then}`"));
                };
                self.index += 1;
                let els = self.parse_general()?;

                Ok(Expr::If {
                    cond: Box::new(condition),
                    body: Box::new(then),
                    els: Box::new(els),
                })
            }
            _ => self.parse_app(),
        }
    }

    fn parse_app(&mut self) -> Result<Expr, String> {
        let expr = match self.parse_lambda() {
            Ok(fun) => {
                fun
            }
            Err(_) => {
                match &self.tokens[self.index] {
                    Token::OpenParen => {
                        self.index += 1;
                        let expr = self.parse_expr()?;
                        let mut args = Vec::new();

                        while self.tokens[self.index] != Token::CloseParen {
                            let arg = self.parse_expr()?;
                            args.push(arg);
                        }

                        let Token::CloseParen = &self.tokens[self.index] else {
                            return Err(String::from("Expected a closing paren"));
                        };
                        self.index += 1;
                        Expr::Application {
                            func: Box::new(expr),
                            args,
                        }
                    }
                    _ => {
                        self.parse_body()?
                    }
                }
            }
        };
        Ok(expr)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        if self.index >= self.tokens.len() {
            return Err(String::from("Reached end of input"));
        }
        let Token::Lambda = &self.tokens[self.index] else {
            return self.parse_body();
        };
        self.parse_lambda()
    }
    fn parse_lambda(&mut self) -> Result<Expr, String> {
        let Token::Lambda = &self.tokens[self.index] else {
            return Err(String::from("Expected a lambda character"));
        };
        self.index += 1;
        self.push_scope();
        let mut params = Vec::new();
        while self.tokens[self.index] != Token::Period {
            let Token::Identifier(id) = &self.tokens[self.index] else {
                return Err(String::from("Expected a identifier"));
            };
            let var_index = self.increment_var(id);
            let id = format!("{id}{var_index}");
            self.index += 1;
            params.push(id);
        }

        let Token::Period = &self.tokens[self.index] else {
            return Err(String::from("Expected a period"));
        };
        self.index += 1;
        let body = self.parse_general()?;

        Ok(Expr::Function {
            params,
            body: Box::new(body),
        })
    }

    fn parse_body(&mut self) -> Result<Expr, String> {
        match &self.tokens[self.index] {
            Token::Identifier(id) => {
                self.index += 1;
                let var_index = self.lookup_var(id).ok_or(String::from("Variable not bound"))?;
                let id = format!("{id}{var_index}");
                Ok(Expr::Variable(id))
            }
            Token::OpenParen => {
                let expr = self.parse_app()?;
                Ok(expr)
            }
            Token::Number(num) => {
                self.index += 1;
                Ok(Expr::Number(*num))
            }
            Token::Boolean(bool) => {
                self.index += 1;
                Ok(Expr::Boolean(*bool))
            }
            x => Err(format!("Unexpected token: {x:?}")),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Identifier(String),
    Let,
    In,
    If,
    Then,
    Else,
    Equals,
    Number(u64),
    Boolean(bool),
    Period,
    Lambda,
    OpenParen,
    CloseParen,
}

struct ExprLexer<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> ExprLexer<'a> {
    fn new(input: &'a str) -> Self {
        let iter = input.char_indices().peekable();
        Self {
            input,
            chars: iter,
        }
    }
}

impl<'a> Iterator for ExprLexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((start, c)) = self.chars.next() {
            match c {
                'λ' | '\\' => {
                    return Some(Token::Lambda);
                }
                '.' => {
                    return Some(Token::Period);
                }
                '(' => {
                    return Some(Token::OpenParen);
                }
                ')' => {
                    return Some(Token::CloseParen);
                }
                '=' => {
                    return Some(Token::Equals);
                }
                ' ' | '\t' | '\n' => {}
                '0'..= '9' => {
                    let mut end = start;
                    while let Some((next, c)) = self.chars.peek() {
                        let c = *c;
                        let next = *next;
                        if c >= '0' && c <= '9' {
                            self.chars.next();
                        } else {
                            end = next;
                            break;
                        }
                    }
                    let str = &self.input[start..end];
                    return Some(Token::Number(str.parse::<u64>().unwrap()));
                }
                _ => {
                    let mut end = start + c.len_utf8();
                    while let Some((next, c)) = self.chars.peek() {
                        let c = *c;
                        let next = *next;
                        end = next;
                        if c.is_alphabetic() {
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    let str = &self.input[start..end];
                    return match str {
                        "lambda" => Some(Token::Lambda),
                        "let" => Some(Token::Let),
                        "in" => Some(Token::In),
                        "if" => Some(Token::If),
                        "then" => Some(Token::Then),
                        "else" => Some(Token::Else),
                        "true" => Some(Token::Boolean(true)),
                        "false" => Some(Token::Boolean(false)),
                        str => Some(Token::Identifier(str.to_string())),
                    }
                }
            }
        }
        None
    }
}