mod ast;
mod compute;
mod parser;
mod primitive;

pub use compute::compute;
pub use primitive::Primitive;
use serde::{Deserialize, Serialize};
use strum::EnumCount;

use self::constants::{
    ABS, COS, INCLUDE, LENGTH, LN, LOG, PRINT, PRINT_LN, SIN, SQRT, TAN,
};

#[macro_use]
pub mod constants {
    macro_rules! pi {
        () => {
            'π'
        };
    }
    macro_rules! euler_number {
        () => {
            'γ'
        };
    }
    macro_rules! tau {
        () => {
            'τ'
        };
    }
    pub const TRUE: &str = "true";
    pub const FALSE: &str = "false";
    pub const IF: &str = "if";
    pub const ELSE: &str = "else";
    pub const WHILE: &str = "while";
    pub const TAU: &str = concat!(tau!());
    pub const PI: &str = concat!(pi!());
    pub const EULER_NUMBER: &str = concat!(euler_number!());
    pub const SQRT: &str = "sqrt";
    pub const ABS: &str = "abs";
    pub const LENGTH: &str = "length";
    pub const LOG: &str = "log";
    pub const LN: &str = "ln";
    pub const SIN: &str = "sin";
    pub const COS: &str = "cos";
    pub const TAN: &str = "tan";
    pub const PRINT_LN: &str = "println";
    pub const PRINT: &str = "print";
    pub const INCLUDE: &str = "include";
    pub const MULTILINE: &str = "multiline";
}

#[derive(Debug, EnumCount)]
pub(super) enum MathConstants {
    Pi,
    EulerNumber,
    Tau,
}

impl MathConstants {
    pub(super) const fn get_symbol(&self) -> char {
        match self {
            MathConstants::Pi => pi!(),
            MathConstants::EulerNumber => euler_number!(),
            MathConstants::Tau => tau!(),
        }
    }
    pub(super) const fn get_symbols() -> &'static str {
        concat!(pi!(), euler_number!(), tau!())
    }

    pub(super) const fn _get_variants(
    ) -> &'static [&'static MathConstants; MathConstants::COUNT] {
        &[&MathConstants::Pi, &MathConstants::EulerNumber, &MathConstants::Tau]
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Value {
    Expression(Vec<Value>),
    Operation(Operator),
    BuiltInFunction {
        fn_type: BuiltInFunctionType,
        expr: Box<Value>,
    },
    Function {
        parameters: Box<Value>,
        exprs: Vec<Value>,
    },
    // FunctionCall {
    //    name: Block<Value>,
    //    parameters: Vec<Value>,
    //}
    Decimal(f64),
    Integer(i128),
    Bool(bool),
    String(String),
    BlockParen(Vec<Value>),
    Variable(String),
    Const(char),
    VariableNegate(String),
    VariableExpr {
        name: Box<Value>,
        expr: Box<Value>,
    },
    IfExpr {
        cond: Box<Value>,
        exprs: Vec<Value>,
        else_expr: Option<Vec<Value>>,
    },
    WhileExpr {
        cond: Box<Value>,
        exprs: Vec<Value>,
    },
    Array(Vec<Value>),
    ArrayAccess {
        arr: Box<Value>,
        index: Box<Value>,
    },
}
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum BuiltInFunctionType {
    Sqrt,
    Abs,
    Log,
    Ln,
    Sin,
    Cos,
    Tan,
    Println,
    Print,
    Length,
    Include,
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Operator {
    Add,
    Subtr,
    Mult,
    Div,
    Mod,
    Pow,
    Not,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

#[derive(Debug)]
pub(super) enum TreeNodeValue {
    VariableAssign(String),
    VariableArrayAssign { name: String, index: Primitive },
    Ops(Operator),
    Primitive(Primitive),
    BuiltInFunction(BuiltInFunctionType),
    IfExpr(Value),
    WhileExpr(Value),
    Array(Vec<Value>),
    ArrayAccess { index: Primitive, array: Value },
    Function(Value),
}

impl BuiltInFunctionType {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            BuiltInFunctionType::Sqrt => SQRT,
            BuiltInFunctionType::Abs => ABS,
            BuiltInFunctionType::Log => LOG,
            BuiltInFunctionType::Ln => LN,
            BuiltInFunctionType::Length => LENGTH,
            BuiltInFunctionType::Sin => SIN,
            BuiltInFunctionType::Cos => COS,
            BuiltInFunctionType::Tan => TAN,
            BuiltInFunctionType::Println => PRINT_LN,
            BuiltInFunctionType::Print => PRINT,
            BuiltInFunctionType::Include => INCLUDE,
        }
    }
}
impl Operator {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            Operator::Add => "+",
            Operator::Subtr => "-",
            Operator::Div => "/",
            Operator::Mult => "*",
            Operator::Pow => "^",
            Operator::Not => "!",
            Operator::Mod => "%",
            Operator::Less => "<",
            Operator::Greater => ">",
            Operator::LessOrEqual => "<=",
            Operator::GreaterOrEqual => ">=",
            Operator::Equal => "==",
            Operator::NotEqual => "!=",
            Operator::And => "&&",
            Operator::Or => "||",
        }
    }
}

#[cfg(test)]
mod tests;
