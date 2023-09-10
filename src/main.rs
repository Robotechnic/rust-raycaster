mod map;
mod player;
mod render;
mod vector;

use macroquad::prelude::*;
use map::Map;
use player::Player;
use render::Render;
use vector::Vector;
use std::fs::File;

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
        map.render();
        player.render();

        let ray = player.raycast(&map, 0.0);
        match ray {
            player::RayCastResult::NoHit => {}
            player::RayCastResult::Hit(distance, Vector{x: i, y : j}, side) => {
                draw_rectangle(
                    i as f32 * map.get_tile_size(),
                    j as f32 * map.get_tile_size(),
                    map.get_tile_size(),
                    map.get_tile_size(),
                    if side {RED} else {BLUE},
                );
                let distance_text = format!("Distance: {:.2}", distance);
                draw_text(&distance_text, 10.0, 60.0, 20.0, GREEN);
            } 
        }

        debug_infos();

        next_frame().await
    }
}
