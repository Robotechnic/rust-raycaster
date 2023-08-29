use regex::Regex;
use std::collections::HashMap;
use std::fmt::{Debug};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::ops::{Index, IndexMut};

pub struct Map {
    name: String,
    width: u32,
    height: u32,
    tiles: Vec<u8>,
}

pub struct ParseErrorDetails {
    line: u32,
    message: String,
}

pub enum ParseError {
    FileError(io::Error),
    InvalidFormat(ParseErrorDetails),
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::FileError(e) => write!(f, "File error: {}", e),
            ParseError::InvalidFormat(e) => {
                write!(f, "Invalid format: {}\n", e.message)?;
                write!(f, " at line {}", e.line)
            },
        }
    }
}

/// Check if the given line is a valid field
fn parse_field(field: String, line_number: &u32) -> Result<(String, String), ParseError> {
    if field.len() < 3 {
        return Err(ParseError::InvalidFormat(ParseErrorDetails {
            line: *line_number,
            message: "Invalid field format".to_string(),
        }));
    }
    let mut field = field.splitn(2, '=');
    let key = field.next().unwrap();
    let value = field.next();
    if value.is_none() {
        return Err(ParseError::InvalidFormat(ParseErrorDetails {
            line: *line_number,
            message: "Invalid field format".to_string(),
        }));
    }
    let value = value.unwrap();
    if key.len() == 0 || value.len() == 0 {
        return Err(ParseError::InvalidFormat(ParseErrorDetails {
            line: *line_number,
            message: "Invalid field format".to_string(),
        }));
    }
    return Ok((key.trim().to_owned(), value.trim().to_owned()));
}

/// loads all the fields from the given iterator of lines
/// until the "---" separator is reached
/// Returns a hashmap of the fields
fn load_fields(
    lines: &mut Lines<BufReader<File>>,
    line_number: &mut u32,
) -> Result<HashMap<String, String>, ParseError> {
    let mut fields = HashMap::new();
    let mut map_reached = false;
    while let Some(line) = lines.next() {
        *line_number += 1;
        let line = line.map_err(ParseError::FileError)?;
        println!("Field: {}", line);
        if line == "---" {
            map_reached = true;
            break;
        }
        let field = parse_field(line, line_number)?;
        fields.insert(field.0, field.1);
    }
    if !map_reached {
        return Err(ParseError::InvalidFormat(ParseErrorDetails {
            line: *line_number,
            message: "Map separator not found".to_string(),
        }));
    }
    return Ok(fields);
}

fn parse_size(size: &str, line: &u32) -> Result<(u32, u32), ParseError> {
    let size_regex = Regex::new(r"(\d+)x(\d+)").unwrap();

    let captures =
        size_regex
            .captures(size)
            .ok_or(ParseError::InvalidFormat(ParseErrorDetails {
                line: *line,
                message: "Invalid size format".to_string(),
            }))?;
    let width = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
    let height = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
    return Ok((width, height));
}

/// Parses the tiles from the given iterator of lines
/// until the end of the file is reached
/// Returns a vector of tiles if there are enough tiles
/// or an error if there are too many or too few tiles or
/// if the tiles are invalid
fn parse_tiles(
    lines: &mut Lines<BufReader<File>>,
    line_number: &mut u32,
    width: &u32,
    height: &u32,
) -> Result<Vec<u8>, ParseError> {
    let mut tiles = Vec::new();
    let expected_len = (width * height) as usize;
    tiles.reserve(expected_len);
    while let Some(line) = lines.next() {
        *line_number += 1;
        let line = line.map_err(ParseError::FileError)?;
        let mut line = line.split_whitespace();
        while let Some(tile) = line.next() {
            let tile = tile.parse::<u8>().map_err(|_| ParseError::InvalidFormat(ParseErrorDetails {
                line: *line_number,
                message: "Invalid tile format".to_string(),
            }))?;
            if tiles.len() >= expected_len {
                return Err(ParseError::InvalidFormat(ParseErrorDetails {
                    line: *line_number,
                    message: "Too many tiles".to_string(),
                }));
            }
            tiles.push(tile);
        }
    }
    if tiles.len() < expected_len {
        return Err(ParseError::InvalidFormat(ParseErrorDetails {
            line: *line_number,
            message: format!("Not enough tiles, expected {} but got {}", expected_len, tiles.len()).to_owned(),
        }));
    }
    return Ok(tiles);
}

