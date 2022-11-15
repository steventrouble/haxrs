use std::{mem::size_of, sync::mpsc::Sender};

use num_traits::Float;
use pest::Parser;

use crate::windex::{scanner::SearchResult, DataTypeEnum};

/// The default search type to use when no comparator is specified.
const DEFAULT_CMP: Comparator = Comparator::Approx;

/// Parser that parses search queries using the grammar in search.pest.
#[derive(pest_derive::Parser)]
#[grammar = "parser/search.pest"]
struct SearchParser;

/// All the info needed to match against a constant value.
#[derive(Debug)]
pub struct ConstantMatcher {
    as_int: Option<i64>,
    as_float: Option<(f64, i32)>,
}

/// Represents a comparator to use. Values in memory will match if greater than the
/// value that follows the comparator.
pub enum Comparator {
    Approx, // Default
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Neq,
}

impl Comparator {
    pub fn matches_int<T: PartialOrd>(&self, mem: T, val: T) -> bool {
        match self {
            Comparator::Approx => mem == val,
            Comparator::Eq => mem == val,
            Comparator::Neq => mem != val,
            Comparator::Gt => mem > val,
            Comparator::Gte => mem >= val,
            Comparator::Lt => mem < val,
            Comparator::Lte => mem <= val,
        }
    }

    pub fn matches_float<T: Float>(&self, mem: T, val: T, lower: T, upper: T) -> bool {
        match self {
            Comparator::Approx => mem > lower && mem < upper,
            Comparator::Eq => mem >= val - T::min_value() && mem <= val + T::min_value(),
            Comparator::Neq => mem < lower || mem > upper,
            Comparator::Gt => mem > val,
            Comparator::Gte => mem >= val,
            Comparator::Lt => mem < val,
            Comparator::Lte => mem <= val,
        }
    }

    fn from_str(val: &str) -> Comparator {
        match val {
            ">" => Comparator::Gt,
            ">=" => Comparator::Gte,
            "<" => Comparator::Lt,
            "<=" => Comparator::Lte,
            "=" => Comparator::Eq,
            "!=" => Comparator::Neq,
            "~" => Comparator::Approx,
            "" => Comparator::Eq,
            unknown => panic!("Unknown operator {}.", unknown),
        }
    }
}

/// A node in the AST of the search query.
/// Generally, a Node represents a Matcher that can match values in memory.
pub enum Node {
    Constant(ConstantMatcher),
    MatchExpr { op: Comparator, val: Box<Node> },
}

impl Node {
    pub fn as_constant(&self) -> &ConstantMatcher {
        match self {
            Node::Constant(info) => info,
            _unknown => panic!("No constant found for comparator."),
        }
    }

    /// Scans an entire page of memory for things that match.
    pub fn scan_page(&self, tx: &Sender<SearchResult>, mem: &[u8], addr_start: usize) {
        match self {
            Node::Constant(matcher) => scan_page(tx, mem, addr_start, &DEFAULT_CMP, matcher),
            Node::MatchExpr { op, val } => scan_page(tx, mem, addr_start, op, val.as_constant()),
        }
    }

    /// Compares a single value against this matcher.
    pub fn test_single(&self, mem: &[u8], data_type: DataTypeEnum) -> bool {
        match self {
            Node::Constant(matcher) => compare_single(mem, data_type, &DEFAULT_CMP, matcher),
            Node::MatchExpr { op, val } => compare_single(mem, data_type, op, val.as_constant()),
        }
    }
}

