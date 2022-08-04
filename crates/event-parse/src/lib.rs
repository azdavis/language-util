//! Event-based parsers. Designed to be paired with libraries like `rowan`.
//!
//! To use this library:
//! 1. Define your own enum, perhaps called `SyntaxKind`, which includes all of
//!    the kinds of tokens and syntactic constructs found in your language,
//!    including 'trivia' like comments and whitespace.
//! 2. Implement the following traits for the enum:
//!    - [`Copy`]
//!    - [`Eq`]
//!    - [`Triviable`]
//!    - [`fmt::Display`]
//! 3. Define a lexer which transforms an input string into a vector of
//!    contiguous [`Token`]s using this `SyntaxKind`.
//! 4. Define your language's grammar with functions operating on a [`Parser`].
//! 5. Call [`Parser::finish`] when done, and feed it a suitable [`Sink`] for
//!    the collected parsing events.
//!
//! A similar approach is used in [rust-analyzer][1].
//!
//! [1]: https://github.com/rust-analyzer/rust-analyzer

#![deny(missing_debug_implementations, missing_docs, rust_2018_idioms)]

#[cfg(feature = "rowan")]
pub mod rowan_sink;

use drop_bomb::DropBomb;
use std::fmt;
use token::{Token, Triviable};

/// A event-based parser.
#[derive(Debug)]
pub struct Parser<'input, K> {
  tokens: &'input [Token<'input, K>],
  tok_idx: usize,
  events: Vec<Option<Event<K>>>,
}

impl<'input, K> Parser<'input, K> {
  /// Returns a new parser for the given tokens.
  pub fn new(tokens: &'input [Token<'input, K>]) -> Self {
    Self {
      tokens,
      tok_idx: 0,
      events: Vec::new(),
    }
  }

  /// Starts parsing a syntax construct.
  ///
  /// The returned [`Entered`] must eventually be passed to [`Parser::exit`] or
  /// [`Parser::abandon`]. If it is not, it will panic when dropped.
  ///
  /// `Entered`s returned from `enter` should be consumed with `exit` or
  /// `abandon` in a FIFO manner. That is, the first most recently created
  /// `Entered` should be the first one to be consumed. (Might be more like
  /// first-out first-in in this case actually.)
  ///
  /// If this invariant isn't upheld, as in e.g.
  ///
  /// ```ignore
  /// let e1 = p.enter();
  /// let e2 = p.enter();
  /// p.exit(k, e1);
  /// ```
  ///
  /// then Weird Things might happen.
  pub fn enter(&mut self) -> Entered {
    let ev_idx = self.events.len();
    self.events.push(None);
    Entered {
      bomb: DropBomb::new("Entered markers must be exited"),
      ev_idx,
      tok_idx: self.tok_idx,
    }
  }

  /// Abandons parsing a syntax construct.
  ///
  /// The events recorded since this syntax construct began, if any, will belong
  /// to the parent.
  pub fn abandon(&mut self, mut en: Entered) {
    en.bomb.defuse();
    assert!(self.events[en.ev_idx].is_none());
  }

  /// Finishes parsing a syntax construct.
  pub fn exit(&mut self, mut en: Entered, kind: K) -> Exited {
    en.bomb.defuse();
    let ev = &mut self.events[en.ev_idx];
    assert!(ev.is_none());
    *ev = Some(Event::Enter(kind, None));
    self.events.push(Some(Event::Exit));
    Exited {
      ev_idx: en.ev_idx,
      is_empty: self.tok_idx == en.tok_idx,
    }
  }

  /// Starts parsing a syntax construct and makes it the parent of the given
  /// completed node.
  ///
  /// Consider an expression grammar `<expr> ::= <int> | <expr> + <expr>`. When
  /// we see an `<int>`, we enter and exit an `<expr>` node for it. But then
  /// we see the `+` and realize the completed `<expr>` node for the int should
  /// be the child of a node for the `+`. That's when this function comes in.
  pub fn precede(&mut self, ex: Exited) -> Entered {
    let ret = self.enter();
    match self.events[ex.ev_idx] {
      Some(Event::Enter(_, ref mut parent)) => {
        assert!(parent.is_none());
        *parent = Some(ret.ev_idx);
      }
      ref ev => unreachable!("{:?} preceded {:?}, not Enter", ex, ev),
    }
    ret
  }
}