impl Map {
    pub fn new(name: String, width: u32, height: u32, tiles: Vec<u8>) -> Map {
        assert!(tiles.len() == (width * height) as usize);
        Map {
            name: name,
            width: width,
            height: height,
            tiles: tiles,
        }
    }

    fn get_tile(&self, x: u32, y: u32) -> &u8 {
        &self.tiles[(y * self.width + x) as usize]
    }

    fn get_tile_mut(&mut self, x: u32, y: u32) -> &mut u8 {
        &mut self.tiles[(y * self.width + x) as usize]
    }

    pub fn parse(map: File) -> Result<Map, ParseError> {
        let reader = BufReader::new(map);
        let mut lines = reader.lines();
        let mut line = 0;
        let fields = load_fields(&mut lines, &mut line)?;

        let name = fields
            .get("name")
            .ok_or(ParseError::InvalidFormat(ParseErrorDetails {
                line: line,
                message: "Missing name field".to_string(),
            }))?;

        let size = fields
            .get("size")
            .ok_or(ParseError::InvalidFormat(ParseErrorDetails {
                line: line,
                message: "Missing size field".to_string(),
            }))?;

        let (width, height) = parse_size(size, &line)?;
        let tiles = parse_tiles(&mut lines, &mut line, &width, &height)?;
        Ok(Map::new(name.to_string(), width, height, tiles))
    }
}

impl Index<(u32, u32)> for Map {
    type Output = u8;

    fn index(&self, index: (u32, u32)) -> &u8 {
        self.get_tile(index.0, index.1)
    }
}

impl IndexMut<(u32, u32)> for Map {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut u8 {
        self.get_tile_mut(index.0, index.1)
    }
}

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}, width: {}, height: {}\n", self.name, self.width, self.height)?;
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{} ", self.get_tile(x, y))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod parser_tests {
    use super::{parse_field, parse_size};
    #[test]
    fn test_parse_fiels() {
        let field = "name = test".to_string();
        let result = parse_field(field, &1);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, "name");
        assert_eq!(result.1, "test");
    }

    #[test]
    fn test_parse_fiels_non_ascii() {
        let field = "name = test äöü".to_string();
        let result = parse_field(field, &1);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, "name");
        assert_eq!(result.1, "test äöü");
    }

    #[test]
    fn test_parse_fiels_no_value() {
        let field = "name =".to_string();
        let result = parse_field(field, &1);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fiels_no_key() {
        let field = "= test".to_string();
        let result = parse_field(field, &1);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fiels_no_key_no_value() {
        let field = "=".to_string();
        let result = parse_field(field, &1);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fiels_no_key_no_value_no_equals() {
        let field = "".to_string();
        let result = parse_field(field, &1);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_size() {
        let size = "10x10";
        let result = parse_size(size, &1);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 10);
        assert_eq!(result.1, 10);
    }

    #[test]
    fn test_parse_size_invalid() {
        let size = "10x10x10";
        let result = parse_size(size, &1);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_size_invalid2() {
        let size = "10x";
        let result = parse_size(size, &1);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_size_invalid3() {
        let size = "x10";
        let result = parse_size(size, &1);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod simple_load_test {
    use super::*;

    /// Just a simple test to see if the map can be loaded
    /// and printed
    #[test]
    fn load() -> Result<(), Box<dyn std::error::Error>> {
        let map = std::fs::File::open("maps/testMap.map")?;
        let map = Map::parse(map);
        match map {
            Ok(map) => {
                println!("Map is ok:\n{:?}", map);
            }
            Err(e) => {
                println!("An error occurred: {:?}", e);
            }
        }
        Ok(())
    }
}