/// Scans an entire page of memory for things that match.
fn scan_page(
    tx: &Sender<SearchResult>,
    mem: &[u8],
    addr_start: usize,
    op: &Comparator,
    values: &ConstantMatcher,
) {
    // Cast the memory into 4-byte word arrays
    let i32_arr = bytemuck::try_cast_slice::<u8, i32>(mem).unwrap_or(&[]);
    let f32_arr = bytemuck::try_cast_slice::<u8, f32>(mem).unwrap_or(&[]);
    // And 8-byte dword arrays
    let i64_arr = bytemuck::try_cast_slice::<u8, i64>(mem).unwrap_or(&[]);
    let f64_arr = bytemuck::try_cast_slice::<u8, f64>(mem).unwrap_or(&[]);
    let num_words = i32_arr.len();

    // Loop over each entry and compare it to the search value
    for i32_offset in 0..num_words {
        let i64_offset = i32_offset / 2;

        if let Some(value) = values.as_int {
            // i64
            if i32_offset % 2 == 0 {
                let v = i64_arr[i64_offset];
                if op.matches_int(v, value) {
                    let result = SearchResult {
                        address: i64_offset * size_of::<i64>() + addr_start,
                        data_type: DataTypeEnum::EightBytes,
                        value: v.to_ne_bytes().to_vec(),
                    };
                    tx.send(result).unwrap();
                }
            }

            // i32
            let value = value as i32;
            let v = i32_arr[i32_offset];
            if op.matches_int(v, value) {
                let result = SearchResult {
                    address: i32_offset * size_of::<i32>() + addr_start,
                    data_type: DataTypeEnum::FourBytes,
                    value: v.to_ne_bytes().to_vec(),
                };
                tx.send(result).unwrap();
            }
        }

        if let Some((value, decimal_digits)) = values.as_float {
            // f64
            let lowest_digit = (10.0 as f64).powf(-(decimal_digits as f64));
            if (i32_offset % 2) == 0 {
                let f64_offset = i32_offset / 2;
                let lower_bound = value - lowest_digit * 0.999;
                let upper_bound = value + lowest_digit * 0.999;
                let v = f64_arr[f64_offset];
                if op.matches_float(v, value, lower_bound, upper_bound) {
                    let result = SearchResult {
                        address: f64_offset * size_of::<f64>() + addr_start,
                        data_type: DataTypeEnum::Double,
                        value: v.to_ne_bytes().to_vec(),
                    };
                    tx.send(result).unwrap();
                }
            }

            // f32
            let value = value as f32;
            let lowest_digit = lowest_digit as f32;
            let lower_bound = value - lowest_digit;
            let upper_bound = value + lowest_digit;
            let v = f32_arr[i32_offset];
            if op.matches_float(v, value, lower_bound, upper_bound) {
                let result = SearchResult {
                    address: i32_offset * size_of::<f32>() + addr_start,
                    data_type: DataTypeEnum::Float,
                    value: v.to_ne_bytes().to_vec(),
                };
                tx.send(result).unwrap();
            }
        }
    }
}

