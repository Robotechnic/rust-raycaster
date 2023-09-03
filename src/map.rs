use regex::Regex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::ops::{Index, IndexMut};

use crate::render::Render;
use macroquad::prelude::{draw_rectangle, BLACK, RED, WHITE};


pub struct Map {
    name: String,
    width: usize,
    height: usize,
    tiles: Vec<u8>,
    x: f32,
    y: f32,
    tile_size: f32,
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
            }
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

fn parse_size(size: &str, line: &u32) -> Result<(usize, usize), ParseError> {
    let size_regex = Regex::new(r"(\d+)x(\d+)").unwrap();

    let captures =
        size_regex
            .captures(size)
            .ok_or(ParseError::InvalidFormat(ParseErrorDetails {
                line: *line,
                message: "Invalid size format".to_string(),
            }))?;
    let width = captures.get(1).unwrap().as_str().parse::<usize>().unwrap();
    let height = captures.get(2).unwrap().as_str().parse::<usize>().unwrap();
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
    width: &usize,
    height: &usize,
) -> Result<Vec<u8>, ParseError> {
    let mut tiles = Vec::new();
    let expected_len = (width * height) as usize;
    tiles.reserve(expected_len);
    while let Some(line) = lines.next() {
        *line_number += 1;
        let line = line.map_err(ParseError::FileError)?;
        let mut line = line.split_whitespace();
        while let Some(tile) = line.next() {
            let tile = tile.parse::<u8>().map_err(|_| {
                ParseError::InvalidFormat(ParseErrorDetails {
                    line: *line_number,
                    message: "Invalid tile format".to_string(),
                })
            })?;
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
            message: format!(
                "Not enough tiles, expected {} but got {}",
                expected_len,
                tiles.len()
            )
            .to_owned(),
        }));
    }
    return Ok(tiles);
}

impl Map {
    pub fn new(name: String, width: usize, height: usize, tile_size: f32, tiles: Vec<u8>) -> Map {
        assert!(tiles.len() == (width * height) as usize);
        Map {
            name: name,
            width: width,
            height: height,
            tiles: tiles,
            x: 0.0,
            y: 0.0,
            tile_size: tile_size,
        }
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }

    /// Automatically sets the tile size so that the map
    /// fits into the given window size
    pub fn auto_tile_size(&mut self, window_width: f32, window_height: f32) {
        let width = window_width as usize / self.width;
        let height = window_height as usize / self.height;
        self.tile_size = width.min(height) as f32;
    }

    fn get_tile(&self, x: usize, y: usize) -> &u8 {
        &self.tiles[y * self.width + x]
    }

    fn get_tile_mut(&mut self, x: usize, y: usize) -> &mut u8 {
        &mut self.tiles[y * self.width + x]
    }

    /// Parses the given map file and returns a map
    /// To see how the map file is structured, see the
    /// README.md file
    pub fn parse(map: File, tile_size: f32) -> Result<Map, ParseError> {
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
        Ok(Map::new(name.to_string(), width, height, tile_size, tiles))
    }

    pub fn to_map_coordinates(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let x = x - self.x;
        let y = y - self.y;
        let x = (x / self.tile_size) as usize;
        let y = (y / self.tile_size) as usize;
        if x >= self.width || y >= self.height {
            return None;
        }
        Some((x, y))
    }
}

impl Index<(usize, usize)> for Map {
    type Output = u8;

    fn index(&self, index: (usize, usize)) -> &u8 {
        self.get_tile(index.0, index.1)
    }
}

impl IndexMut<(usize, usize)> for Map {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut u8 {
        self.get_tile_mut(index.0, index.1)
    }
}

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "name: {}, width: {}, height: {}\n",
            self.name, self.width, self.height
        )?;
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{} ", self.get_tile(x, y))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Render for Map {
    /// Renders the map in a top-down view
    fn render(&self) {
        for tile_y in 0..self.height {
            for tile_x in 0..self.width {
                let tile = self.get_tile(tile_x, tile_y);
                let color = match tile {
                    0 => WHITE,
                    1 => BLACK,
                    _ => RED,
                };
                draw_rectangle(
                    self.x + tile_x as f32 * self.tile_size,
                    self.y + tile_y as f32 * self.tile_size,
                    self.tile_size,
                    self.tile_size,
                    color,
                );
            }
        }
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
        let map = Map::parse(map, 20.0);
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
