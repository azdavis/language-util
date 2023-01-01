# language-util

Various utility crates that might be useful when implementing a programming language in Rust.

Many of these crates used to be from [c0ls][] or [Millet][], but they are generic enough that they were moved out of it so that they may be re-used in other language servers.

## `char-name`

Report the name of some punctuation characters.

## `event-parse`

A generic framework for writing event-based parsers. Such parsers are ones that take as input a flat list of tokens, and produce as output a flat list of events. Events describe how to build a structured syntax tree from the flat list of tokens:

- Start a tree node
- Consume some tokens
- Finish the node

This also lets us handle trivia (whitespace, comments) in one place rather than all over the parser.

## `identifier-case`

Turn `PascalCase` into `snake_case` and vice versa.

## `pattern-match`

Determine whether a sequence of patterns is exhaustive or if any of the patterns are unreachable.

## `syntax-gen`

Generates Rust code from an [ungrammar][].

## `text-pos`

Allows translating between byte indices and line-and-character positions in a string.

## `token`

A simple token type (text + token kind) and a simple trait for trivia.

## `topo-sort`

Generic topological sorting. Useful for when you have many interdependent things and would like to know what order to process them in.

## `uniq`

Unique identifiers, and their generation.

## `unwrap-or`

The macro form of `Option::unwrap_or`.

## `uri-db`

A database of URIs. Allows us to turn a URI (heap-allocated, expensive to pass around) into a cheap, integer-sized ID, and also convert that ID back into a URI.

## `paths`

Types for working with paths, notably:

- A wrapper type for `PathBuf` that guarantees the inner `PathBuf` is canonical.
- A type that transforms these canonical path buffers into cheap IDs.

## `str-util`

Some common string utilities, like:

- Small strings, just a re-export of `smol_str::SmolStr`.
- Names, aka non-empty `SmolStr`s.

## `text-size-util`

A wrapper around the `text-size` crate to add some helpers, primarily `WithRange`, a pair of a value and a text range.

## `fmt-util`

A small utility crate for formatting.

## `diagnostic-util`

A small crate defining primarily the overall `Error` type, which Millet reports to a language client.

## `fast-hash`

A thin wrapper over `FxHash{Map, Set}` with some extra helper functions. These types use `FxHasher`, which is a very fast, but not HashDOS-resistant, hashing algorithm used in Firefox and `rustc`.

## `idx`

A utility crate for an `Idx` type, a cheap copyable type that can index into slices.

## `elapsed`

A small utility crate for timing function calls.

## `code-h2-md-map`

Converts a Markdown file with many level 2 headings with inline code, followed by arbitrary Markdown, into a mapping.

Given a file like this:

```md
# Characters

This is information about characters from _Castle in the Sky_.

## `Sheeta`

A girl who lived on a farm until her parents died, after which she was abducted by the government. She fell out of their airship, and was saved by her magic necklace and a boy, Pazu.

## `Pazu`

A boy who worked in the mines, until he met Sheeta after she fell from the sky. They then went on an adventure to the titular castle and defeated their enemy, Muska.

## `Muska`

An agent of the government, who wants Sheeta's necklace and the floating castle's power.
```

This crate will turn it into a mapping like this:

```json
{
  "Sheeta": "A girl who lived on a farm...",
  "Pazu": "A boy who worked in the mines...",
  "Muska": "An agent of the government..."
}
```

[c0ls]: https://github.com/azdavis/c0ls
[millet]: https://github.com/azdavis/millet
[ungrammar]: https://github.com/rust-analyzer/ungrammar
