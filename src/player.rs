use crate::map::Map;
use crate::render::Render;
use crate::vector::Vector;
use macroquad::prelude::{draw_circle, draw_line, RED, YELLOW};

const PLAYER_RADIUS: f32 = 10.0;

pub struct Player {
    pub pos: Vector<f32>,
    angle: f32,
}

pub enum RayCastResult {
    Hit(f32, Vector<usize>, bool),
    NoHit,
}

impl Player {
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        Self {
            pos: Vector::new(x, y),
            angle,
        }
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
        let x = self.pos.x + self.angle.cos() * distance;
        let y = self.pos.y + self.angle.sin() * distance;

        self.move_collision_check(x, y, map, false);
    }

    pub fn move_backward(&mut self, distance: f32, map: &Map) {
        let x = self.pos.x - self.angle.cos() * distance;
        let y = self.pos.y - self.angle.sin() * distance;

        self.move_collision_check(x, y, map, true);
    }

    fn move_collision_check(&mut self, x: f32, y: f32, map: &Map, rev: bool) {
        let map_pos = map.to_map_coordinates(&self.pos);
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

        let new_map_pos = map.to_map_coordinates(&Vector::new(x_rad, y_rad));
        if map_pos.is_none() || new_map_pos.is_none() {
            return;
        }

        let map_pos = map_pos.unwrap();
        let new_map_pos = new_map_pos.unwrap();

        if map[(new_map_pos.x, map_pos.y)] != 1 {
            self.pos.x = x;
        }
        if map[(map_pos.x, new_map_pos.y)] != 1 {
            self.pos.y = y;
        }
    }

    /// cast a single ray from (x, y) and return the distance to the nearest wall
    /// Return [RayCastResult::Hit] if the ray hit a wall
    /// Return [RayCastResult::NoHit] if the ray didn't hit a wall
    pub fn raycast(&self, map: &Map, offset: f32) -> RayCastResult {
        if !map.in_map(&self.pos) {
            return RayCastResult::NoHit;
        }

        let pos = self.pos / map.get_tile_size();
        let angle = self.angle + offset;
        let direction = Vector::new(angle.cos(), angle.sin());

        let step_size = Vector::new(
            (1.0 + (direction.y / direction.x).powi(2)).sqrt(),
            (1.0 + (direction.x / direction.y).powi(2)).sqrt(),
        );

        let mut map_pos: Vector<i32> = pos.to_i32();
        let ray_len_x = if direction.x < 0.0 {
            (pos.x - map_pos.x as f32) * step_size.x
        } else {
            ((map_pos.x as f32 + 1.0) - pos.x) * step_size.x
        };

        let ray_len_y = if direction.y < 0.0 {
            (pos.y - map_pos.y as f32) * step_size.y
        } else {
            ((map_pos.y as f32 + 1.0) - pos.y) * step_size.y
        };

        let mut ray_len = Vector::new(ray_len_x, ray_len_y);

        let step = Vector::new(
            if direction.x < 0.0 { -1 } else { 1 },
            if direction.y < 0.0 { -1 } else { 1 },
        );

        let mut hit = false;
        let mut side = false;
        let mut out = false;
        let mut distance = 0.0;

        while !hit && !out {
            if ray_len.x < ray_len.y {
                map_pos.x += step.x;
                ray_len.x += step_size.x;
                side = false;
                distance = ray_len.x;
            } else {
                map_pos.y += step.y;
                ray_len.y += step_size.y;
                side = true;
                distance = ray_len.y;
            }

            if map_pos.x < 0
                || map_pos.x >= map.get_width() as i32
                || map_pos.y < 0
                || map_pos.y >= map.get_height() as i32
            {
                out = true;
            } else if map[(map_pos.x as usize, map_pos.y as usize)] == 1 {
                hit = true;
            }
        }

        if hit {
            let hit_pos = (pos + (direction * distance)) * map.get_tile_size();
            RayCastResult::Hit(
                distance * map.get_tile_size(),
                Vector::new(map_pos.x as usize, map_pos.y as usize),
                side,
            )
        } else {
            RayCastResult::NoHit
        }
    }
}

impl Render for Player {
    fn render(&self) {
        draw_circle(self.pos.x, self.pos.y, 10.0, RED);
        draw_line(
            self.pos.x,
            self.pos.y,
            (self.angle.cos() * 30.0) + self.pos.x,
            (self.angle.sin() * 30.0) + self.pos.y,
            1.0,
            YELLOW,
        );
    }
}
