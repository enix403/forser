use crate::token::Token;

enum BuiltInDataType {
    String,
    Int(i8),
    UInt(i8),
    Float,
    Double,
    Boolean,
}

enum DataType {
    BuiltIn(BuiltInDataType),
    UserDefined(String), // TODO: use interning
}

struct StructDefinition {
    name: String,
    fields: Vec<StructField>,
}

struct StructField {
    datatype: DataType,
    name: String,
}

struct Parser<L> {
    structs: Vec<StructDefinition>,
    lexer: L,
    current: Option<Token>,
    next: Option<Token>,
}
