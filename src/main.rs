mod map;
mod player;
mod render;
mod vector;

use macroquad::prelude::*;
use map::Map;
use player::Player;
use std::fs::File;

use crate::player::RayCastResult;

fn open_map() -> Map {
    let map = File::open("./maps/testMap.map").unwrap();
    match Map::parse(map, 64.0) {
        Ok(map) => map,
        Err(e) => panic!("Error parsing map: {:?}", e),
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "RayCaster".to_owned(),
        window_width: 800,
        window_height: 800,
        ..Default::default()
    }
}

fn move_player(player: &mut Player, map: &Map) {
    if is_key_down(KeyCode::Q) {
        player.rotate(-0.1);
    }
    if is_key_down(KeyCode::D) {
        player.rotate(0.1);
    }
    if is_key_down(KeyCode::Z) {
        player.move_forward(100.0 * get_frame_time(), &map);
    }
    if is_key_down(KeyCode::S) {
        player.move_backward(100.0 * get_frame_time(), &map);
    }
}

fn draw_rays(map: &Map, player: &Player) {
    let mut angle = -std::f32::consts::PI / 4.0;
    let increment = std::f32::consts::PI / 2.0 / screen_width();

    for i in 0..screen_width() as i32 {
        let ray = player.raycast(&map, angle);
        angle += increment;
        match ray {
            RayCastResult::NoHit => {}
            RayCastResult::Hit(distance, _, side) => {
                let height = screen_height() / distance;
                let color = if side {
                    Color::from_rgba(255, 0, 0, 255)
                } else {
                    Color::from_rgba(190, 0, 0, 255)
                };
                draw_line(
                    i as f32,
                    screen_height() / 2.0 - height / 2.0,
                    i as f32,
                    screen_height() / 2.0 + height / 2.0,
                    1.0,
                    color,
                );
            }
        }
    }
}

fn debug_infos() {
    let fps = get_fps();
    let render_time = get_frame_time();
    let render_time_ms = render_time * 1000.0;
    let fps_text = format!("FPS: {:.2}", fps);
    let render_time_text = format!("Render time: {:.2}ms", render_time_ms);
    draw_text(&fps_text, 10.0, 20.0, 20.0, GREEN);
    draw_text(&render_time_text, 10.0, 40.0, 20.0, GREEN);
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut map = open_map();
    let mut player = Player::new(100.0, 100.0, 0.0);
    loop {
        if is_key_pressed(KeyCode::Escape) {
            return;
        }

        move_player(&mut player, &map);
        let width = screen_width();
        let height = screen_height();
        map.auto_tile_size(width, height);
        clear_background(BLACK);
        // map.render();
        // player.render();

        draw_rays(&map, &player);

        debug_infos();

        next_frame().await
    }
}
