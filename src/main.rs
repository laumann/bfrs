//! A brainf*ck interpreter
//!
//! Cells are u8 and addition/subtraction is allowed to wrap around. The cells
//! are only infinite to the right, which means attempting to go back '<' beyond
//! the first cell, moves the pointer nowhere
//!
#![feature(collections)]
#![feature(core)]
use std::env;
use std::io;
use std::fs::File;
use std::io::{Write, Read, Bytes, Result};
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

// The state during computation
struct State {
    stack: Vec<Vec<Op>>,
    cells: Vec<u8>,
    ptr: usize,
    skipping: bool
}

impl State {
    fn new() -> State {
        State {
            stack: Vec::new(),
            cells: {
                let mut v = Vec::new();
                v.resize(1024, 0);
                v
            },
            ptr: 0,
            skipping: false
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

fn run<I: Read>(input: I) {
    use Op::*;
    let mut state = State::new();
    let commands = Commands { bytes: input.bytes() };

    println!("Initial state:\n{:?}", state.cells);
    for c in commands {
        match c {
            Ok(cmd) => match cmd {
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
                    let b = state.cells[state.ptr];
                    let _ = io::stdout().write(&[b]);
                }
                JmpFwd => {
                    if state.cells[state.ptr] == 0 {
                        state.skipping = true;
                    } else {
                        // ...
                    }
                }
                JmpBck => {
                    if state.skipping {
                        state.skipping = false
                    } else {

                    }
                }
            },
            Err(e) => {
                println!("{}", e);
                return
            }
        }
    }
    println!("Final state:\n{:?}", state.cells);
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

    run(file);
}
