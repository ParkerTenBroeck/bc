use std::{
    collections::VecDeque,
    default,
    marker::PhantomData,
    mem::{discriminant, Discriminant},
};

use lalrpop_util::lalrpop_mod;

#[derive(Clone, Copy, Debug)]
enum State {
    Default,

    L,
    Le,
    Len,
    Leng,
    Lengt,
    Length,
    LengthColon,

    ArrLength,
    ArrayStart,
    ArrayInteger,
    ArrayFirst,
    SubOneStart,
    SubOneCarry,
    ReturnToNum,
    CheckOneStart,
    CheckOne,
    CheckOneC1,
    ArrayLengthNonZero,
    ArrayStartZero,
    ArrayStartZeroEnd,
}
/// Skipping "Array: " because Im lazy
/// <test_case> ::= <array_length> <int_array>
/// <array_length> ::= "Length: " <integer>
/// <int_array> ::= "Array: [" <integer> { "," <integer> } "]"
/// <integer> ::= <digit> { <digit> }
/// <digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
struct Grammar {
    // str: &'a str,
    data: Vec<u8>,
    position: usize,

    state: State,
}

impl Grammar {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string().into_bytes(),
            state: State::Default,
            position: 0,
        }
    }

    pub fn matches(&mut self) -> bool {
        loop {
            let c = if let Some(c) = self.data.get_mut(self.position) {
                c
            } else {
                return false;
            };

            enum CursorEvent {
                Next,
                Stay,
                Prev,
            }
            let mut cursor_event = CursorEvent::Next;

            match self.state {
                State::Default => match *c {
                    b'L' => self.state = State::L,
                    _ => return false,
                },
                State::L => match *c {
                    b'e' => self.state = State::Le,
                    _ => return false,
                },
                State::Le => match *c {
                    b'n' => self.state = State::Len,
                    _ => return false,
                },
                State::Len => match *c {
                    b'g' => self.state = State::Leng,
                    _ => return false,
                },
                State::Leng => match *c {
                    b't' => self.state = State::Lengt,
                    _ => return false,
                },
                State::Lengt => match *c {
                    b'h' => self.state = State::Length,
                    _ => return false,
                },
                State::Length => match *c {
                    b':' => self.state = State::LengthColon,
                    _ => return false,
                },
                State::LengthColon => match *c {
                    b' ' => {
                        // s for start :)
                        *c = b'S';
                        self.state = State::ArrLength
                    }
                    _ => return false,
                },
                State::ArrLength => match *c {
                    b'0' => {}
                    b'1'..=b'9' => {
                        self.state = State::ArrayLengthNonZero;
                    }
                    b'[' => {
                        self.state = State::ArrayStartZero;
                    }
                    _ => return false,
                },

                State::ArrayStartZero => match *c {
                    b' ' => self.state = State::ArrayStartZeroEnd,
                    _ => return false,
                },
                State::ArrayStartZeroEnd => match *c {
                    b']' => return true,
                    _ => return false,
                },
                State::ArrayLengthNonZero => match *c {
                    b'0'..=b'9' => {}
                    b'[' => {
                        self.state = State::ArrayStart;
                    }
                    _ => return false,
                },
                State::ArrayStart => match *c {
                    b' ' => self.state = State::ArrayFirst,
                    _ => return false,
                },
                State::ArrayFirst => match *c {
                    b'0'..=b'9' => self.state = State::ArrayInteger,
                    _ => return false,
                },
                State::ArrayInteger => match *c {
                    b'0'..=b'9' => {}
                    b']' => {
                        cursor_event = CursorEvent::Prev;
                        self.state = State::CheckOneStart;
                    }
                    b',' => {
                        // P for position
                        *c = b'P';
                        cursor_event = CursorEvent::Prev;
                        self.state = State::SubOneStart;
                    }
                    _ => return false,
                },
                State::SubOneStart => match *c {
                    b'[' => {
                        self.state = State::SubOneCarry;
                        cursor_event = CursorEvent::Prev;
                    }
                    _ => {
                        cursor_event = CursorEvent::Prev;
                    }
                },
                State::SubOneCarry => match *c {
                    b'0' => {
                        *c = b'9';
                        cursor_event = CursorEvent::Prev;
                    }
                    b'1'..=b'9' => {
                        *c = *c - 1;
                        self.state = State::ReturnToNum;
                    }
                    _ => return false,
                },
                State::ReturnToNum => match *c {
                    b'P' => {
                        // D for done
                        *c = b',';
                        self.state = State::ArrayInteger;
                    }
                    _ => {}
                },
                State::CheckOneStart => match c {
                    b'[' => {
                        self.state = State::CheckOneC1;
                        cursor_event = CursorEvent::Prev;
                    }
                    _ => {
                        cursor_event = CursorEvent::Prev;
                    }
                },
                State::CheckOneC1 => match *c {
                    b'1' => {
                        self.state = State::CheckOne;
                        cursor_event = CursorEvent::Prev
                    }
                    b'S' => return true,
                    _ => return false,
                },
                State::CheckOne => match *c {
                    b'0' => cursor_event = CursorEvent::Prev,
                    b'S' => return true,
                    _ => return false,
                },
            }

            match cursor_event {
                CursorEvent::Next => self.position += 1,
                CursorEvent::Stay => {}
                CursorEvent::Prev => self.position -= 1,
            }
        }
    }
}

