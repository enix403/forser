#[derive(Debug, Clone)]
pub enum PrimitiveType {
    String,
    Int,
}

#[derive(Debug, Clone)]
pub enum DataType {
    Primitive(PrimitiveType),
    UserDefined(String), // TODO: use interning
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub datatype: DataType,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub structs: Vec<StructDefinition>
}