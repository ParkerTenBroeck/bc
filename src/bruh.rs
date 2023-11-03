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

#[allow(unused)]
fn test2(val: &mut i32) {}
