use nom::{
    bytes::complete::take_while1, combinator::rest, multi::separated_list0,
    sequence::pair,
};

use crate::{
    prelude::{
        all_consuming, alt, cut, delimited, double, many0, many1, map,
        map_parser, multispace0, one_of, opt, preceded, recognize_float,
        separated_pair, tag, tag_no_case, take_until, terminated, tuple,
        verify, Res, I128,
    },
    reserved_keywords::check_reserved_keyword,
};

use super::{
    constants::{ELSE, IF, MULTILINE, WHILE},
    BuiltInFunctionType, MathConstants, Operator, Value,
};

fn comments(s: &str) -> Res<Vec<&str>> {
    terminated(
        many0(preceded(tag_no_space("#"), take_until("\n"))),
        multispace0,
    )(s)
}

fn tag_no_space<'a>(t: &'a str) -> impl Fn(&'a str) -> Res<&'a str> {
    move |s: &str| delimited(multispace0, tag(t), multispace0)(s)
}
fn tag_no_space_no_case<'a>(t: &'a str) -> impl Fn(&'a str) -> Res<&'a str> {
    move |s: &str| delimited(multispace0, tag_no_case(t), multispace0)(s)
}

fn parse_number(s: &str) -> Res<Value> {
    map_parser(
        recognize_float,
        alt((
            map(all_consuming(I128), Value::Integer),
            map(all_consuming(double), Value::Decimal),
        )),
    )(s)
}

fn parse_bool(s: &str) -> Res<Value> {
    alt((
        map(tag_no_space("true"), |_| Value::Bool(true)),
        map(tag_no_space("false"), |_| Value::Bool(false)),
    ))(s)
}

fn parse_string(s: &str) -> Res<Value> {
    map(
        delimited(
            preceded(multispace0, tag("\"")),
            take_until("\""),
            tag_no_space("\""),
        ),
        |s| Value::String(s.to_string()),
    )(s)
}

fn parse_variable(s: &str) -> Res<Value> {
    let allowed_values =
        |s| take_while1(|s: char| s.is_alphanumeric() || s == '_')(s);
    map_parser(
        verify(allowed_values, |s: &str| {
            s.chars().next().filter(|c| c.is_alphabetic()).is_some()
        }),
        map(
            verify(all_consuming(allowed_values), |s: &str| {
                !check_reserved_keyword(&[s])
            }),
            |s: &str| Value::Variable(s.to_string()),
        ),
    )(s)
}
fn parse_constant(s: &str) -> Res<Value> {
    map(one_of(MathConstants::get_symbols()), Value::Const)(s)
}

fn parse_block_paren(s: &str) -> Res<Value> {
    let parser = |p| many1(parse_value)(p);
    parse_paren(parser)(s)
}

fn parse_paren<'a, F>(parser: F) -> impl Fn(&'a str) -> Res<Value>
where
    F: Fn(&'a str) -> Res<Vec<Value>>,
{
    move |s| {
        delimited(
            tag_no_space("("),
            map(&parser, Value::BlockParen),
            cut(tag_no_space(")")),
        )(s)
    }
}

fn parse_fn(s: &str) -> Res<Value> {
    let parser = |p| separated_list0(tag_no_space(","), parse_value)(p);
    map(
        separated_pair(parse_paren(parser), tag("=>"), parse_block),
        |(parameters, exprs)| Value::Function {
            parameters: Box::new(parameters),
            exprs,
        },
    )(s)
}

fn parse_builtin_fn(s: &str) -> Res<Value> {
    fn parse_builtin<'a>(
        fn_type: BuiltInFunctionType,
    ) -> impl Fn(&'a str) -> Res<Value> {
        move |s: &str| {
            map(
                preceded(
                    tag_no_space_no_case(fn_type.as_str()),
                    parse_block_paren,
                ),
                |expr| Value::BuiltInFunction { fn_type, expr: Box::new(expr) },
            )(s)
        }
    }
    alt((
        parse_builtin(BuiltInFunctionType::Sqrt),
        parse_builtin(BuiltInFunctionType::Abs),
        parse_builtin(BuiltInFunctionType::Ln),
        parse_builtin(BuiltInFunctionType::Log),
        parse_builtin(BuiltInFunctionType::Sin),
        parse_builtin(BuiltInFunctionType::Cos),
        parse_builtin(BuiltInFunctionType::Tan),
        parse_builtin(BuiltInFunctionType::Println),
        parse_builtin(BuiltInFunctionType::Print),
        parse_builtin(BuiltInFunctionType::Length),
        parse_builtin(BuiltInFunctionType::Include),
    ))(s)
}

