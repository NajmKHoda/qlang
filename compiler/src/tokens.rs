pub struct ProgramNode {
    pub datasources: Vec<DatasourceNode>,
    pub tables: Vec<TableNode>,
    pub structs: Vec<StructNode>,
    pub functions: Vec<FunctionNode>,
}

pub struct DatasourceNode {
    pub name: String,
    pub is_readonly: bool,
}

pub struct TableNode {
    pub name: String,
    pub datasource_name: String,
    pub columns: Vec<TypedQNameNode>,
    pub is_readonly: bool,
}

pub struct StructNode {
    pub name: String,
    pub fields: Vec<TypedQNameNode>,
}

pub struct FunctionNode {
    pub name: String,
    pub return_type: TypeNode,
    pub params: Vec<TypedQNameNode>,
    pub body: Vec<StatementNode>,
}

pub enum StatementNode {
    VariableDefinition {
        name: String,
        var_type: Option<TypeNode>, 
        init_expr: Box<ExpressionNode> 
    },
    Assignment {
        name: String,
        expr: Box<ExpressionNode>,
    },
    Conditional {
        branches: Vec<ConditionalBranchNode>,
        else_branch: Option<Vec<StatementNode>>,
    },
    ConditionalLoop {
        condition: Box<ExpressionNode>,
        body: Vec<StatementNode>,
        label: Option<String>,
    },
    ForLoop {
        variable_name: String,
        iterable_expr: Box<ExpressionNode>,
        body: Vec<StatementNode>,
        label: Option<String>,
    },
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
    Iterator(Box<TypeNode>),
    Struct(String),
    Callable(Vec<TypeNode>, Box<TypeNode>),
    Void
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
    Range {
        start: Option<Box<ExpressionNode>>,
        end: Option<Box<ExpressionNode>>,
        inclusive: bool,
        step: Option<Box<ExpressionNode>>,
    },
    Closure(Vec<TypedQNameNode>, Option<TypeNode>, ClosureBodyNode),
    Add(Box<ExpressionNode>, Box<ExpressionNode>),
    Subtract(Box<ExpressionNode>, Box<ExpressionNode>),
    Comparison(Box<ExpressionNode>, Box<ExpressionNode>, ComparisonType),
    FunctionCall(String, Vec<Box<ExpressionNode>>),
    Struct(Option<String>, Vec<ColumnValueNode>),
    Array(Vec<Box<ExpressionNode>>),
    ArrayIndex(Box<ExpressionNode>, Box<ExpressionNode>),
    MethodCall(Box<ExpressionNode>, String, Vec<Box<ExpressionNode>>),
    ImmediateQuery(QueryNode),
    ParameterizedQuery {
        parameters: Vec<TypedQNameNode>,
        query: QueryNode
    }
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

pub enum ClosureBodyNode {
    Statements(Vec<StatementNode>),
    Expression(Box<ExpressionNode>),
}

// --- QUERIES ---

pub enum QueryNode {
    Select(SelectQueryNode),
    Insert(InsertQueryNode),
    Delete(DeleteQueryNode),
    Update(UpdateQueryNode),
}

pub struct SelectQueryNode {
    pub capturing_struct_name: String,
    pub captured_columns: Vec<(String, QColumnNode)>,
    pub root_table_name: String,
    pub join_clauses: Vec<JoinNode>,
    pub where_clause: Option<WhereNode>,
}

pub struct JoinNode {
    pub left_column: QColumnNode,
    pub right_column: QColumnNode,
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

pub struct QColumnNode {
    pub table_name: String,
    pub column_name: String
}