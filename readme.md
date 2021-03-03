# language-server-util

Various utility crates that might be useful when implementing a language server.

Most (all?) of these crates used to be from [c0ls][], but they are generic
enough that they were moved out of it so that they may be re-used in other
language servers.

## `char-name`

A helper function that gives an English name to some characters.

## `event-parse`

A generic framework for writing event-based parsers. Such parsers are ones that
take as input a flat list of tokens, and produce as output a flat list of
events. Events describe how to build a structured syntax tree from the flat list
of tokens:

- Start a tree node
- Consume some tokens
- Finish the node

This also lets us handle trivia (whitespace, comments) in one place rather than
all over the parser.

## `identifier-case`

Conversions between various identifier cases, like `snake_case` and
`PascalCase`.

## `syntax-gen`

Generates Rust code from an [ungrammar][].

## `text-pos`

Allows translating between byte indices and line-and-character positions in a
string.

## `topo-sort`

Generic topological sorting. Useful for when you have many interdependent things
and would like to know what order to process them in.

## `unwrap-or`

The macro form of `Option::unwrap_or`.

## `uri-db`

A database of URIs. Allows us to turn a URI (heap-allocated, expensive to pass
around) into a cheap, integer-sized ID, and also convert that ID back into a
URI.

[c0ls]: https://github.com/azdavis/c0ls
[ungrammar]: https://github.com/rust-analyzer/ungrammar
