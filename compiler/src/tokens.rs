use crate::codegen::{CodeGen, CodeGenError, QLValue, QLType, ComparisonOp};

pub struct ProgramNode {
    pub tables: Vec<TableNode>,
    pub functions: Vec<FunctionNode>,
}

pub struct TableNode {
    pub name: String,
    pub columns: Vec<TypedQNameNode>,
}

pub struct FunctionNode {
    pub name: String,
    pub return_type: QLType,
    pub params: Vec<TypedQNameNode>,
    pub body: Vec<StatementNode>,
}

pub enum StatementNode {
    VariableDefinition(TypedQNameNode, Box<ExpressionNode>),
    Assignment(String, Box<ExpressionNode>),
    Conditional(Vec<ConditionalBranchNode>, Option<Vec<StatementNode>>),
    ConditionalLoop(Box<ExpressionNode>, Vec<StatementNode>, Option<String>),
    LoneExpression(Box<ExpressionNode>),
    Return(Option<Box<ExpressionNode>>),
    Break(Option<String>),
    Continue(Option<String>)
}

impl StatementNode {
    pub fn gen_stmt<'a>(&self, code_gen: &mut CodeGen<'a>) -> Result<bool, CodeGenError> {
        match self {
            StatementNode::VariableDefinition(var, expr) => {
                let value = expr.gen_eval(code_gen)?;
                code_gen.define_var(&var.name, &var.ql_type, value)?;
            },
            StatementNode::Assignment(var_name, expr) => {
                let value = expr.gen_eval(code_gen)?;
                code_gen.store_var(&var_name, value)?;
            },
            StatementNode::Conditional(cond_branches, else_body) => {
                return code_gen.gen_conditional(cond_branches, else_body);
            }
            StatementNode::ConditionalLoop(cond_expr, body_stmts, loop_label) => {
                code_gen.gen_loop(cond_expr, body_stmts, loop_label)?;
            }
            StatementNode::LoneExpression(expr) => {
                code_gen.gen_lone_expression(expr)?;
            }
            StatementNode::Return(expr) => {
                let val = match expr {
                    Some(e) => Some(e.gen_eval(code_gen)?),
                    None => None
                };
                code_gen.gen_return(val)?;
                return Ok(true);
            }
            StatementNode::Break(label) => {
                code_gen.gen_break(label)?;
                return Ok(true);
            },
            StatementNode::Continue(label) => {
                code_gen.gen_continue(label)?;
                return Ok(true);
            }
        };
        Ok(false)
    }
    
}

pub struct TypedQNameNode {
    pub name: String,
    pub ql_type: QLType,
}

pub struct ConditionalBranchNode {
    pub condition: Box<ExpressionNode>,
    pub body: Vec<StatementNode>
}

pub enum ExpressionNode {
    QName(String),
    Column(Box<ExpressionNode>, String),
    IntegerLiteral(i32),
    BoolLiteral(bool),
    StringLiteral(String),
    Add(Box<ExpressionNode>, Box<ExpressionNode>),
    Subtract(Box<ExpressionNode>, Box<ExpressionNode>),
    Comparison(Box<ExpressionNode>, Box<ExpressionNode>, ComparisonOp),
    FunctionCall(String, Vec<Box<ExpressionNode>>),
    TableRow(String, Vec<ColumnValueNode>),
}

impl ExpressionNode {
    pub fn gen_eval<'a>(&self, code_gen: &CodeGen<'a>) -> Result<QLValue<'a>, CodeGenError> {
        match self {
            ExpressionNode::IntegerLiteral(x) => Ok(code_gen.const_int(*x)),
            ExpressionNode::BoolLiteral(x) => Ok(code_gen.const_bool(*x)),
            ExpressionNode::StringLiteral(x) => Ok(code_gen.const_str(x)?),
            ExpressionNode::QName(name) => code_gen.load_var(&name),
            ExpressionNode::Column(table_row_expr, column_name) => {
                let table_row = table_row_expr.gen_eval(code_gen)?;
                code_gen.get_column_value(table_row, column_name)
            }
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
                code_gen.gen_compare(val1, val2, *op)
            }
            ExpressionNode::FunctionCall(fn_name, arg_exprs) => {
                let args = arg_exprs.into_iter()
                    .map(|expr| expr.gen_eval(code_gen))
                    .collect::<Result<Vec<QLValue>, CodeGenError>>()?;
                code_gen.gen_call(&fn_name, args)
            }
            ExpressionNode::TableRow(table_name, columns) => {
                code_gen.gen_table_row(&table_name, columns)
            }
        }
    }
}

pub struct ColumnValueNode {
    pub name: String,
    pub value: Box<ExpressionNode>
}