fn parse_array(s: &str) -> Res<Value> {
    map(
        preceded(
            tag_no_space("["),
            terminated(
                separated_list0(tag_no_space(","), parse_value),
                preceded(multispace0, tag("]")),
            ),
        ),
        Value::Array,
    )(s)
}
fn parse_array_access(s: &str) -> Res<Value> {
    map(
        pair(
            alt((parse_variable, parse_array, parse_string)),
            preceded(
                terminated(tag("["), multispace0),
                terminated(parse_value, preceded(multispace0, tag("]"))),
            ),
        ),
        |(arr, idx)| Value::ArrayAccess {
            arr: Box::new(arr),
            index: Box::new(idx),
        },
    )(s)
}

fn parse_value(s: &str) -> Res<Value> {
    preceded(
        multispace0,
        terminated(
            alt((
                parse_array_access,
                parse_array,
                parse_string,
                parse_block_paren,
                parse_operation,
                parse_number,
                parse_bool,
                parse_builtin_fn,
                parse_variable,
                parse_constant,
            )),
            opt(comments),
        ),
    )(s)
}

fn parse_operation(s: &str) -> Res<Value> {
    fn parse_op<'a>(operation: Operator) -> impl Fn(&'a str) -> Res<Value> {
        move |s| {
            map(tag_no_space(operation.as_str()), |_| {
                Value::Operation(operation)
            })(s)
        }
    }
    alt((
        parse_op(Operator::Pow),
        parse_op(Operator::Mult),
        parse_op(Operator::Mod),
        parse_op(Operator::Div),
        parse_op(Operator::Add),
        parse_op(Operator::Subtr),
        parse_op(Operator::LessOrEqual),
        parse_op(Operator::GreaterOrEqual),
        parse_op(Operator::Less),
        parse_op(Operator::Greater),
        parse_op(Operator::Equal),
        parse_op(Operator::NotEqual),
        parse_op(Operator::Not),
        parse_op(Operator::And),
        parse_op(Operator::Or),
    ))(s)
}

fn parse_expression(s: &str) -> Res<Value> {
    map_parser(
        parse_multiline,
        map(many1(preceded(opt(comments), parse_value)), Value::Expression),
    )(s)
}

fn parse_simple_instruction(s: &str) -> Res<Value> {
    alt((
        map(
            separated_pair(
                alt((parse_array_access, parse_variable)),
                tag_no_space("="),
                alt((parse_fn, parse_expression)),
            ),
            |(name, expr)| Value::VariableExpr {
                name: Box::new(name),
                expr: Box::new(expr),
            },
        ),
        parse_expression,
    ))(s)
}

fn parse_if_statement(s: &str) -> Res<Value> {
    map(
        preceded(
            tag_no_space(IF),
            tuple((
                parse_block_paren,
                parse_block,
                opt(preceded(
                    tag_no_space(ELSE),
                    alt((map(parse_if_statement, |v| vec![v]), parse_block)),
                )),
            )),
        ),
        |(cond, exprs, else_expr)| Value::IfExpr {
            cond: Box::new(cond),
            exprs,
            else_expr,
        },
    )(s)
}
fn parse_multiline(s: &str) -> Res<&str> {
    alt((
        preceded(
            tag_no_space(MULTILINE),
            delimited(tag_no_space("{"), take_until("}"), tag_no_space("}")),
        ),
        alt((take_until("\n"), rest)),
    ))(s)
}
fn parse_while_statement(s: &str) -> Res<Value> {
    map(
        preceded(tag_no_space(WHILE), pair(parse_block_paren, parse_block)),
        |(cond, exprs)| Value::WhileExpr { cond: Box::new(cond), exprs },
    )(s)
}

