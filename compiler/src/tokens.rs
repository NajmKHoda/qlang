pub struct ProgramNode {
    pub datasources: Vec<DatasourceNode>,
    pub tables: Vec<TableNode>,
    pub functions: Vec<FunctionNode>,
}

pub struct DatasourceNode {
    pub name: String
}

pub struct TableNode {
    pub name: String,
    pub datasource_name: String,
    pub columns: Vec<TypedQNameNode>,
}

pub struct FunctionNode {
    pub name: String,
    pub return_type: TypeNode,
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

pub struct TypedQNameNode {
    pub name: String,
    pub type_node: TypeNode,
}

pub enum TypeNode {
    Integer,
    Bool,
    String,
    Array(Box<TypeNode>),
    Struct(String),
}

pub struct ConditionalBranchNode {
    pub condition: Box<ExpressionNode>,
    pub body: Vec<StatementNode>
}

pub enum ExpressionNode {
    QName(String),
    StructField(Box<ExpressionNode>, String),
    IntegerLiteral(i32),
    BoolLiteral(bool),
    StringLiteral(String),
    Add(Box<ExpressionNode>, Box<ExpressionNode>),
    Subtract(Box<ExpressionNode>, Box<ExpressionNode>),
    Comparison(Box<ExpressionNode>, Box<ExpressionNode>, ComparisonType),
    FunctionCall(String, Vec<Box<ExpressionNode>>),
    Struct(Option<String>, Vec<ColumnValueNode>),
    Array(Vec<Box<ExpressionNode>>),
    ArrayIndex(Box<ExpressionNode>, Box<ExpressionNode>),
    MethodCall(Box<ExpressionNode>, String, Vec<Box<ExpressionNode>>),
    Query(QueryNode)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparisonType {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual
}

pub struct ColumnValueNode {
    pub name: String,
    pub value: Box<ExpressionNode>
}

// --- QUERIES ---

pub enum QueryNode {
    Select(SelectQueryNode),
    Insert(InsertQueryNode),
    Delete(DeleteQueryNode),
    Update(UpdateQueryNode),
}

pub struct SelectQueryNode {
    pub table_name: String,
    pub where_clause: Option<WhereNode>,
}

pub struct WhereNode {
    pub column_name: String,
    pub value: Box<ExpressionNode>,
}

pub struct InsertQueryNode {
    pub table_name: String,
    pub data_expr: Box<ExpressionNode>,
}

pub struct DeleteQueryNode {
    pub table_name: String,
    pub where_clause: Option<WhereNode>,
}

pub struct UpdateQueryNode {
    pub table_name: String,
    pub assignments: Vec<UpdateAssignmentNode>,
    pub where_clause: Option<WhereNode>,
}

pub struct UpdateAssignmentNode {
    pub column_name: String,
    pub value_expr: Box<ExpressionNode>,
}