#[test]
fn test() {
    assert!(Grammar::new("Length: 5[ 1,2,3,4,5]").matches());

    assert!(Grammar::new("Length: 0[ ]").matches());
    assert!(!Grammar::new("Length: 0[ 1]").matches());

    assert!(!Grammar::new("Length: 10[ ]").matches());
    assert!(Grammar::new("Length: 10[ 1,200,3,4,5,6,7,8,9,10]").matches());

    assert!(!Grammar::new("Length: 3[ 1,2]").matches());
}

#[derive(Debug)]
pub enum BasicOperator {
    Times,
    Div,
    Add,
    Minus,

    Eq,
    Gt,
    Lt,

    Or,
    And,
}

#[derive(Clone, Copy, Debug)]
pub enum BasicValue {
    Numeric(f64),
    Boolean(bool),
}

pub trait Operator<V> {
    fn output(&self) -> &[Discriminant<V>];
    fn input(&self) -> &[Discriminant<V>];
    fn run(&self, output: &mut Vec<V>);
}

impl Operator<BasicValue> for BasicOperator {
    fn run(&self, output: &mut Vec<BasicValue>) {
        let right = output.pop().unwrap();
        let left = output.pop().unwrap();
        let out = match (left, self, right) {
            (BasicValue::Numeric(left), BasicOperator::Times, BasicValue::Numeric(right)) => {
                BasicValue::Numeric(left * right)
            }
            (BasicValue::Numeric(left), BasicOperator::Div, BasicValue::Numeric(right)) => {
                BasicValue::Numeric(left / right)
            }
            (BasicValue::Numeric(left), BasicOperator::Add, BasicValue::Numeric(right)) => {
                BasicValue::Numeric(left + right)
            }
            (BasicValue::Numeric(left), BasicOperator::Minus, BasicValue::Numeric(right)) => {
                BasicValue::Numeric(left - right)
            }

            (BasicValue::Numeric(left), BasicOperator::Eq, BasicValue::Numeric(right)) => {
                BasicValue::Boolean(left == right)
            }
            (BasicValue::Numeric(left), BasicOperator::Gt, BasicValue::Numeric(right)) => {
                BasicValue::Boolean(left > right)
            }
            (BasicValue::Numeric(left), BasicOperator::Lt, BasicValue::Numeric(right)) => {
                BasicValue::Boolean(left < right)
            }

            (BasicValue::Boolean(left), BasicOperator::Or, BasicValue::Boolean(right)) => {
                BasicValue::Boolean(left || right)
            }
            (BasicValue::Boolean(left), BasicOperator::And, BasicValue::Boolean(right)) => {
                BasicValue::Boolean(left && right)
            }
            _ => todo!(),
        };
        output.push(out);
    }

    fn output(&self) -> &'static [Discriminant<BasicValue>] {
        match self {
            BasicOperator::Times
            | BasicOperator::Div
            | BasicOperator::Add
            | BasicOperator::Minus => {
                const V: [Discriminant<BasicValue>; 1] = [discriminant(&BasicValue::Numeric(0.0))];
                &V
            }
            BasicOperator::Eq
            | BasicOperator::Gt
            | BasicOperator::Lt
            | BasicOperator::And
            | BasicOperator::Or => {
                const V: [Discriminant<BasicValue>; 1] =
                    [discriminant(&BasicValue::Boolean(false))];
                &V
            }
        }
    }

    fn input(&self) -> &'static [Discriminant<BasicValue>] {
        match self {
            BasicOperator::Times
            | BasicOperator::Div
            | BasicOperator::Add
            | BasicOperator::Minus
            | BasicOperator::Eq
            | BasicOperator::Gt
            | BasicOperator::Lt => {
                const V: [Discriminant<BasicValue>; 2] = [
                    discriminant(&BasicValue::Numeric(0.0)),
                    discriminant(&BasicValue::Numeric(0.0)),
                ];
                &V
            }
            BasicOperator::And | BasicOperator::Or => {
                const V: [Discriminant<BasicValue>; 2] = [
                    discriminant(&BasicValue::Boolean(false)),
                    discriminant(&BasicValue::Boolean(false)),
                ];
                &V
            }
        }
    }
}

#[derive(Debug)]
pub enum MachineOperation<V, O> {
    Lit(Discriminant<V>),
    Op(O),
}

#[derive(Default, Debug)]
struct Run<V, O: Operator<V>> {
    pub values: VecDeque<V>,
    stack: Vec<V>,
    pub operator: Vec<MachineOperation<V, O>>,
}

impl<V, O: Operator<V>> Run<V, O> {
    pub fn new() -> Self {
        Self {
            operator: Default::default(),
            stack: Default::default(),
            values: Default::default(),
        }
    }

    pub fn run(&mut self) {
        for op in self.operator.iter() {
            match op {
                MachineOperation::Lit(dis) => {
                    let val = self.values.pop_front().unwrap();
                    //debug type check
                    debug_assert_eq!(discriminant(&val), *dis);
                    self.stack.push(val);
                }
                MachineOperation::Op(op) => {
                    op.run(&mut self.stack);
                    //debug type check
                    debug_assert!({
                        let mut val = true;
                        for (got, expec) in self.stack.iter().rev().zip(op.output().iter()) {
                            if discriminant(got) != *expec {
                                val = false;
                                break;
                            }
                        }
                        val
                    });
                }
            }
        }
    }
}

lalrpop_mod!(calc);

#[test]
#[allow(enum_intrinsics_non_enums)]
fn test4() {
    println!("{:?}", discriminant(&12f32));
    let input = "(2+3*4) > 15";
    let mut context = Run::new();
    calc::FinishedParser::new()
        .parse(&mut context, input)
        .unwrap();
    println!("{:#?}", context);
    context.run();
    println!("{:#?}", context);
    println!("{:#?}", context.stack.pop());
}
