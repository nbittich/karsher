use std::collections::BTreeMap;

use serial_test::serial;

use crate::adana_script::{compute, Primitive};

#[test]
#[serial]
fn test_simple_file() {
    let file_path = r#"
    include("file_tests/test1.adana")
    "#;
    let mut ctx = BTreeMap::new();
    let r = compute(file_path, &mut ctx).unwrap();
    assert_eq!(
        &BTreeMap::from([
            ("a".to_string(), Primitive::Int(25)),
            ("b".to_string(), Primitive::Bool(true)),
            ("c".to_string(), Primitive::Bool(true)),
            ("d".to_string(), Primitive::Int(150)),
        ]),
        &ctx
    );
    assert_eq!(Primitive::Int(150), r);
}

#[test]
#[serial]
fn test_if_statement() {
    let file_path = r#"
    include("file_tests/test2.adana")
    "#;
    let mut ctx = BTreeMap::new();
    let r = compute(file_path, &mut ctx).unwrap();
    assert_eq!(
        &BTreeMap::from([
            ("a".to_string(), Primitive::Int(25)),
            ("b".to_string(), Primitive::Int(12)),
            ("c".to_string(), Primitive::Int(20)),
            ("x".to_string(), Primitive::Int(15)),
            ("r".to_string(), Primitive::Int(11)),
            ("z".to_string(), Primitive::Int(18)),
        ]),
        &ctx
    );
    assert_eq!(Primitive::Int(20), r);
}
#[test]
#[serial]
fn test_while_statement() {
    let file_path = r#"
    include("file_tests/testfib.adana")
    "#;
    let mut ctx = BTreeMap::new();

    fn fib(mut n: i128) -> i128 {
        let mut a = 0;
        let mut b = 1;
        let mut c = n;
        while n > 1 {
            c = a + b;
            a = b;
            b = c;
            n = n - 1;
        }
        c
    }

    for n in 0..=10 {
        ctx.insert("n".to_string(), Primitive::Int(n));
        let r = compute(file_path, &mut ctx).unwrap();
        let fibonacci = fib(n);
        assert_eq!(Some(&Primitive::Int(fibonacci)), ctx.get("c"));
        assert_eq!(Primitive::Int(fibonacci), r);
        if fibonacci < 55 {
            assert_eq!(
                Some(&Primitive::String(format!(
                    "this is a complex program: {fibonacci}"
                ))),
                ctx.get("x")
            );
        }
    }
}

#[test]
#[serial]
fn test_nested_file() {
    let file_path = r#"
    include("file_tests/test_nested.adana")
    "#;
    let mut ctx = BTreeMap::new();
    let r = compute(file_path, &mut ctx).unwrap();

    assert_eq!(
        &BTreeMap::from([
            ("a".to_string(), Primitive::Int(0)),
            ("b".to_string(), Primitive::Int(240)),
            ("x".to_string(), Primitive::Int(50)),
            ("s".to_string(), Primitive::String("mod 3".to_string())),
            ("z".to_string(), Primitive::String("mod 1".to_string())),
        ]),
        &ctx
    );
    assert_eq!(Primitive::Int(240), r);
}

#[test]
#[serial]
fn test_fizz_buzz() {
    let file_path = r#"
    include("file_tests/test_fizzbuzz.adana")
    "#;
    let mut ctx = BTreeMap::new();
    let _ = compute(file_path, &mut ctx);

    assert_eq!(
        Some(&Primitive::String("100 = Buzz".to_string())),
        ctx.get("text")
    );
}

#[test]
#[serial]
fn test_includes() {
    let mut ctx = BTreeMap::new();
    let file_path = r#"
    include("file_tests/includes/reverse.adana")
"#;
    let _ = compute(file_path, &mut ctx).unwrap();
    dbg!("wesh", &ctx);
    assert_eq!(
        ctx,
        BTreeMap::from([
            ("a".to_string(), Primitive::Int(144,)),
            (
                "arr".to_string(),
                Primitive::Array(vec![
                    Primitive::Int(1,),
                    Primitive::Int(2,),
                    Primitive::Int(3,),
                ],)
            ),
            (
                "arr2".to_string(),
                Primitive::Array(vec![
                    Primitive::Int(4,),
                    Primitive::Int(5,),
                    Primitive::Int(6,),
                ],)
            ),
            ("b".to_string(), Primitive::Int(233,)),
            ("bfr".to_string(), Primitive::Int(2,)),
            ("c".to_string(), Primitive::Int(233,)),
            ("n".to_string(), Primitive::Int(1,)),
            ("x".to_string(), Primitive::Int(4,))
        ]),
    );
}

#[test]
#[serial]
fn test_multiline_file() {
    let file_path = r#"
    include("file_tests/test_multiline.adana")
"#;
    let mut ctx = BTreeMap::new();
    let _ = compute(file_path, &mut ctx);

    assert_eq!(Some(&Primitive::Int(77)), ctx.get("s"));
    assert_eq!(
        Some(&Primitive::String(
            "\n    multiline\n    sin v\n    text\n  ".to_string()
        )),
        ctx.get("v")
    );
    assert_eq!(
        Some(&Primitive::String("sincostanmultiline".to_string())),
        ctx.get("x")
    );
    assert_eq!(Some(&Primitive::Int(1)), ctx.get("xy"));
    assert_eq!(Some(&Primitive::Int(2)), ctx.get("ze"));
    assert_eq!(Some(&Primitive::Int(3)), ctx.get("de"));
}

#[test]
#[serial]
fn test_if_else_file() {
    let file_path = r#"
    include("file_tests/test_if_else.adana")
"#;
    let mut ctx = BTreeMap::new();
    ctx.insert("count".to_string(), Primitive::Int(102));
    let _ = compute(file_path, &mut ctx);

    assert_eq!(ctx.get("count"), Some(&Primitive::Int(101)));
    let _ = compute(file_path, &mut ctx);
    assert_eq!(ctx.get("count"), Some(&Primitive::Int(51)));
    let file_path = r#"
    include("file_tests/test_fizzbuzz_else.adana")
"#;
    let mut ctx = BTreeMap::new();
    let _ = compute(file_path, &mut ctx);

    assert_eq!(Some(&Primitive::String("".to_string())), ctx.get("text"));
    assert_eq!(Some(&Primitive::Int(101)), ctx.get("count"));
}
