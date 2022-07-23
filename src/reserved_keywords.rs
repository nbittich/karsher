use crate::{adana_script::constants::*, cache_command::CacheCommand};

pub const CACHE_COMMAND_DOC: &[(&[&str], &str)] = CacheCommand::doc();

pub const FORBIDDEN_VARIABLE_NAME: &[&str] = &[
    TRUE,
    FALSE,
    TAU,
    IF,
    PI,
    PRINT_LN,
    PRINT,
    LENGTH,
    EULER_NUMBER,
    ABS,
    LOG,
    LN,
    SIN,
    COS,
    TAN,
    INCLUDE,
    WHILE,
    ELSE,
    MULTILINE,
];

pub fn check_reserved_keyword(aliases: &[&str]) -> bool {
    CACHE_COMMAND_DOC
        .iter()
        .flat_map(|c| c.0.iter())
        .chain(FORBIDDEN_VARIABLE_NAME.iter())
        .any(|c| aliases.iter().any(|al| al.eq_ignore_ascii_case(c)))
}
