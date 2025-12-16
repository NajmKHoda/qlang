pub enum StatementNode {
    Assignment(String, Box<ExpressionNode>),
    Print(Box<ExpressionNode>)
}

pub enum ExpressionNode {
    QName(String),
    Integer(i32),
    Add(Box<ExpressionNode>, Box<ExpressionNode>),
    Input
}
