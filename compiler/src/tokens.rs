use crate::codegen::{CodeGen, CodeGenError, QLValue, QLType, ComparisonOp};

pub enum StatementNode {
    VariableDefinition(String, QLType, Box<ExpressionNode>),
    Assignment(String, Box<ExpressionNode>),
    Conditional(Box<ExpressionNode>, Vec<StatementNode>, Vec<StatementNode>),
    ConditionalLoop(Box<ExpressionNode>, Vec<StatementNode>),
    LoneExpression(Box<ExpressionNode>),
}

impl StatementNode {
    pub fn gen_stmt<'a>(self, code_gen: &mut CodeGen<'a>) -> Result<(), CodeGenError> {
        match self {
            StatementNode::VariableDefinition(var_name, var_type, expr) => {
                let value = expr.gen_eval(code_gen)?;
                code_gen.define_var(&var_name, var_type, value)
            },
            StatementNode::Assignment(var_name, expr) => {
                let value = expr.gen_eval(code_gen)?;
                code_gen.store_var(&var_name, value)
            },
            StatementNode::Conditional(cond_expr, then_stmts, else_stmts) => {
                let condition = cond_expr.gen_eval(code_gen)?;
                code_gen.gen_conditional_loop(condition, then_stmts, else_stmts)
            }
            StatementNode::ConditionalLoop(cond_expr, body_stmts) => {
                code_gen.gen_loop(cond_expr, body_stmts)
            }
            StatementNode::LoneExpression(expr) => {
                expr.gen_eval(code_gen).map(|_| ())
            }
        }
    }
}

pub enum ExpressionNode {
    QName(String),
    IntegerLiteral(i32),
    BoolLiteral(bool),
    Add(Box<ExpressionNode>, Box<ExpressionNode>),
    Subtract(Box<ExpressionNode>, Box<ExpressionNode>),
    Comparison(Box<ExpressionNode>, Box<ExpressionNode>, ComparisonOp),
    FunctionCall(String, Vec<Box<ExpressionNode>>),
}

impl ExpressionNode {
    pub fn gen_eval<'a>(self, code_gen: &CodeGen<'a>) -> Result<QLValue<'a>, CodeGenError> {
        match self {
            ExpressionNode::IntegerLiteral(x) => Ok(code_gen.const_int(x)),
            ExpressionNode::BoolLiteral(x) => Ok(code_gen.const_bool(x)),
            ExpressionNode::QName(name) => code_gen.load_var(&name),
            ExpressionNode::Add(expr1, expr2) => {
                let val1 = expr1.gen_eval(code_gen)?;
                let val2 = expr2.gen_eval(code_gen)?;
                code_gen.gen_add(val1, val2)
            },
            ExpressionNode::Subtract(expr1, expr2) => {
                let val1 = expr1.gen_eval(code_gen)?;
                let val2 = expr2.gen_eval(code_gen)?;
                code_gen.gen_subtract(val1, val2)
            },
            ExpressionNode::Comparison(expr1, expr2, op) => {
                let val1 = expr1.gen_eval(code_gen)?;
                let val2 = expr2.gen_eval(code_gen)?;
                code_gen.gen_compare(val1, val2, op)
            }
            ExpressionNode::FunctionCall(fn_name, arg_exprs) => {
                let args = arg_exprs.into_iter()
                    .map(|expr| expr.gen_eval(code_gen))
                    .collect::<Result<Vec<QLValue>, CodeGenError>>()?;
                code_gen.gen_call(&fn_name, args)
            }
        }
    }
}
