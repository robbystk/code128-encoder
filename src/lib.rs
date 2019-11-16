use std::iter::Peekable;
use std::fmt::{Debug,Display,Formatter};

/// Encodes a string into a payload for use in ZPL for a code128 barcode
pub fn encode(s: String) -> String {
    encode_tokens(Tokenizer::initialize(s.bytes()))
}

pub struct Tokenizer<S>
    where S: Iterator<Item = u8>
{
    source: Peekable<S>,
}

impl<S> Tokenizer<S>
    where S: Iterator<Item = u8>
{
    pub fn initialize(source: S) -> Self {
        Tokenizer { source: source.peekable() }
    }
}

/// whether the byte represents a digit
fn is_digit(b: &u8) -> bool {
    b'0' <= *b && *b <= b'9'
}

/// whether thy byte represents a lettor or symbol, but _not_ a digit
fn is_letter_or_symbol(b: &u8) -> bool {
    b' ' <= *b && *b < b'0' || b'9' < *b && *b <= b''
}

/// whether the byte is an ascii control character
fn is_ctrl(b: &u8) -> bool {
    *b <= 31
}

fn chunk<A, I, P>(iter: &mut Peekable<I>, predicate: P) -> Vec<A>
where
    I: Iterator<Item = A>,
    P: Fn(&A) -> bool
{
    let mut rv = Vec::new();
    let mut flag = iter.peek().is_some();
    while flag {
        rv.push(iter.next().unwrap());
        flag = match iter.peek() {
            Some(p) => predicate(p),
            None => false,
        }
    }
    return rv;
}
    
impl<S> Iterator for Tokenizer<S>
    where S: Iterator<Item = u8>
{
    type Item = Token;

    fn next(& mut self) -> Option<Token> {
        match self.source.peek() {
            Some(b) => Some({
                let t = match b {
                    b'0'..=b'9' => Token::Digits(convert_vec(chunk(&mut self.source, is_digit))),
                    b' '..=b'' => Token::Chars(convert_vec(chunk(&mut self.source, is_letter_or_symbol))),
                    0..=31 => Token::Controls(convert_vec(chunk(&mut self.source, is_ctrl))),
                    128..=255 => panic!("Illegal character {}", b),
                };
                println!("parsed token: {}", t);
                t
            }),
            None => None,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct StrChar(u8);

impl StrChar {
    fn to_char(&self) -> char {
        self.0 as char
    }
}

impl Display for StrChar {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.to_char())
    }
}

impl Debug for StrChar {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        if is_ctrl(&self.0) {
            write!(f, "({})", self.0)
        } else {
            write!(f, "{}", self.to_char())
        }
    }
}

fn convert_vec(v: Vec<u8>) -> Vec<StrChar> {
    v.iter().map(|c| StrChar(*c)).collect()
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Chars(Vec<StrChar>),
    Digits(Vec<StrChar>),
    Controls(Vec<StrChar>),
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result<(),std::fmt::Error> {
        use Token::*;

        let variant = match self {
            Chars(_) => "Chars",
            Digits(_) => "Digits",
            Controls(_) => "Controls"
        };
        let internal = match self {
            Chars(b) | Digits(b) | Controls(b) => b,
        };
        f.write_str(variant)?;
        f.debug_list().entries(internal.iter()).finish()
    }
}

