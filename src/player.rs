use crate::map::Map;
use crate::render::Render;
use macroquad::prelude::{draw_circle, draw_line, RED, YELLOW};

const PLAYER_RADIUS: f32 = 10.0;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        Self { x, y, angle }
    }

    pub fn rotate(&mut self, angle: f32) {
        self.angle += angle;

        if self.angle > std::f32::consts::PI {
            self.angle -= std::f32::consts::TAU;
        }

        if self.angle < -std::f32::consts::PI {
            self.angle += std::f32::consts::TAU;
        }
    }

    pub fn move_forward(&mut self, distance: f32, map: &Map) {
        let x = self.x + self.angle.cos() * distance;
        let y = self.y + self.angle.sin() * distance;

        self.move_collision_check(x, y, map, false);
    }

    pub fn move_backward(&mut self, distance: f32, map: &Map) {
        let x = self.x - self.angle.cos() * distance;
        let y = self.y - self.angle.sin() * distance;

        self.move_collision_check(x, y, map, true);
    }

    fn move_collision_check(&mut self, x: f32, y: f32, map: &Map, rev: bool) {
        let map_pos = map.to_map_coordinates(self.x, self.y);
        let x_rad = x
            + (if rev ^ (self.angle.abs() < std::f32::consts::PI / 2.0) {
                PLAYER_RADIUS
            } else {
                -PLAYER_RADIUS
            });
        let y_rad = y
            + (if rev ^ (self.angle > 0.0) {
                PLAYER_RADIUS
            } else {
                -PLAYER_RADIUS
            });
        
        let new_map_pos = map.to_map_coordinates(x_rad, y_rad);
        if map_pos.is_none() || new_map_pos.is_none() {
            return;
        }

        let map_pos = map_pos.unwrap();
        let new_map_pos = new_map_pos.unwrap();

        if map[(new_map_pos.0, map_pos.1)] != 1 {
            self.x = x;
        }
        if map[(map_pos.0, new_map_pos.1)] != 1 {
            self.y = y;
        }
    }
}

impl Render for Player {
    fn render(&self) {
        draw_circle(self.x, self.y, 10.0, RED);
        draw_line(
            self.x,
            self.y,
            (self.angle.cos() * 30.0) + self.x,
            (self.angle.sin() * 30.0) + self.y,
            1.0,
            YELLOW,
        );
    }
}
