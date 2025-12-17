use crate::codegen::{CodeGen, CodeGenError, QLValue};

pub enum StatementNode {
    Assignment(String, Box<ExpressionNode>),
    Conditional(Box<ExpressionNode>, Vec<StatementNode>, Vec<StatementNode>),
    LoneExpression(Box<ExpressionNode>),
}

impl StatementNode {
    pub fn gen_stmt<'a>(&self, code_gen: &mut CodeGen<'a>) -> Result<(), CodeGenError> {
        match self {
            StatementNode::Assignment(var_name, expr) => {
                let value = expr.gen_eval(code_gen)?;
                code_gen.store_var(var_name, value)
            },
            StatementNode::Conditional(cond_expr, then_stmts, else_stmts) => {
                let condition = cond_expr.gen_eval(code_gen)?;
                code_gen.gen_conditional(condition, then_stmts, else_stmts)
            }
            StatementNode::LoneExpression(expr) => {
                expr.gen_eval(code_gen).map(|_| ())
            }
        }
    }
}

pub enum ExpressionNode {
    QName(String),
    Integer(i32),
    Add(Box<ExpressionNode>, Box<ExpressionNode>),
    FunctionCall(String, Vec<Box<ExpressionNode>>),
}

impl ExpressionNode {
    pub fn gen_eval<'a>(&self, code_gen: &CodeGen<'a>) -> Result<QLValue<'a>, CodeGenError> {
        match self {
            ExpressionNode::Integer(x) => Ok(code_gen.const_int(*x)),
            ExpressionNode::QName(name) => code_gen.load_var(name),
            ExpressionNode::Add(expr1, expr2) => {
                let val1 = expr1.gen_eval(code_gen)?;
                let val2 = expr2.gen_eval(code_gen)?;
                code_gen.add(val1, val2)
            },
            ExpressionNode::FunctionCall(fn_name, arg_exprs) => {
                let args = arg_exprs.iter()
                    .map(|expr| expr.gen_eval(code_gen))
                    .collect::<Result<Vec<QLValue>, CodeGenError>>()?;
                code_gen.call(fn_name, args)
            }
        }
    }
}
