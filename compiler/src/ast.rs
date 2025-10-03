use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Impl(Impl),
    Import(Import),
    Actor(Actor),
    Statement(Statement),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<Type>,
    pub default: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Impl {
    pub target: String,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Import {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub items: Option<Vec<String>>, // For selective imports
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Actor {
    pub name: String,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Primitive(PrimitiveType),
    Array {
        element_type: Box<Type>,
        size: Option<usize>,
    },
    Optional(Box<Type>),
    Reference {
        target: Box<Type>,
        mutable: bool,
    },
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Int,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Char,
    String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    Expression(ExpressionStatement),
    VariableDeclaration(VariableDeclaration),
    Assignment(Assignment),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Match(MatchStatement),
    Return(ReturnStatement),
    Break,
    Continue,
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionStatement {
    pub expression: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableDeclaration {
    pub name: String,
    pub var_type: Option<Type>,
    pub initializer: Expression,
    pub mutable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub target: Expression,
    pub operator: AssignmentOperator,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignmentOperator {
    Assign,           // =
    AddAssign,        // +=
    SubAssign,        // -=
    MulAssign,        // *=
    DivAssign,        // /=
    ModAssign,        // %=
    BitwiseAndAssign, // &=
    BitwiseOrAssign,  // |=
    BitwiseXorAssign, // ^=
    LeftShiftAssign,  // <<=
    RightShiftAssign, // >>=
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Block,
    pub elif_branches: Vec<(Expression, Block)>,
    pub else_branch: Option<Block>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForStatement {
    pub variable: String,
    pub iterable: Expression,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchStatement {
    pub expression: Expression,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Statement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Identifier(String),
    Literal(Literal),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Call(CallExpression),
    Index(IndexExpression),
    FieldAccess(FieldAccessExpression),
    MethodCall(MethodCallExpression),
    ModuleAccess(ModuleAccessExpression),
    Array(ArrayExpression),
    Dictionary(DictionaryExpression),
    Set(SetExpression),
    Tuple(TupleExpression),
    StructInit(StructInitExpression),
    If(IfExpression),
    Match(MatchExpression),
    Range(RangeExpression),
    Await(AwaitExpression),
    Spawn(SpawnExpression),
    Cast(CastExpression),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Boolean(bool),
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
    Power,    // **

    // Comparison
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    // Logical
    And, // and
    Or,  // or

    // Bitwise
    BitwiseAnd, // &
    BitwiseOr,  // |
    BitwiseXor, // ^
    LeftShift,  // <<
    RightShift, // >>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Minus,            // -
    Not,              // not, !
    BitwiseNot,       // ~
    Reference,        // &
    MutableReference, // &mut
    Dereference,      // *
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallExpression {
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexExpression {
    pub object: Box<Expression>,
    pub index: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldAccessExpression {
    pub object: Box<Expression>,
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodCallExpression {
    pub object: Box<Expression>,
    pub method: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleAccessExpression {
    pub module: String,
    pub item: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayExpression {
    pub elements: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DictionaryExpression {
    pub pairs: Vec<(Expression, Expression)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetExpression {
    pub elements: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleExpression {
    pub elements: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructInitExpression {
    pub struct_name: String,
    pub fields: Vec<(String, Expression)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub then_expr: Box<Expression>,
    pub else_expr: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchExpression {
    pub expression: Box<Expression>,
    pub arms: Vec<MatchExpressionArm>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchExpressionArm {
    pub pattern: Pattern,
    pub expression: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RangeExpression {
    pub start: Box<Expression>,
    pub end: Box<Expression>,
    pub inclusive: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AwaitExpression {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnExpression {
    pub expression: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CastExpression {
    pub expression: Box<Expression>,
    pub target_type: Type,
}
