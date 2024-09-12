#[derive(Debug, Clone)]
pub enum PrimitiveType {
    String,
    Int,
    Float,
    Bool
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
    Array(Box<TyKind>),

    // The map type <T>
    Map(Box<TyKind>),

    // The tuple type (A, B, C, ...)
    Tuple(Vec<TyKind>),
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
pub enum EnumVariantValue {
    Int(i32),
    String(String)
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: EnumVariantValue
}

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub structs: Vec<StructDefinition>,
    pub enums: Vec<EnumDefinition>,
}
