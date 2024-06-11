use std::task::Context;

use crate::parser::ast::{BinOpKind, Expression, Literal, Path, Type, UnaryOpKind};

use super::TypeMap;


pub enum Value{
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    F32(f32),
    F64(f64),

    Bool(bool),
    Char(char),

    Void(()),

    Ref(Path),
    Ptr(Path),

    Array(Vec<Value>),

    Struct(),
    Enum(),
    Union()
}

pub fn const_eval(context: &mut Context, desired_type: &Type, expr: &Expression) -> Value{
    match expr{
        Expression::Path(_) => todo!(),

        Expression::Literal(lit) => {
            match lit{
                Literal::String(_) => todo!(),
                Literal::Char(_) => todo!(),
                Literal::Boolean(value) => {
                    assert_eq!(desired_type, &Type::Bool);
                    Value::Bool(*value)
                },
                Literal::Number(_) => todo!(),
            }
        },
        Expression::Block(_) => todo!("Cannot evaluate block expressions in constant evaluation"),
        Expression::FieldAccess(expr, field) => {
            todo!()
        },
        Expression::MemberFunction(_, _, _) => todo!("Cannot call member functions in constant evaluation"),
        
        Expression::ArrayAccess(contents, index) => todo!(),

        Expression::FunctionCall(_, _) => todo!("Cannot call functions in constant evaluation"),
        Expression::UnaryOp(op, expr) => {
            // let val = const_eval(type_map, desired_type, expr);
            apply_unary_op(expr, *op)
        },
        Expression::BinaryOp(l, op, r) => {
            // apply_binop_op(l, op, r)
            todo!()
        },
        Expression::Assign(_, _) => todo!("Cannot perform assignment in constant evaluation"),
        
        Expression::StructCon(_, _) => todo!(),
        Expression::ArrayCon(exprs) => todo!(),

        Expression::Break(_, _) => todo!("No loop to break out of"),
        Expression::Continue(_) => todo!("No loop to continue in"),
        Expression::Return(_) => todo!("Cannot return not in a function"),
        
        Expression::SizeOf(_) => todo!(),
        Expression::AlignOf(_) => todo!(),
        Expression::Sized(_) => todo!(),
        Expression::OffsetOf(_, _) => todo!(),
        Expression::TypeName(_) => todo!(),
        
    }
}

fn apply_unary_op(value: &Expression, op: UnaryOpKind) -> Value{
    match op{
        UnaryOpKind::Negate => todo!(),
        UnaryOpKind::Deref => todo!(),
        UnaryOpKind::Not => todo!(),
        UnaryOpKind::Ref => todo!(),
        UnaryOpKind::RefMut => todo!(),
    }
}

fn apply_binop_op(l: &Expression, op: BinOpKind, r: &Expression) -> Value{
    match op{
        BinOpKind::Times => todo!(),
        BinOpKind::Divide => todo!(),
        BinOpKind::Modulo => todo!(),
        BinOpKind::Plus => todo!(),
        BinOpKind::Minus => todo!(),
        BinOpKind::ShiftLeft => todo!(),
        BinOpKind::ShiftRight => todo!(),
        BinOpKind::BitAnd => todo!(),
        BinOpKind::BitXor => todo!(),
        BinOpKind::BitOr => todo!(),
        BinOpKind::Eq => todo!(),
        BinOpKind::Neq => todo!(),
        BinOpKind::Gt => todo!(),
        BinOpKind::Lt => todo!(),
        BinOpKind::Gteq => todo!(),
        BinOpKind::Lteq => todo!(),
        BinOpKind::LogicalAnd => todo!(),
        BinOpKind::LogicalOr => todo!(),
    }
}