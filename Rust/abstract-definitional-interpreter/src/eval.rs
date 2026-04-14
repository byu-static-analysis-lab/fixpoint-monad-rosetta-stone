use std::cell::RefCell;
use std::rc::Rc;
use fixpoint_monad::{bind, bind_each, Memo, Monad};
use crate::ast::Expr;

use crate::Value;

fn apply_clos<'a>(func_value: Value, arg_values: &[Value], sigma: Memo<'a, Value, String>) -> Monad<'a, Value> {
    match func_value {
        Value::Function {
            args,
            body,
        } => {
            bind_each(&args, arg_values).and_then(move |_| {
                let sigma = sigma.clone();
                let body = body.clone();
                eval(sigma).call(body)
            })
        }
        _ => Monad::fail()
    }
}

fn aeval<'a>(expr: &Expr, sigma: Memo<'a, Value, String>) -> Monad<'a, Value> {
    match expr {
        Expr::Variable(name) => sigma.call(name.clone()),
        Expr::Function {
            params, body
        } => {
            Monad::inject(Value::Function { args: params.clone(), body: body.as_ref().clone() })
        }
        Expr::Boolean(bool) => Monad::inject(Value::Boolean(*bool)),
        Expr::Number(num) => Monad::inject(Value::Number(*num)),
        _ => unreachable!(),
    }
}

fn ceval<'a>(sigma: Memo<'a, Value, String>) -> Memo<'a, Value, Expr> {
    Memo::new("ceval", move |expr| {
        match &expr {
            Expr::Application {
                func, args
            } => {
                let sigma = sigma.clone();
                let func = func.clone();
                let args = args.clone();
                aeval(func.as_ref(), sigma.clone()).and_then(move|fv| {
                    let sigma = sigma.clone();
                    let args = args.clone();

                    let values = Rc::new(RefCell::new(Vec::with_capacity(args.len())));
                    let monads = args.into_iter().map(|arg| {
                        let values = values.clone();
                        aeval(&arg, sigma.clone()).and_then(move |arg_v| {
                            values.borrow_mut().push(arg_v[0].clone());
                            Monad::inject_values(arg_v)
                        })
                    }).collect::<Vec<_>>();
                    Monad::each(monads).and_then(move |_| {
                        let sigma = sigma.clone();
                        apply_clos(fv[0].clone(), &values.clone().borrow(), sigma)
                    })
                })
            }
            _ => aeval(&expr, sigma.clone()),
        }
    })
}

pub fn eval<'a>(sigma: Memo<'a, Value, String>) -> Memo<'a, Value, Expr> {
    Memo::new("eval", move |expr| {
        let sigma = sigma.clone();
        match &expr {
            Expr::Let {
                var,
                val,
                body
            } => {
                let sigma = sigma.clone();
                let var = var.clone();
                let val = val.clone();
                let body = body.clone();
                ceval(sigma.clone()).call(*val.clone())
                    .and_then(move |value| {
                        let sigma = sigma.clone();
                        let body = body.clone();
                        bind(&var, value[0].clone()).and_then(move |_| {
                            let sigma = sigma.clone();
                            eval(sigma).call(*body.clone())
                        })
                    })
            }
            Expr::If {
                cond,
                body,
                els
            } => {
                let sigma = sigma.clone();
                let cond = cond.clone();
                let body = body.clone();
                let els = els.clone();
                aeval(cond.as_ref(), sigma.clone()).and_then(move |cond_value| {
                    let sigma = sigma.clone();
                    match cond_value[0].clone() {
                        Value::Boolean(false) => eval(sigma).call(*els.clone()),
                        _ => eval(sigma).call(*body.clone())
                    }
                })
            }
            _ => ceval(sigma).call(expr)
        }
    })
}