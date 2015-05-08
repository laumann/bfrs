//! A brainf*ck interpreter
//!
//! Cells are u8 and addition/subtraction is allowed to wrap around. The cells
//! are only infinite to the right, which means attempting to go back '<' beyond
//! the first cell, moves the pointer nowhere
//!
#![feature(collections)]
#![feature(core)]
use std::env;
use std::fs::File;
use std::io::{Read, Bytes, Result};
use std::num::wrapping::OverflowingOps;

// Each operation (read stdin, or ',' is omitted for now
enum Op {
    Inc,    // +
    Dec,    // -
    Next,   // >
    Prev,   // <
    Out,    // .
    JmpFwd, // [
    JmpBck  // ]
}

#[derive(Clone, Debug)]
enum Inst {
    Inc,
    Dec,
    Next,
    Prev,
    Out,
    Loop(Vec<Inst>)
}

// The state during computation
struct State {
    cells: Vec<u8>,
    ptr: usize,
}

impl State {
    fn new() -> State {
        State {
            cells: {
                let mut v = Vec::new();
                v.resize(1024, 0);
                v
            },
            ptr: 0,
        }
    }
}

struct Commands<R> {
    bytes: Bytes<R>
}

impl<R: Read> Iterator for Commands<R> {
    type Item = Result<Op>;

    fn next(&mut self) -> Option<Result<Op>> {
        use Op::*;
        // if read is successful, need to check byte can be mapped (otherwise
        // read the next byte)
        loop {
            let byte = self.bytes.next();
            match byte {
                Some(res) => {
                    if res.is_err() {
                        return Some(Err(res.unwrap_err()))
                    }
                    let r = match res.unwrap() {
                        b'+' => Some(Inc),
                        b'-' => Some(Dec),
                        b'<' => Some(Prev),
                        b'>' => Some(Next),
                        b'.' => Some(Out),
                        b'[' => Some(JmpFwd),
                        b']' => Some(JmpBck),
                        _   => None
                    };
                    if r.is_some() {
                        return Some(Ok(r.unwrap()))
                    }
                }
                None => return None
            }
        }
    }
}

fn parse<I: Read>(input: I) -> Vec<Inst> {
    use Op::*;
    let commands = Commands { bytes: input.bytes() };
    let mut v = Vec::new();
    v.push(Vec::new());

    let mut tos = 0;

    for cmd in commands {
        match cmd {
            Ok(c) => {
                match c {
                    Inc => {
                        v[tos].push(Inst::Inc);
                    },
                    Dec => {
                        v[tos].push(Inst::Dec);
                    },
                    Next => {
                        v[tos].push(Inst::Next);
                    },
                    Prev => {
                        v[tos].push(Inst::Prev);
                    },
                    Out => {
                        v[tos].push(Inst::Out);
                    }
                    JmpFwd => {
                        v.push(Vec::new());
                        tos += 1;
                    },
                    JmpBck => {
                        if tos == 0 {
                            panic!("Unmatched closing loop!");
                        }
                        let mut body = v.remove(tos);
                        body.reverse();
                        tos -= 1;
                        v[tos].push(Inst::Loop(body));
                    }
                }
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    let mut instrs = v.pop().unwrap();
    instrs.reverse();
    instrs
}

fn run(mut instrs: Vec<Inst>) {
    use Inst::*;
    let mut state = State::new();

    while !instrs.is_empty() {
        match instrs.pop().unwrap() {
            Inc => {
                state.cells[state.ptr] = state.cells[state.ptr].overflowing_add(1).0;
            }
            Dec => {
                state.cells[state.ptr] = state.cells[state.ptr].overflowing_sub(1).0;
            }
            Next => {
                state.ptr += 1;
                let len = state.cells.len();
                if state.ptr >= len {
                    state.cells.resize(if len == 0 { 2 } else { len << 1 }, 0);
                }
            }
            Prev => {
                if state.ptr > 0 {
                    state.ptr -= 1;
                }
            }
            Out => {
                print!("{}", state.cells[state.ptr] as char);
            }
            Loop(mut body) => {
                if state.cells[state.ptr] != 0 {
                    instrs.push(Loop(body.clone()));
                    instrs.append(&mut body);
                }
            }
        }
    }
}

fn usage() {}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();

    if args.is_empty() {
        usage();
        return;
    }

    let file = match File::open(args[0].clone()) {
        Ok(file) => file,
        Err(e)   => {
            println!("{}", e);
            return
        }
    };

    run(parse(file));
}