fn compare_single(
    mem: &[u8],
    data_type: DataTypeEnum,
    op: &Comparator,
    values: &ConstantMatcher,
) -> bool {
    match data_type {
        DataTypeEnum::FourBytes => {
            if let Some(value) = values.as_int {
                if let Ok(&v) = bytemuck::try_from_bytes::<i32>(mem) {
                    if op.matches_int(v, value as i32) {
                        return true;
                    }
                }
            }
        }
        DataTypeEnum::EightBytes => {
            if let Some(value) = values.as_int {
                if let Ok(&v) = bytemuck::try_from_bytes::<i64>(mem) {
                    if op.matches_int(v, value) {
                        return true;
                    }
                }
            }
        }
        DataTypeEnum::Float => {
            if let Some((value, decimal_digits)) = values.as_float {
                let lowest_digit = (10.0 as f32).powf(-(decimal_digits as f32));
                let value = value as f32;
                let lower_bound = value - lowest_digit;
                let upper_bound = value + lowest_digit;
                if let Ok(&v) = bytemuck::try_from_bytes::<f32>(mem) {
                    if op.matches_float(v, value, lower_bound, upper_bound) {
                        return true;
                    }
                }
            }
        }
        DataTypeEnum::Double => {
            if let Some((value, decimal_digits)) = values.as_float {
                let lowest_digit = (10.0 as f64).powf(-(decimal_digits as f64));
                let lower_bound = value - lowest_digit;
                let upper_bound = value + lowest_digit;
                if let Ok(&v) = bytemuck::try_from_bytes::<f64>(mem) {
                    if op.matches_float(v, value, lower_bound, upper_bound) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Tries to parse a float, and returns (float, precision) if successful.
fn parse_float(val: pest::iterators::Pair<Rule>) -> Option<(f64, i32)> {
    if let Ok(parsed) = val.as_str().parse::<f64>() {
        let mut precision: i32 = 0;
        for pair in val.into_inner() {
            if pair.as_rule() == Rule::TrailingDecimal {
                precision += pair.as_str().len() as i32;
            } else if pair.as_rule() == Rule::FloatPower {
                if let Ok(power) = pair.as_str().parse::<i32>() {
                    precision -= power;
                }
            }
        }
        return Some((parsed, precision));
    }
    return None;
}

/// Builds a matcher from a parsed search query.
fn build_matcher(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::Num => Node::Constant(ConstantMatcher {
            as_int: pair.as_str().parse::<i64>().ok(),
            as_float: parse_float(pair),
        }),
        Rule::MatchExpr => {
            let mut pair = pair.into_inner();
            let first = pair.next().unwrap();
            let mut op = Comparator::Approx;
            let val: Box<Node>;
            if first.as_rule() == Rule::Comparator {
                op = Comparator::from_str(first.as_str());
                val = Box::new(build_matcher(pair.next().unwrap()));
            } else {
                val = Box::new(build_matcher(first))
            }
            Node::MatchExpr { op, val }
        }
        unknown => panic!("Unexpected syntax element {:?}.", unknown),
    }
}

/// Parses a search query and returns a matcher representing that query.
pub fn parse(input: &str) -> Result<Node, String> {
    let pairs = SearchParser::parse(Rule::SearchQuery, input);
    match pairs {
        Ok(pairs) => {
            for pair in pairs {
                return Ok(build_matcher(pair));
            }
        }
        Err(err) => {
            return Err(err.to_string());
        }
    }
    Err("Unknown error".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn haystack(needle: &[u8]) -> (Vec<u8>, usize) {
        let needle_len = needle.len();
        let mut bytes: Vec<u8> = vec![0; 4096];
        let index = ((rand::random::<usize>() % bytes.len()) / needle_len) * needle_len;
        for (idx, b) in needle.iter().enumerate() {
            bytes[index + idx] = *b;
        }
        (bytes, index)
    }

    fn perform_search(val_bytes: &[u8], query: &str) -> bool {
        let (bytes, needle) = haystack(val_bytes);
        let (tx, rx) = mpsc::channel();

        let fun = parse(query).unwrap();
        fun.scan_page(&tx, &bytes, 0);
        let actual: Vec<SearchResult> = rx.try_iter().collect();

        let mut found = false;
        if actual.len() >= 1 && actual.iter().any(|res| res.address == needle) {
            found = true;
        }

        found
    }

    #[test]
    fn finds_int() {
        let needle: i32 = 625;
        assert!(perform_search(&needle.to_ne_bytes(), "625"));
    }

    #[test]
    fn finds_float() {
        let needle: f32 = 625.10001;
        assert!(perform_search(&needle.to_ne_bytes(), "=625.1"));
    }

    #[test]
    fn finds_gt() {
        let needle: i64 = 625;
        assert!(perform_search(&needle.to_ne_bytes(), " > 500 "));
    }

    #[test]
    fn no_find() {
        let needle: i32 = 625;
        assert!(!perform_search(&needle.to_ne_bytes(), "9999 "));
    }

    #[test]
    fn no_find_gt() {
        let needle: i32 = 625;
        assert!(!perform_search(&needle.to_ne_bytes(), " > 625"));
    }
}
