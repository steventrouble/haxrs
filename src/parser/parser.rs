use std::{mem::size_of, sync::mpsc::Sender};

use pest::Parser;

use crate::windex::{scanner::SearchResult, DataTypeEnum};

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

/// A node in the AST of the search query.
/// Generally, a Node represents a Matcher that can match values in memory.
pub enum Node {
    Constant(ConstantMatcher),
}

impl Node {
    /// Scans an entire page of memory for things that match.
    pub fn scan_page(&self, tx: &Sender<SearchResult>, mem: &[u8], addr_start: usize) {
        match self {
            Node::Constant(info) => scan_page(tx, mem, addr_start, info),
        }
    }

    /// Compares a single value against this matcher.
    pub fn test_single(&self, mem: &[u8], data_type: DataTypeEnum) -> bool {
        match self {
            Node::Constant(info) => {
                match data_type {
                    DataTypeEnum::FourBytes => {
                        if let Some(value) = info.as_int {
                            if let Ok(v) = bytemuck::try_from_bytes::<i32>(mem) {
                                if *v == value as i32 {
                                    return true;
                                }
                            }
                        }
                    }
                    DataTypeEnum::EightBytes => {
                        if let Some(value) = info.as_int {
                            if let Ok(v) = bytemuck::try_from_bytes::<i64>(mem) {
                                if *v == value {
                                    return true;
                                }
                            }
                        }
                    }
                    DataTypeEnum::Float => {
                        if let Some((value, decimal_digits)) = info.as_float {
                            let lowest_digit = (10.0 as f32).powf(-(decimal_digits as f32));
                            let value = value as f32;
                            let lower_bound = value - lowest_digit;
                            let upper_bound = value + lowest_digit;
                            if let Ok(v) = bytemuck::try_from_bytes::<f32>(mem) {
                                if *v > lower_bound && *v < upper_bound {
                                    return true;
                                }
                            }
                        }
                    }
                    DataTypeEnum::Double => {
                        if let Some((value, decimal_digits)) = info.as_float {
                            let lowest_digit = (10.0 as f64).powf(-(decimal_digits as f64));
                            let lower_bound = value - lowest_digit;
                            let upper_bound = value + lowest_digit;
                            if let Ok(v) = bytemuck::try_from_bytes::<f64>(mem) {
                                if *v > lower_bound && *v < upper_bound {
                                    return true;
                                }
                            }
                        }
                    }
                }
                false
            }
        }
    }
}

/// Scans an entire page of memory for things that match.
fn scan_page(tx: &Sender<SearchResult>, mem: &[u8], addr_start: usize, values: &ConstantMatcher) {
    if let Some(value) = values.as_int {
        for (i, &v) in bytemuck::try_cast_slice::<u8, i64>(mem)
            .unwrap_or(&[])
            .iter()
            .enumerate()
        {
            if v == value {
                let result = SearchResult {
                    address: i * size_of::<i64>() + addr_start,
                    data_type: DataTypeEnum::EightBytes,
                    value: v.to_ne_bytes().to_vec(),
                };
                tx.send(result).unwrap();
            }
        }
        let value = value as i32;
        for (i, v) in bytemuck::try_cast_slice::<u8, i32>(mem)
            .unwrap_or(&[])
            .iter()
            .enumerate()
        {
            if *v == value {
                let result = SearchResult {
                    address: i * size_of::<i32>() + addr_start,
                    data_type: DataTypeEnum::FourBytes,
                    value: v.to_ne_bytes().to_vec(),
                };
                tx.send(result).unwrap();
            }
        }
    }

    if let Some((value, decimal_digits)) = values.as_float {
        let lowest_digit = (10.0 as f64).powf(-(decimal_digits as f64));
        let lower_bound = value - lowest_digit * 0.999;
        let upper_bound = value + lowest_digit * 0.999;
        for (i, v) in bytemuck::try_cast_slice::<u8, f64>(mem)
            .unwrap_or(&[])
            .iter()
            .enumerate()
        {
            if *v > lower_bound && *v < upper_bound {
                let result = SearchResult {
                    address: i * size_of::<f64>() + addr_start,
                    data_type: DataTypeEnum::Double,
                    value: v.to_ne_bytes().to_vec(),
                };
                tx.send(result).unwrap();
            }
        }
        let value = value as f32;
        let lowest_digit = lowest_digit as f32;
        let lower_bound = value - lowest_digit;
        let upper_bound = value + lowest_digit;
        for (i, v) in bytemuck::try_cast_slice::<u8, f32>(mem)
            .unwrap_or(&[])
            .iter()
            .enumerate()
        {
            if *v > lower_bound && *v < upper_bound {
                let result = SearchResult {
                    address: i * size_of::<f32>() + addr_start,
                    data_type: DataTypeEnum::Float,
                    value: v.to_ne_bytes().to_vec(),
                };
                tx.send(result).unwrap();
            }
        }
    }
}

pub enum ParseError {
    Unknown,
}

impl<T> From<pest::error::Error<T>> for ParseError {
    fn from(_: pest::error::Error<T>) -> Self {
        ParseError::Unknown
    }
}

impl From<ParseError> for String {
    fn from(_: ParseError) -> String {
        "".to_string()
    }
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
        _unknown => panic!("Unknown err, call the dev."),
    }
}

/// Parses a search query and returns a matcher representing that query.
pub fn parse(input: &str) -> Result<Node, ParseError> {
    let pairs = SearchParser::parse(Rule::SearchQuery, input)?;
    for pair in pairs {
        if let Rule::Num = pair.as_rule() {
            return Ok(build_matcher(pair));
        }
    }
    Err(ParseError::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn haystack(needle: &[u8]) -> (Vec<u8>, usize) {
        let needle_len = needle.len();
        let mut bytes: Vec<u8> = vec![1; 4096];
        let index = ((rand::random::<usize>() % bytes.len()) / needle_len) * needle_len;
        for (idx, b) in needle.iter().enumerate() {
            bytes[index + idx] = *b;
        }
        (bytes, index)
    }

    #[test]
    fn finds_int() -> Result<(), String> {
        let val: i32 = 625;
        let (bytes, needle) = haystack(&val.to_ne_bytes());
        let (tx, rx) = mpsc::channel();

        let fun = parse("625")?;
        fun.scan_page(&tx, &bytes, 0);
        let found: Vec<SearchResult> = rx.try_iter().collect();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].address, needle);
        Ok(())
    }

    #[test]
    fn finds_float() -> Result<(), String> {
        let val: f32 = 625.10001; // should find approx match
        let (bytes, needle) = haystack(&val.to_ne_bytes());
        let (tx, rx) = mpsc::channel();

        let fun = parse("625.1")?;
        fun.scan_page(&tx, &bytes, 0);
        let found: Vec<SearchResult> = rx.try_iter().collect();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].address, needle);
        Ok(())
    }

    #[test]
    fn no_find() -> Result<(), String> {
        let val: i32 = 625;
        let (bytes, _) = haystack(&val.to_ne_bytes());
        let (tx, rx) = mpsc::channel();

        let fun = parse("9999")?;
        fun.scan_page(&tx, &bytes, 0);
        let found: Vec<SearchResult> = rx.try_iter().collect();
        assert_eq!(found.len(), 0);
        Ok(())
    }
}
