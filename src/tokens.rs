pub enum Statement {
    Assignment(String, Expression),
    Print(Expression)
}

pub enum Expression {
    QName(String),
    Integer(i32)
}
