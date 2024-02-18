#[derive(Debug, Clone)]
pub enum PrimitiveType {
    String,
    Int,
}

#[derive(Debug, Clone)]
pub enum TyKind {
    // Built-int, primtive types 
    Primitive(PrimitiveType),

    // User Defined
    UserDefined(String),

    // The nullable type T?
    Nullable(Box<TyKind>),

    // The array type [T]
    Array(Box<TyKind>)
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub datatype: TyKind,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub structs: Vec<StructDefinition>,
}
