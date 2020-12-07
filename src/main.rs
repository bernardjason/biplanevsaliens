use sdl_main::sdl_main;
mod macros;
mod shader;
mod common;
mod camera;
mod sdl_main;
mod draw_text;
mod landscape;
mod player_shape;
mod model;
mod mesh;
mod bullet;
mod enemy_ship;
mod sound;
mod start_screen;
mod explosion;
fn main() {
    sdl_main().unwrap();
}


