use crate::tokenizer::Number;

#[derive(Debug, Default)]
pub struct Module {
    pub use_statements: Vec<()>,

    pub struct_def: Vec<StructDef>,
    pub union_def: Vec<UnionDef>,
    pub enum_def: Vec<EnumDef>,

    pub glob_def: Vec<GlobalDef>,

    pub function_def: Vec<FunctionDef>,
    pub function_header: Vec<FunctionHeader>,
}

impl Module {
    pub fn single(tl: TopLevelDef) -> Self {
        let mut this = Self::default();
        this.append(tl);
        this
    }

    pub fn append(&mut self, tl: TopLevelDef) {
        match tl{
            TopLevelDef::FunctionDef(item) => self.function_def.push(item),
            TopLevelDef::FunctionHeader(item) => self.function_header.push(item),
            TopLevelDef::StructDef(item) => self.struct_def.push(item),
            TopLevelDef::EnumDef(item) => self.enum_def.push(item),
            TopLevelDef::UnionDef(item) => self.union_def.push(item),
            TopLevelDef::GlobalDef(item) => self.glob_def.push(item),
            TopLevelDef::UseStatement() => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TopLevelDef {
    FunctionDef(FunctionDef),
    FunctionHeader(FunctionHeader),
    StructDef(StructDef),
    EnumDef(EnumDef),
    UnionDef(UnionDef),
    GlobalDef(GlobalDef),
    UseStatement(),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Block {
    Scope(Option<String>, Vec<Statement>),
    While(Option<String>, Box<Expression>, Vec<Statement>),
    If(
        Option<String>,
        Box<Expression>,
        Vec<Statement>,
        Vec<(Expression, Vec<Statement>)>,
        Option<Vec<Statement>>,
    ),
}

#[derive(Debug, Clone)]
pub enum LiteralType<'a> {
    Boolean(bool),
    String(byteyarn::YarnBox<'a, str>),
    Char(char),
    Number(Number<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    String(String),
    Char(String),
    Boolean(bool),
    Number(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntSize {
    U8,
    U16,
    U32,
    U64,
    Usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FloatType {
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int(IntSize, bool),
    Float(FloatType),
    Bool,
    Char,
    Void,
    Str,
    FnPointer(Vec<Type>, Option<Box<Type>>),
    Nammed(Path),
    Ptr(Box<Type>),
    Ref(Box<Type>),
    Array(Box<Type>),
    ArrayStatic(Box<Type>, Box<Expression>),
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Path(Path),
    Literal(Literal),
    Block(Block),
    FieldAccess(Box<Expression>, String),
    MemberFunction(Box<Expression>, String, Vec<Expression>),
    ArrayAccess(Box<Expression>, Box<Expression>),
    FunctionCall(Box<Expression>, Vec<Expression>),
    UnaryOp(UnaryOpKind, Box<Expression>),

    BinaryOp(Box<Expression>, BinOpKind, Box<Expression>),

    Assign(Box<Expression>, Box<Expression>),

    SizeOf(Type),
    AlignOf(Type),
    Sized(Type),
    OffsetOf(Type, String),
    TypeName(Type),

    StructCon(Path, Vec<(String, Expression)>),
    ArrayCon(Vec<Expression>),

    Break(Option<String>, Option<Box<Expression>>),
    Continue(Option<String>),
    Return(Option<Box<Expression>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Statement {
    Expression(Expression),
    VariableDeclaration(Type, String, Expression),
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Path {
    pub path: std::path::PathBuf,
}

impl Path {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, str: &str) {
        self.path.push(str);
    }

    pub fn new_path(path: &str) -> Self {
        let mut new = Self::new();
        for part in path.split("::") {
            new.push(part)
        }
        new
    }
}

#[derive(Debug, Clone)]
pub enum GlobalKind {
    Const,
    Static,
}

#[derive(Debug, Clone)]
pub struct GlobalDef {
    pub kind: GlobalKind,
    pub ty: Type,
    pub name: String,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub kind: Option<String>,
    pub params: Vec<(Type, String)>,
    pub ret: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct FunctionHeader {
    pub name: String,
    pub kind: Option<String>,
    pub params: Vec<(Type, String)>,
    pub ret: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub values: Vec<(Type, String)>,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UnionDef {
    pub name: String,
    pub values: Vec<(Type, String)>,
}

#[derive(Debug, Clone)]
pub struct UnOp {
    pub kind: UnaryOpKind,
    pub expr: Expression,
}

#[derive(Debug, Clone)]
pub struct BinOp {
    pub left: Expression,
    pub kind: BinOpKind,
    pub right: Expression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOpKind {
    Negate,
    Deref,
    Not,
    Ref,
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOpKind {
    Times,
    Divide,
    Modulo,
    Plus,
    Minus,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitXor,
    BitOr,
    Eq,
    Neq,
    Gt,
    Lt,
    Gteq,
    Lteq,
    LogicalAnd,
    LogicalOr,
}


impl Type{
    pub fn new(path: Path) -> Self {
        use crate::parser::ast::FloatType;
        use crate::parser::ast::IntSize;
        match path.path.as_os_str().to_str().unwrap_or_default() {
            "u8" => Self::Int(IntSize::U8, false),
            "u16" => Self::Int(IntSize::U16, false),
            "u32" => Self::Int(IntSize::U32, false),
            "u64" => Self::Int(IntSize::U64, false),
            "usize" => Self::Int(IntSize::Usize, false),

            "i8" => Self::Int(IntSize::U8, true),
            "i16" => Self::Int(IntSize::U16, true),
            "i32" => Self::Int(IntSize::U32, true),
            "i64" => Self::Int(IntSize::U64, true),
            "isize" => Self::Int(IntSize::Usize, true),

            "f32" => Self::Float(FloatType::F32),
            "f64" => Self::Float(FloatType::F64),

            "bool" => Self::Bool,
            "char" => Self::Char,
            "str" => Self::Str,
            "void" => Self::Void,

            _ => Self::Nammed(path),
        }
    }

    pub fn new_fn(args: Vec<Self>, ret: Option<Self>) -> Self {
        Self::FnPointer(args, ret.map(Box::new))
    }

    pub fn wrap_ref(self) -> Self {
        Self::Ref(self.into())
    }

    pub fn wrap_ptr(self) -> Self {
        Self::Ptr(self.into())
    }

    pub fn wrap_array(self) -> Self {
        Self::Array(self.into())
    }

    pub fn wrap_array_sized(self, e: Expression) -> Self {
        Self::ArrayStatic(self.into(), e.into())
    }
}