impl<'input, K> Parser<'input, K>
where
  K: Copy + Triviable,
{
  /// Returns the token after the "current" token, or `None` if the parser is
  /// out of tokens.
  ///
  /// Equivalent to `self.peek_n(0)`. See [`Parser::peek_n`].
  pub fn peek(&mut self) -> Option<Token<'input, K>> {
    while let Some(&tok) = self.tokens.get(self.tok_idx) {
      if tok.kind.is_trivia() {
        self.tok_idx += 1;
      } else {
        return Some(tok);
      }
    }
    None
  }

  /// Returns the token `n` tokens in front of the current token, or `None` if
  /// there is no such token.
  ///
  /// The current token is the first token not yet consumed for which
  /// [`Triviable::is_trivia`] returns `true`; thus, if this returns
  /// `Some(tok)`, then `tok.kind.is_trivia()` is `false`.
  pub fn peek_n(&mut self, n: usize) -> Option<Token<'input, K>> {
    let mut ret = self.peek();
    let old_tok_idx = self.tok_idx;
    for _ in 0..n {
      self.tok_idx += 1;
      ret = self.peek();
    }
    self.tok_idx = old_tok_idx;
    ret
  }

  /// Consumes and returns the current token.
  ///
  /// Panics if there are no more tokens, i.e. if [`Parser::peek`] would return
  /// `None` just prior to calling this.
  ///
  /// This is often used after calling [`Parser::at`] to verify some expected
  /// token was present.
  pub fn bump(&mut self) -> Token<'input, K> {
    let ret = self.peek().expect("bump with no tokens");
    self.events.push(Some(Event::Token));
    self.tok_idx += 1;
    ret
  }

  /// Records an error at the current token, with an "expected `desc`" error
  /// message.
  pub fn error(&mut self, desc: &'static str) {
    self.error_(Expected::Custom(desc))
  }

  fn error_(&mut self, expected: Expected<K>) {
    self.events.push(Some(Event::Error(expected)));
  }

  fn eat_trivia(&mut self, sink: &mut dyn Sink<K>) {
    while let Some(&tok) = self.tokens.get(self.tok_idx) {
      if !tok.kind.is_trivia() {
        break;
      }
      sink.token(tok);
      self.tok_idx += 1;
    }
  }

  /// Finishes parsing, and writes the parsed tree into the `sink`.
  pub fn finish(mut self, sink: &mut dyn Sink<K>) {
    self.tok_idx = 0;
    let mut kinds = Vec::new();
    let mut levels: usize = 0;
    for idx in 0..self.events.len() {
      let ev = match self.events[idx].take() {
        Some(ev) => ev,
        None => continue,
      };
      match ev {
        Event::Enter(kind, mut parent) => {
          assert!(kinds.is_empty());
          kinds.push(kind);
          while let Some(p) = parent {
            match self.events[p].take() {
              Some(Event::Enter(kind, new_parent)) => {
                kinds.push(kind);
                parent = new_parent;
              }
              // abandoned precede
              None => break,
              ev => unreachable!("{:?} was {:?}, not Enter", parent, ev),
            }
          }
          for kind in kinds.drain(..).rev() {
            // keep as much trivia as possible outside of what we're entering.
            if levels != 0 {
              self.eat_trivia(sink);
            }
            sink.enter(kind);
            levels += 1;
          }
        }
        Event::Exit => {
          sink.exit();
          levels -= 1;
          // keep as much trivia as possible outside of top-level items.
          if levels == 1 {
            self.eat_trivia(sink);
          }
        }
        Event::Token => {
          self.eat_trivia(sink);
          sink.token(self.tokens[self.tok_idx]);
          self.tok_idx += 1;
        }
        Event::Error(expected) => sink.error(expected),
      }
    }
    assert_eq!(levels, 0);
  }
}

impl<'input, K> Parser<'input, K>
where
  K: Copy + Triviable + Eq,
{
  /// Returns whether the current token has the given `kind`.
  pub fn at(&mut self, kind: K) -> bool {
    self.at_n(0, kind)
  }

  /// Returns whether the token `n` ahead has the given `kind`.
  pub fn at_n(&mut self, n: usize, kind: K) -> bool {
    self.peek_n(n).map_or(false, |tok| tok.kind == kind)
  }

  /// If the current token's kind is `kind`, then this consumes it, else this
  /// errors. Returns the token if it was eaten.
  pub fn eat(&mut self, kind: K) -> Option<Token<'input, K>> {
    if self.at(kind) {
      Some(self.bump())
    } else {
      self.error_(Expected::Kind(kind));
      None
    }
  }
}

/// A marker for a syntax construct that is mid-parse. If this is not consumed
/// by a [`Parser`], it will panic when dropped.
#[derive(Debug)]
pub struct Entered {
  bomb: DropBomb,
  ev_idx: usize,
  tok_idx: usize,
}

/// A marker for a syntax construct that has been fully parsed.
///
/// We let this be `Copy` so we can do things like this:
/// ```ignore
/// let mut ex: Exited = ...;
/// loop {
///   let en = p.precede(ex);
///   if ... {
///     ...;
///     ex = p.exit(en, ...);
///   } else {
///     p.abandon(en);
///     return Some(ex);
///   }
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Exited {
  ev_idx: usize,
  is_empty: bool,
}

impl Exited {
  /// Returns whether there are no tokens in the node closed by this [`Exited`].
  pub fn is_empty(&self) -> bool {
    self.is_empty
  }
}

/// Types which can construct a syntax tree.
pub trait Sink<K> {
  /// Enters a syntax construct with the given kind.
  fn enter(&mut self, kind: K);
  /// Adds a token to the given syntax construct.
  fn token(&mut self, token: Token<'_, K>);
  /// Exits a syntax construct.
  fn exit(&mut self);
  /// Reports an error.
  fn error(&mut self, expected: Expected<K>);
}

/// Something expected.
#[derive(Debug)]
pub enum Expected<K> {
  /// A kind.
  Kind(K),
  /// A custom description.
  Custom(&'static str),
}

impl<K> fmt::Display for Expected<K>
where
  K: fmt::Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("expected ")?;
    match self {
      Expected::Kind(k) => k.fmt(f),
      Expected::Custom(d) => f.write_str(d),
    }
  }
}

enum Event<K> {
  Enter(K, Option<usize>),
  Token,
  Exit,
  Error(Expected<K>),
}

impl<K> fmt::Debug for Event<K> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Event::Enter(_, n) => f.debug_tuple("Enter").field(n).finish(),
      Event::Token => f.debug_tuple("Token").finish(),
      Event::Exit => f.debug_tuple("Exit").finish(),
      Event::Error(_) => f.debug_tuple("Error").finish(),
    }
  }
}

#[test]
fn event_size() {
  let ev = std::mem::size_of::<Event<()>>();
  let op_ev = std::mem::size_of::<Option<Event<()>>>();
  assert_eq!(ev, op_ev)
}
