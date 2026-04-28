# Delimited Alegbra (String Algebra)

A small Rust crate implementing a tiny algebra over delimited collections.
Originally a Google Sheet/Excel doc (https://docs.google.com/spreadsheets/d/1fOlZ3YK-Fk7w4OU3tagHfohqSGkAPGL3CrboVoFcnZA/edit?gid=0#gid=0) defining four operators (join, split, union,
forward-difference); generalized here via traits so the same code works on
strings, numbers, or any type satisfying the bounds.

## The operators

| Operator         | Type signature                              | Meaning                                  |
| ---------------- | ------------------------------------------- | ---------------------------------------- |
| `join`           | `(&[T], &str) -> String`                    | Collection -> delimited string           |
| `split`          | `(&str, &str) -> Vec<T>`                    | Delimited string -> collection           |
| `union`          | `(&[HashSet<T>]) -> HashSet<T>`             | Combine, dedupe                          |
| `difference`     | `(&HashSet<T>, &HashSet<T>) -> HashSet<T>`  | Items in B not in A                      |

## Why this exists

This is the same algebra Docker uses for image layers, Git uses for commits,
Nix uses for derivations, and B-rep CAD kernels use for boolean solid
operations. Different domains, same operators. Once you write the operators
generically, you can apply them anywhere the trait bounds are satisfied.

## Usage

```rust
use string_algebra::{join, split, union, difference, union_of_delimited};
use std::collections::HashSet;

// Serialize / deserialize
let s = join(&["red", "blue"], "~");           // "red~blue"
let v: Vec<String> = split("red~blue", "~");   // ["red", "blue"]

// Set algebra
let a: HashSet<i32> = [1, 2, 3].iter().copied().collect();
let b: HashSet<i32> = [3, 4, 5].iter().copied().collect();
let new_in_b = difference(&a, &b);             // {4, 5}

// End-to-end pipeline
let result: HashSet<String> = union_of_delimited(&["red~blue", "blue~green"], "~");
// {red, blue, green}
```

## Run the tests

```bash
cargo test
```

## Where to take it next

- Add `intersection` and `symmetric_difference` to round out the set algebra
- Implement `Display` and `FromStr` for a custom struct, watch the same
  functions work on it without changes
- Use it on filesystem paths to compute a "tarball layer" between two snapshots
- Apply to face IDs in a B-rep geometry kernel for boolean solid operations