fn encode_tokens<I>(tokens: I) -> String
where
    I: IntoIterator<Item = Token>
{
    use Token::*;
    use Code::*;

    let mut tokens = tokens.into_iter().peekable();
    let mut payload = String::new();
    let mut prev_code = match tokens.next() {
        Some(Digits(c)) => {
            if c.len() >= 4 {
                payload.push_str(">;");
                if c.len() % 2 != 0 {
                    match tokens.peek() {
                        Some(Chars(_)) | None => {
                            push_bytes(&mut payload, c);
                            payload.insert_str(payload.len() - 1, ">6");
                            B
                        },
                        Some(Controls(_)) => {
                            push_bytes(&mut payload, c);
                            payload.insert_str(payload.len() - 2, ">7");
                            A
                        },
                        Some(Digits(_)) => panic!("Something is wrong"),
                    }
                } else {
                    push_bytes(&mut payload, c);
                    C
                }
            } else {
                match tokens.peek() {
                    Some(Chars(_)) => {
                        payload.push_str(">:");
                        push_bytes(&mut payload, c);
                        B
                    },
                    Some(Controls(_)) => {
                        payload.push_str(">9");
                        push_bytes(&mut payload, c);
                        A
                    },
                    None => {
                        payload.push_str(">;");
                        push_bytes(&mut payload, c);
                        payload.insert_str(payload.len() - 2, ">6");
                        B
                    },
                    Some(Digits(_)) => panic!("Something is wrong"),
                }
            }
        },
        Some(Chars(c)) => {
            payload.push_str(">:");
            push_bytes(&mut payload, c);
            B
        },
        Some(Controls(c)) => {
            payload.push_str(">9");
            push_bytes(&mut payload, c);
            A
        },
        None => B,
    };

    let mut flag = true;
    while flag {
        prev_code = match tokens.next() {
            Some(Digits(mut c)) => {
                if c.len() >= 4 {
                    let len = c.len();
                    if len % 2 != 0 {
                        payload.push(c[0].to_char());
                        println!("adding {:?} to payload", c[0]);
                        c = c[1..len].to_vec();
                    }
                    if len >= 6 {
                        payload.push_str(">5");
                        push_bytes(&mut payload, c);
                        C
                    } else {
                        if tokens.peek().is_some() {
                            push_bytes(&mut payload, c);
                            prev_code
                        } else {
                            payload.push_str(">5");
                            push_bytes(&mut payload, c);
                            C
                        }
                    }
                } else {
                    push_bytes(&mut payload, c);
                    prev_code
                }
            },
            Some(Chars(c)) => match prev_code {
                A => {
                    if c.len() >= 2 {
                        payload.push_str(">6");
                        push_bytes(&mut payload, c);
                        B
                    } else {
                        c.iter().for_each(|b| {
                            // switch to code B for single character
                            payload.push_str(">7");
                            payload.push(b.to_char());
                        });
                        A
                    }
                },
                B => {
                    push_bytes(&mut payload, c);
                    B
                },
                C => {
                    payload.push_str(">6");
                    push_bytes(&mut payload, c);
                    B
                }
            },
            Some(Controls(c)) => match prev_code {
                A => {
                    c.iter().for_each(|b| payload.push(b.to_char()));
                    A
                },
                B => {
                    if c.len() >= 2 {
                        payload.push_str(">7");
                        c.iter().for_each(|b| payload.push_str(format!("{:02}", *b).as_str()));
                        A
                    } else {
                        c.iter().for_each(|b| payload.push_str(format!(">7{:02}", *b).as_str()));
                        B
                    }
                },
                C => {
                    payload.push_str(">5");
                    c.iter().for_each(|b| payload.push_str(format!("{:02}", *b).as_str()));
                    A
                },
            },
            None => {
                flag = false;
                prev_code
            }
        }
    }
    return payload;
}

pub enum Code { A, B, C }

fn push_bytes(s: &mut String, bytes: Vec<StrChar>) {
    bytes.iter().for_each(|c| {
        s.push(c.to_char());
        println!("adding {:?} to payload", c);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_tokenize_a_string() {
        let s = "ABC123".bytes();
        let tokens: Vec<_> = Tokenizer::initialize(s).collect();
        assert_eq!(tokens[0], Token::Chars(b"ABC".iter().map(|c| StrChar(*c)).collect()));
        assert_eq!(tokens[1], Token::Digits(b"123".iter().map(|c| StrChar(*c)).collect()));
    }

    #[test]
    fn test_can_encode_a_string_of_tokens() {
        let tokens = vec!(
            Token::Digits(b"1234".iter().map(|c| StrChar(*c)).collect())
        );
        assert_eq!(">;1234", encode_tokens(tokens).as_str());
    }

    #[test]
    fn test_token_can_be_displayed() {
        let token = Token::Digits(b"1234".iter().map(|c| StrChar(*c)).collect());
        assert_eq!("Digits([1, 2, 3, 4])", format!("{:?}", token));
    }
}
