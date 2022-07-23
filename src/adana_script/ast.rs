use std::ops::Neg;

use slab_tree::{NodeId, Tree};

use crate::prelude::{BTreeMap, Context};

use super::{MathConstants, Operator, Primitive, TreeNodeValue, Value};

fn variable_from_ctx(
    name: &str,
    negate: bool,
    ctx: &mut BTreeMap<String, Primitive>,
) -> anyhow::Result<Primitive> {
    let value = ctx
        .get(name)
        .context(format!("variable {name} not found in ctx"))?
        .as_ref_ok()?
        .clone();

    if cfg!(test) {
        dbg!(&value);
    }
    let primitive = if negate { value.neg() } else { value };

    Ok(primitive)
}

fn filter_op(
    op: Operator,
    operations: &[Value],
) -> impl FnOnce() -> Option<usize> + '_ {
    move || {
        operations.iter().rposition(
            |c| matches!(c, Value::Operation(operator) if operator == &op),
        )
    }
}

pub(super) fn to_ast(
    ctx: &mut BTreeMap<String, Primitive>,
    value: Value,
    tree: &mut Tree<TreeNodeValue>,
    curr_node_id: &Option<NodeId>,
) -> anyhow::Result<Option<NodeId>> {
    fn append_to_current_and_return(
        v: TreeNodeValue,
        tree: &mut Tree<TreeNodeValue>,
        curr_node_id: &Option<NodeId>,
    ) -> anyhow::Result<Option<NodeId>> {
        if let Some(node_id) = curr_node_id {
            let mut node =
                tree.get_mut(*node_id).context("node id does not exist!")?;
            node.append(v);
            Ok(Some(node.node_id()))
        } else if let Some(mut root_node) = tree.root_mut() {
            root_node.append(v);
            Ok(tree.root_id())
        } else {
            Ok(Some(tree.set_root(v)))
        }
    }

    match value {
        Value::Expression(mut operations)
        | Value::BlockParen(mut operations) => {
            if cfg!(test) {
                dbg!(&operations);
            }

            if operations.is_empty() {
                return Ok(None);
            }
            if operations.len() == 1 {
                return to_ast(ctx, operations.remove(0), tree, curr_node_id);
            }

            let op_pos = None
                .or_else(filter_op(Operator::Or, &operations))
                .or_else(filter_op(Operator::And, &operations))
                .or_else(filter_op(Operator::GreaterOrEqual, &operations))
                .or_else(filter_op(Operator::LessOrEqual, &operations))
                .or_else(filter_op(Operator::Greater, &operations))
                .or_else(filter_op(Operator::Less, &operations))
                .or_else(filter_op(Operator::Equal, &operations))
                .or_else(filter_op(Operator::NotEqual, &operations))
                .or_else(filter_op(Operator::Add, &operations))
                .or_else(filter_op(Operator::Subtr, &operations))
                .or_else(filter_op(Operator::Mult, &operations))
                .or_else(filter_op(Operator::Mod, &operations))
                .or_else(filter_op(Operator::Div, &operations))
                .or_else(filter_op(Operator::Pow, &operations))
                .or_else(filter_op(Operator::Not, &operations));

            if let Some(op_pos) = op_pos {
                let mut left: Vec<Value> =
                    operations.drain(0..op_pos).collect();

                let operation = operations.remove(0);

                // handle negation
                if operation == Value::Operation(Operator::Subtr)
                    && matches!(left.last(), Some(Value::Operation(_)))
                {
                    let right_first = match operations.first() {
                        Some(Value::Decimal(d)) => Some(Value::Decimal(-d)),
                        Some(Value::Integer(d)) => Some(Value::Integer(-d)),
                        Some(Value::Variable(d)) => {
                            Some(Value::VariableNegate(d.to_string()))
                        }
                        _ => None,
                    };
                    if let Some(first) = right_first {
                        operations[0] = first;
                        left.append(&mut operations);
                        return to_ast(
                            ctx,
                            Value::BlockParen(left),
                            tree,
                            curr_node_id,
                        );
                    }
                }

                if cfg!(test) {
                    println!("Left => {left:?}");
                    println!("Right => {operation:?}");
                    println!("Op => {operation:?}");
                    println!();
                }

                let curr_node_id = to_ast(ctx, operation, tree, curr_node_id)?;

                to_ast(ctx, Value::BlockParen(left), tree, &curr_node_id)?;
                to_ast(
                    ctx,
                    Value::BlockParen(operations),
                    tree,
                    &curr_node_id,
                )?;

                Ok(curr_node_id)
            } else {
                Err(anyhow::Error::msg("invalid expression!"))
            }
        }

        Value::Operation(operator) => {
            let ops = TreeNodeValue::Ops(operator);
            if let Some(node_id) = curr_node_id {
                let mut node = tree
                    .get_mut(*node_id)
                    .context("node id does not exist!")?;

                let node = node.append(ops);
                Ok(Some(node.node_id()))
            } else if let Some(mut root_node) = tree.root_mut() {
                let node = root_node.append(ops);
                Ok(Some(node.node_id()))
            } else {
                let _ = tree.set_root(ops);
                Ok(tree.root_id())
            }
        }

        Value::Decimal(num) => append_to_current_and_return(
            TreeNodeValue::Primitive(Primitive::Double(num)),
            tree,
            curr_node_id,
        ),
        Value::Integer(num) => append_to_current_and_return(
            TreeNodeValue::Primitive(Primitive::Int(num)),
            tree,
            curr_node_id,
        ),
        Value::Bool(bool_v) => append_to_current_and_return(
            TreeNodeValue::Primitive(Primitive::Bool(bool_v)),
            tree,
            curr_node_id,
        ),
        Value::String(string_v) => append_to_current_and_return(
            TreeNodeValue::Primitive(Primitive::String(string_v)),
            tree,
            curr_node_id,
        ),
        Value::Variable(name) => {
            let value = variable_from_ctx(name.as_str(), false, ctx)?;
            append_to_current_and_return(
                TreeNodeValue::Primitive(value),
                tree,
                curr_node_id,
            )
        }
        Value::VariableNegate(name) => {
            let value = variable_from_ctx(name.as_str(), true, ctx)?;
            append_to_current_and_return(
                TreeNodeValue::Primitive(value),
                tree,
                curr_node_id,
            )
        }
        Value::VariableExpr { name, expr } => {
            anyhow::ensure!(
                tree.root().is_none(),
                "invalid variable assignment "
            );

            let variable_assign_node = if let Value::Variable(n) = *name {
                Ok(TreeNodeValue::VariableAssign(n))
            } else if let Value::ArrayAccess { arr, index } = *name {
                if let (Value::Variable(n), Value::Integer(index)) =
                    (*arr, *index)
                {
                    Ok(TreeNodeValue::VariableArrayAssign {
                        name: n,
                        index: Primitive::Int(index),
                    })
                } else {
                    Err(anyhow::Error::msg("invalid variable expression"))
                }
            } else {
                Err(anyhow::Error::msg("invalid variable expression"))
            }?;

            let node_id = Some(tree.set_root(variable_assign_node));

            let value = *expr;

            let _ = to_ast(ctx, value, tree, &node_id)?
                .context(format!("invalid variable expr {node_id:?}"))?;
            Ok(node_id)
        }
        Value::Const(c) => match c {
            c if c == MathConstants::Pi.get_symbol() => to_ast(
                ctx,
                Value::Decimal(std::f64::consts::PI),
                tree,
                curr_node_id,
            ),
            c if c == MathConstants::EulerNumber.get_symbol() => to_ast(
                ctx,
                Value::Decimal(std::f64::consts::E),
                tree,
                curr_node_id,
            ),
            c if c == MathConstants::Tau.get_symbol() => to_ast(
                ctx,
                Value::Decimal(std::f64::consts::TAU),
                tree,
                curr_node_id,
            ),
            _ => unreachable!("should never happen or it's a bug"),
        },
        Value::BuiltInFunction { fn_type, expr } => {
            let fn_node = TreeNodeValue::BuiltInFunction(fn_type);
            let node_id = if let Some(node_id) = curr_node_id {
                let mut node = tree
                    .get_mut(*node_id)
                    .context("node id does not exist!")?;

                let node = node.append(fn_node);
                Some(node.node_id())
            } else if let Some(mut root_node) = tree.root_mut() {
                let node = root_node.append(fn_node);
                Some(node.node_id())
            } else {
                Some(tree.set_root(fn_node))
            };
            to_ast(ctx, *expr, tree, &node_id)?;
            Ok(node_id)
        }
        v @ Value::IfExpr { cond: _, exprs: _, else_expr: _ } => {
            let if_node = TreeNodeValue::IfExpr(v);
            append_to_current_and_return(if_node, tree, curr_node_id)
        }
        v @ Value::WhileExpr { cond: _, exprs: _ } => {
            let while_node = TreeNodeValue::WhileExpr(v);
            append_to_current_and_return(while_node, tree, curr_node_id)
        }
        Value::Array(arr) => append_to_current_and_return(
            TreeNodeValue::Array(arr),
            tree,
            curr_node_id,
        ),
        Value::ArrayAccess { arr, index } => match (*arr, *index) {
            (v, Value::Integer(idx)) => append_to_current_and_return(
                TreeNodeValue::ArrayAccess {
                    index: Primitive::Int(idx),
                    array: v,
                },
                tree,
                curr_node_id,
            ),
            (v, Value::Variable(idx_var)) => {
                let idx = variable_from_ctx(&idx_var, false, ctx)?;
                append_to_current_and_return(
                    TreeNodeValue::ArrayAccess { index: idx, array: v },
                    tree,
                    curr_node_id,
                )
            }

            (arr, index) => {
                return Err(anyhow::Error::msg(format!(
                    "illegal array access! array => {arr:?}, index=> {index:?}"
                )))
            }
        },
        f @ Value::Function { parameters: _, exprs: _ } => {
            append_to_current_and_return(
                TreeNodeValue::Function(f),
                tree,
                curr_node_id,
            )
        }
    }
}
