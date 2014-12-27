#![feature(globs)]
#![allow(unused_must_use)]

use std::os;
use std::io::File;
use std::iter::Iterator;
use std::iter::Peekable;
use std::iter::IteratorExt;
use Sugar::*;
use State::*;

#[deriving(Copy)]
enum State { Walk, Bracket, Start(Sugar), Inline(Sugar), Multiline(Sugar, u8) }

#[deriving(Copy)]
enum Sugar { Array, Object }

impl Sugar {
  fn from_char(c: char) -> Sugar { if c == '@' { Array } else { Object } }
  fn start_char(&self) -> char { match *self { Array => '[', Object => '{' } }
  fn end_char(&self) -> char { match *self { Array => ']', Object => '}' } }
}

struct StateStack { top_ix: uint, buffer: [State, ..128] }

impl StateStack {
  fn new() -> StateStack { StateStack { top_ix: 0u, buffer: [Walk, ..128] } }
  fn push(&mut self, state: State) { self.buffer[self.top_ix] = state; self.top_ix += 1 }
  fn pop(&mut self) { self.top_ix -= 1 }
  fn top(&self) -> State {
    if self.top_ix == 0 {
      panic!("StateStack::top is called on empty stack!");
    }
    self.buffer[self.top_ix - 1]
  }
}

fn skip_indent<I: Iterator<char>>(iter: &mut Peekable<char, I>) -> u8 {
  let mut indent = 0u8;
  loop {
    if !iter.peek().map_or(false, |c| c.is_whitespace() && !c.is_control()) {
      return indent;
    }
    indent += 1;
    iter.next();
  }
}

fn write_indent(file: &mut File, indent: u8) {
  for _ in range(0u8, indent) {
    file.write_char(' ');
  }
}

fn process_multiline_newline(state_stack: &mut StateStack, out_file: &mut File, indent: u8) {
  loop {
    match state_stack.top() {
      Multiline(sugar, cur_indent) => {
        if indent < cur_indent {
          out_file.write_char(' ');
          out_file.write_char(sugar.end_char());
        } else {
          out_file.write_char('\n');
          write_indent(out_file, indent);
          if indent == cur_indent {
            out_file.write_char(',');
            out_file.write_char(' ');
          }
          break;
        }
      },
      _ => {
        out_file.write_char('\n');
        write_indent(out_file, indent);
        break;
      }
    }
    state_stack.pop();
  }
}

fn main() {
  let args = os::args();
  let contents = File::open(&Path::new(&args[1])).read_to_string().unwrap();
  let iter = &mut contents.as_slice().chars().peekable();
  let out_file = &mut File::create(&Path::new(&args[2])).unwrap();
  let state_stack = &mut StateStack::new();
  state_stack.push(Walk);
  loop {
    let state = state_stack.top();
    match iter.next() {
      None => break,
      Some('\r') => { /* sorry, no support for ms-dos yet! */ },
      Some(c) if c == '@' || c == '#' => {
        let new_sugar = Sugar::from_char(c);
        match state {
          Inline(sugar) => {
            out_file.write_char(sugar.end_char());
            state_stack.pop();
            state_stack.push(Start(new_sugar));
          },
          _ => {
            state_stack.push(Start(new_sugar));
            if iter.peek().map_or(false, |c| c.is_whitespace() && !c.is_control()) {
              iter.next();
            }
          }
        }
      },
      Some('\n') => {
        let indent = skip_indent(iter);
        match state {
          Multiline(_, _) => process_multiline_newline(state_stack, out_file, indent),
          Start(sugar) => {
            state_stack.pop();
            state_stack.push(Multiline(sugar, indent));
            out_file.write_char('\n');
            write_indent(out_file, indent);
            out_file.write_char(sugar.start_char());
            out_file.write_char(' ');
          },
          Inline(sugar) => {
            out_file.write_char(' ');
            out_file.write_char(sugar.end_char());
            state_stack.pop();
            match state_stack.top() {
              Multiline(_, _) => process_multiline_newline(state_stack, out_file, indent),
              _ => {
                out_file.write_char('\n');
                write_indent(out_file, indent);
              }
            }
          },
          _ => {
            out_file.write_char('\n');
            write_indent(out_file, indent);
          }
        }
      },
      Some(')') => loop {
        match state_stack.top() {
          Bracket => {
            state_stack.pop();
            out_file.write_char(')');
            break;
          },
          Start(sugar) | Inline(sugar) | Multiline(sugar, _) => {
            out_file.write_char(' ');
            out_file.write_char(sugar.end_char());
            state_stack.pop();
          },
          Walk => break
        }
      },
      Some(c) => {
        match state {
          Start(sugar) => {
            if !c.is_whitespace() {
              state_stack.pop();
              state_stack.push(Inline(sugar));
              out_file.write_char(sugar.start_char());
              out_file.write_char(' ');
            }
          },
          _ => {}
        }
        if c == '(' {
          state_stack.push(Bracket);
        }
        out_file.write_char(c);
      }
    }
  }
}
