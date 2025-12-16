use std::io::Read;
use std::io::Error as IOError;
use std::fs::File;
use std::collections::HashMap;
use lalrpop_util::lalrpop_mod;

use crate::tokens::ExpressionNode;
use crate::tokens::StatementNode;

mod tokens;

lalrpop_mod!(pub simple);

fn eval_expr(expr: &ExpressionNode, vars: &HashMap<String, i32>) -> i32 {
    match expr {
        ExpressionNode::Integer(val) => *val,
        ExpressionNode::QName(name) => vars[name],
        ExpressionNode::Add(expr1, expr2) => {
            let a = eval_expr(expr1, vars);
            let b = eval_expr(expr2, vars);
            a + b
        }
    }
}

fn main() -> Result<(), IOError> {
    let parser = simple::ProgramParser::new();
    let mut source = String::new();
    let mut file = File::open("main.ql")?;
    file.read_to_string(&mut source)?;

    let program = parser.parse(&source).map_err(|e| {
        eprintln!("Failed to parse main.ql: {e}");
        IOError::new(std::io::ErrorKind::InvalidData, "Parsing failed")
    })?;

    let mut vars = HashMap::<String, i32>::new();
    for statement in program {
        match statement {
            StatementNode::Print(ref expr) => {
                let val = eval_expr(expr, &vars);
                println!("{val}")
            },
            StatementNode::Assignment(ref qname, ref expr) => {
                let val = eval_expr(expr, &vars);
                vars.insert(qname.clone(), val);
            }
        }
    }

    Ok(())
}
