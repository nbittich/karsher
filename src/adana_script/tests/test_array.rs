use std::collections::BTreeMap;

use serial_test::serial;

use crate::adana_script::{
    compute,
    primitive::{
        Array as Arr,
        Primitive::{Array, Bool, Double, Int, String as Str},
    },
};

#[test]
#[serial]
fn test_simple_array() {
    let file_path = r#"
        include("file_tests/test_simple_array.adana")
    "#;
    let mut ctx = BTreeMap::new();
    let _ = compute(file_path, &mut ctx).unwrap();
    assert_eq!(
        &BTreeMap::from([
            (
                "x".to_string(),
                Array(vec![
                    Str("hello".to_string()),
                    Int(1),
                    Str("World".to_string()),
                    Bool(true)
                ])
            ),
            (
                "y".to_string(),
                Array(vec![
                    Str("hello".to_string()),
                    Int(1),
                    Str("World".to_string()),
                    Bool(true),
                    Str("hello".to_string()),
                    Int(1),
                    Str("World".to_string()),
                    Bool(true),
                    Str("hello".to_string()),
                    Int(1),
                    Str("World".to_string()),
                    Bool(true),
                    Str("hello".to_string()),
                    Int(1),
                    Str("World".to_string()),
                    Bool(true),
                    Str("hello".to_string()),
                    Int(1),
                    Str("World".to_string()),
                    Bool(true)
                ])
            ),
            ("z".to_string(), Str("World".to_string())),
            ("a".to_string(), Bool(true)),
        ]),
        &ctx
    );
}
#[test]
#[serial]
fn test_file_array() {
    let file_path = r#"
    include("file_tests/test_array.adana")
    "#;
    let mut ctx = BTreeMap::new();
    let _ = compute(file_path, &mut ctx).unwrap();
    assert_eq!(ctx.get("arrlen"), Some(&Int(18)));

    let arr = Array(vec![
        Str("a".to_string()),
        Bool(true),
        Str("bababa".to_string()),
        Str("zezezeze".to_string()),
        Int(1),
        Double(2.1),
        Double(3.0),
        Int(69),
        Int(420),
        Str("Yolo".to_string()),
        Bool(true),
        Str("bababa".to_string()),
        Str("zezezeze".to_string()),
        Int(1),
        Double(2.1),
        Double(3.0),
        Int(69),
        Int(420),
    ]);

    let mut copy = arr.clone();
    copy.swap_mem(&mut Str("a".to_string()), &Int(9));

    assert_eq!(ctx.get("arr"), Some(&arr));
    assert_eq!(ctx.get("copy"), Some(&copy));

    let fancy_list = Array(vec![
        Int(1),
        Array(vec![
            Int(2),
            Array(vec![
                Int(3),
                Array(vec![
                    Int(4),
                    Array(vec![
                        Int(5),
                        Array(vec![
                            Int(6),
                            Array(vec![
                                Int(7),
                                Array(vec![
                                    Int(8),
                                    Array(vec![Int(9), Array(vec![])]),
                                ]),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]),
    ]);
    assert_eq!(ctx.get("list"), Some(&fancy_list));

    let res = compute("arr[2]", &mut ctx).unwrap();

    assert_eq!(Str("bababa".to_string()), res)
}

#[test]
#[serial]
fn test_string_array() {
    let file_path = r#"
        include("file_tests/test_string_arr.adana")
    "#;
    let mut ctx = BTreeMap::new();
    let _ = compute(file_path, &mut ctx).unwrap();
    assert_eq!(
        &BTreeMap::from([
            ("v".to_string(), Str("nordine".to_string())),
            ("copy".to_string(), Str("eodrnin".to_string())),
            ("s".to_string(), Str("kekeke".to_string())),
            ("count".to_string(), Int(7)),
            ("i".to_string(), Int(6)),
        ]),
        &ctx
    );
}