fn parse_block(s: &str) -> Res<Vec<Value>> {
    preceded(
        tag_no_space("{"),
        terminated(parse_instructions, tag_no_space("}")),
    )(s)
}

pub(super) fn parse_instructions(instructions: &str) -> Res<Vec<Value>> {
    terminated(
        many1(preceded(
            opt(comments),
            alt((
                parse_fn,
                parse_while_statement,
                parse_if_statement,
                parse_simple_instruction,
            )),
        )),
        opt(comments),
    )(instructions)
}

#[cfg(test)]
mod test {
    use crate::adana_script::{
        parser::parse_instructions, BuiltInFunctionType, Value,
    };

    use super::parse_multiline;

    #[test]
    fn test_parse_multiline() {
        let (rest, _result) = parse_multiline(
            r#"
        multiline 
        {
            2*(3/4.-12%5 +7^9) -6/12.*4 / 
            sqrt(2*(3/4.-12%5 +7^9) --6/12.*4) + 
            abs(-2*(3/4.-12%5 +7^9) -6/12.*4 / sqrt(5))
        }
        "#,
        )
        .unwrap();
        assert!(rest.is_empty());
    }
    #[test]
    fn test_parse_fn() {
        let (rest, result) = parse_instructions(
            r#"
            z = (x) => {
                x = 0
             }
        "#,
        )
        .unwrap();
        assert_eq!(
            result,
            vec![Value::VariableExpr {
                name: Box::new(Value::Variable("z".to_string(),)),
                expr: Box::new(Value::Function {
                    parameters: Box::new(Value::BlockParen(vec![
                        Value::Variable("x".to_string(),),
                    ],)),
                    exprs: vec![Value::VariableExpr {
                        name: Box::new(Value::Variable("x".to_string(),)),
                        expr: Box::new(Value::Expression(vec![
                            Value::Integer(0,),
                        ],)),
                    },],
                }),
            },]
        );
        assert!(rest.trim().is_empty());
        let (rest, result) = parse_instructions(
            r#"
             (x, y) => {
                x = 0
             }
        "#,
        )
        .unwrap();
        assert_eq!(
            result,
            vec![Value::Function {
                parameters: Box::new(Value::BlockParen(vec![
                    Value::Variable("x".to_string(),),
                    Value::Variable("y".to_string(),),
                ],)),
                exprs: vec![Value::VariableExpr {
                    name: Box::new(Value::Variable("x".to_string(),)),
                    expr: Box::new(Value::Expression(
                        vec![Value::Integer(0,),],
                    )),
                },],
            },]
        );
        assert!(rest.trim().is_empty());
        let (rest, result) = parse_instructions(
            r#"
             (x, y) => {
                x = 0
                println("hello")
             }
        "#,
        )
        .unwrap();
        assert_eq!(
            result,
            vec![Value::Function {
                parameters: Box::new(Value::BlockParen(vec![
                    Value::Variable("x".to_string(),),
                    Value::Variable("y".to_string(),),
                ],)),
                exprs: vec![
                    Value::VariableExpr {
                        name: Box::new(Value::Variable("x".to_string(),)),
                        expr: Box::new(Value::Expression(vec![
                            Value::Integer(0,),
                        ],)),
                    },
                    Value::Expression(vec![Value::BuiltInFunction {
                        fn_type: BuiltInFunctionType::Println,
                        expr: Box::new(Value::BlockParen(vec![Value::String(
                            format!("hello")
                        )]))
                    }])
                ],
            },]
        );
        assert!(rest.trim().is_empty());
    }
}
