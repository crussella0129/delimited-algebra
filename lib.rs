//! # string_algebra
//!
//! A tiny algebra over delimited collections, generalized via traits.
//!
//! Inspired by a Google Sheet defining four operators:
//!   1. join      (List<T>, sep) -> String
//!   2. split     (String, sep) -> List<T>
//!   3. union     (List<Set<T>>) -> Set<T>
//!   4. difference(B, A) -> Set<T>    (items in B not in A)
//!
//! The math: operators 1 & 2 are *serialization* (between memory and strings).
//! Operators 3 & 4 are *set algebra*. Different concerns -> different traits.

use std::collections::HashSet;
use std::hash::Hash;
use std::str::FromStr;

// ============================================================================
// PART 1: SERIALIZATION (join / split)
// ============================================================================
//
// We want functions that work on ANY type T, as long as T can be turned
// into a string (for join) or parsed from a string (for split).
//
// Rust expresses "any type T" with generics: `fn foo<T>(...)`.
// But generics alone are too permissive — without constraints, T could be
// literally anything, including types that can't be displayed or parsed.
//
// Trait BOUNDS let us say "T, but only types that implement these traits."
// Think of bounds like Excel's data validation: T is allowed, but only
// if it satisfies a contract.

/// Join a slice of items into a single string, separated by `sep`.
///
/// The `T: ToString` bound says: T can be any type, AS LONG AS it implements
/// the `ToString` trait. `String`, `&str`, `i32`, `f64` — they all do.
/// Custom structs need to implement `Display` (which auto-implements `ToString`).
pub fn join<T: ToString>(items: &[T], sep: &str) -> String {
    items
        .iter()
        .map(|x| x.to_string())     // T -> String
        .collect::<Vec<_>>()         // Iterator -> Vec<String>
        .join(sep)                   // Vec<String> -> String with separator
}

/// Split a string by `sep` into a vector of T's.
///
/// `T: FromStr` says T must be parseable from a string. The `where` clause
/// at the end is how we attach a SECOND constraint: T's parse error type
/// must be debuggable, so we can panic with a useful message if parsing fails.
///
/// In production code you'd return `Result<Vec<T>, T::Err>` instead of panicking.
/// We're keeping it simple for now.
pub fn split<T>(s: &str, sep: &str) -> Vec<T>
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    s.split(sep)
        .map(|piece| piece.parse::<T>().expect("failed to parse piece"))
        .collect()
}

// ============================================================================
// PART 2: SET ALGEBRA (union / difference)
// ============================================================================
//
// For set operations we need T to be:
//   - `Hash`: so we can put it in a HashSet (O(1) lookup)
//   - `Eq`:   so we can check equality
//   - `Clone`: so we can copy items into the result set
//
// These are three separate traits, combined with `+`. This is parametric
// polymorphism: the SAME function body works for String, i32, your custom
// `FaceId`, anything that meets the contract.

/// Union: collect every unique element appearing in any of the input sets.
///
/// Note: `&[HashSet<T>]` means "a slice of HashSets" — borrowed, not owned.
/// We don't take ownership; we just read. Caller keeps their data.
pub fn union<T>(sets: &[HashSet<T>]) -> HashSet<T>
where
    T: Hash + Eq + Clone,
{
    let mut result = HashSet::new();
    for set in sets {
        for item in set {
            result.insert(item.clone());
        }
    }
    result
}

/// Forward difference: items in `b` that are NOT in `a`.
/// This is your spreadsheet's "give uniques of 2nd" operator.
///
/// Mathematically: B \ A = { x | x ∈ B ∧ x ∉ A }
pub fn difference<T>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T>
where
    T: Hash + Eq + Clone,
{
    b.iter()
        .filter(|x| !a.contains(x))
        .cloned()
        .collect()
}

// ============================================================================
// PART 3: A CONVENIENCE WRAPPER
// ============================================================================
//
// What if we want to mirror the spreadsheet workflow exactly?
// "Take a list of delimited strings, return the union of unique items."
// That's join + split + union composed. Let's give it a name.

/// Take a list of separator-delimited strings, parse each into a set,
/// and return the union of all elements.
///
/// Example: ["red~blue", "blue~green"] with sep "~" -> {red, blue, green}
pub fn union_of_delimited<T>(strings: &[&str], sep: &str) -> HashSet<T>
where
    T: FromStr + Hash + Eq + Clone,
    T::Err: std::fmt::Debug,
{
    let sets: Vec<HashSet<T>> = strings
        .iter()
        .map(|s| split::<T>(s, sep).into_iter().collect())
        .collect();
    union(&sets)
}

// ============================================================================
// TESTS
// ============================================================================
//
// `#[cfg(test)]` means "only compile this module when running tests."
// Run with: cargo test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_strings() {
        let colors = vec!["Red", "Orange", "Yellow"];
        assert_eq!(join(&colors, "~"), "Red~Orange~Yellow");
    }

    #[test]
    fn test_join_numbers() {
        // Same function, different T — this is parametric polymorphism in action.
        let nums = vec![1, 2, 3, 42];
        assert_eq!(join(&nums, ","), "1,2,3,42");
    }

    #[test]
    fn test_split_strings() {
        let result: Vec<String> = split("Red~Orange~Yellow", "~");
        assert_eq!(result, vec!["Red", "Orange", "Yellow"]);
    }

    #[test]
    fn test_split_numbers() {
        let result: Vec<i32> = split("1,2,3,42", ",");
        assert_eq!(result, vec![1, 2, 3, 42]);
    }

    #[test]
    fn test_union_of_color_sets() {
        // This is row 3 from your spreadsheet.
        let s1: HashSet<String> = ["Red", "Orange", "Yellow"].iter().map(|s| s.to_string()).collect();
        let s2: HashSet<String> = ["Yellow", "Green", "Blue"].iter().map(|s| s.to_string()).collect();
        let result = union(&[s1, s2]);

        let expected: HashSet<String> = ["Red", "Orange", "Yellow", "Green", "Blue"]
            .iter().map(|s| s.to_string()).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_forward_difference() {
        // Row 4 from your spreadsheet:
        // A = Red~Orange~Yellow~Green~Blue
        // B = Red~Orange~Yellow~Green~Blue~Purple
        // B \ A = Purple
        let a: HashSet<String> = ["Red", "Orange", "Yellow", "Green", "Blue"]
            .iter().map(|s| s.to_string()).collect();
        let b: HashSet<String> = ["Red", "Orange", "Yellow", "Green", "Blue", "Purple"]
            .iter().map(|s| s.to_string()).collect();

        let result = difference(&a, &b);
        let expected: HashSet<String> = ["Purple"].iter().map(|s| s.to_string()).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_union_of_delimited_end_to_end() {
        // The full spreadsheet workflow in one call.
        let inputs = vec!["Red~Orange~Yellow", "Yellow~Green~Blue", "Purple~Red"];
        let result: HashSet<String> = union_of_delimited(&inputs, "~");

        let expected: HashSet<String> = ["Red", "Orange", "Yellow", "Green", "Blue", "Purple"]
            .iter().map(|s| s.to_string()).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_works_on_integers_too() {
        // Demonstrates that the SAME functions work on a totally different T.
        // This is the "free theorem" payoff: write once, works for any
        // type that satisfies the trait bounds.
        let result: HashSet<i32> = union_of_delimited(&["1,2,3", "3,4,5"], ",");
        let expected: HashSet<i32> = [1, 2, 3, 4, 5].iter().copied().collect();
        assert_eq!(result, expected);
    }
}
