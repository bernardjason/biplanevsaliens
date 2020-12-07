extern crate sdl2;

use std::collections::HashMap;

use cgmath::{Deg, Matrix4, Point3, Vector3, vec3, perspective};
use cgmath::prelude::*;
use noise::{Value};
use rand::Rng;
use sdl2::event::Event;
use sdl2::gfx::framerate::FPSManager;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureCreator;

use crate::bullet::Bullet;
use crate::camera::Camera;
use crate::draw_text::DrawText;
use crate::enemy_ship::EnemyShip;
use crate::explosion::Explosion;
use crate::landscape::{ACTUAL_WIDTH, CELL, LAND_X, LAND_Z, Landscape, SQUARES};
use crate::player_shape::{PlayerShape};
use crate::sound::{ENGINE, HIT_WALL, load_sound, play};
use crate::start_screen;

use self::sdl2::EventPump;
use self::sdl2::pixels::{Color };
use self::sdl2::rect::Rect;

pub const SCR_WIDTH: u32 = 1600;
pub const SCR_HEIGHT: u32 = 800;
pub const FAR: f32 = 200.0;
pub const MAX_LANDED : i32 = 30;

pub fn sdl_main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut frame_control = FPSManager::new();

    let mut current_screen = 0;

    frame_control.set_framerate(60)?;


    let window = video_subsystem.window("bi-plane vs aliens", SCR_WIDTH, SCR_HEIGHT)
        .opengl()
        .position_centered()
        .build().unwrap();

    let mut canvas = window.into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build()
        .unwrap();

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font_path = "resources/OpenSans-Light.ttf";
    let mut font = ttf_context.load_font(font_path, 32)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);
    let mut message_font = ttf_context.load_font(font_path, 32)?;
    message_font.set_style(sdl2::ttf::FontStyle::NORMAL);


    let texture_creator: TextureCreator<_> = canvas.texture_creator();

    load_sound(&sdl_context);

    play(ENGINE);
    // initialization
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

    let mut camera = Camera {
        Position: Point3::new(0.0, 2.0, 8.0),
        ..Camera::default()
    };

    let mut player = PlayerShape::new();
    let mut bullets = Bullet::new();
    let mut explosions = Explosion::new();
    let mut enemy_ships = EnemyShip::new();

    let value_noise = Value::new();
    let mut landscape_vec = vec![vec![]];
    for z in 0..LAND_Z {
        landscape_vec.push(vec![]);
        for x in 0..LAND_X {
            let left = if x > 0 {
                let l: &Landscape = landscape_vec[z].last().unwrap();
                Some(l.height)
            } else {
                None
            };
            let below = if z > 0 {
                let l: &Landscape = &landscape_vec[z - 1][x];
                Some(l.height)
            } else {
                None
            };
            landscape_vec[z].push(Landscape::new(&value_noise, left, below, x as f32 * SQUARES as f32 * CELL as f32 * 2.0, z as f32 * SQUARES as f32 * CELL as f32 * 2.0));
        }
    }

    let text_draw = DrawText::new();

    let mut delay = 0;
    let mut event_pump = sdl_context.event_pump()?;
    let (mut left, mut right, mut faster, mut slower, mut fire, mut pitch_down, mut pitch_up) =
        (false, false, false, false, false, false, false);
    let mut fire_every = 0;

    let mut start_screen = start_screen::StartScreen::new(&texture_creator, &font);

    'gui_loop: loop {
        let delta_time = 0.5 / delay as f32;

        let quit = process_keyboard(&mut event_pump, &mut left, &mut right, &mut faster, &mut slower, &mut fire, &mut pitch_down, &mut pitch_up);

        if quit {
            if current_screen == 1 {
                current_screen = 2;
            } else {
                break 'gui_loop;
            }
        }


        match current_screen {
            0 => {
                canvas.window().gl_set_context_to_current().unwrap();
                canvas.set_draw_color(Color::RGB(0, 0, 0));
                canvas.clear();
                start_screen.update();
                start_screen = start_screen.draw_on_canvas(&mut canvas, &message_font, &texture_creator);
                if start_screen.bernie_x > SCR_WIDTH as i32 || fire == true {
                    current_screen = 1;
                    fire = false;
                }
                canvas.window().gl_swap_window();
            }
            2 => {
                // cannot get it to start using SDL2 again once I start using opengl shader.
                canvas.window().gl_set_context_to_current().unwrap();

                unsafe {
                    gl::Enable(gl::DEPTH_TEST);
                    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);


                    let (view, projection) = projection_view_camera(&mut camera, &mut player);

                    player.update_player();

                    player.process_keyboard_roll(-0.125, delta_time);
                    player.process_keyboard_pitch(-2.0, delta_time);

                    text_draw.draw_text(format!("New game....").as_ref(), 50.0, SCR_HEIGHT as f32 / 2.0, vec3(1.0, 1.0, 0.0));
                    text_draw.draw_text(format!("lives {} score {} aliens landed={}", player.lives, player.score, enemy_ships.landed).as_ref(), 50.0, SCR_HEIGHT as f32 /2.0 - 64.0, vec3(0.0, 1.0, 0.8));

                    player.render(&view, &projection);
                }
                canvas.window().gl_swap_window();
                if fire == true {
                    current_screen = 1;
                    fire = false;
                    player.game_reset_player();
                    enemy_ships.reset();
                }
            }
            1 => {
                if enemy_ships.landed >= MAX_LANDED  || player.lives <= 0 {
                    canvas.set_viewport(Rect::new(0, 0, SCR_WIDTH, SCR_HEIGHT));
                    current_screen = 2;
                    fire = false;
                    continue 'gui_loop;
                }
                if player.player_ok() {
                    if left { player.process_keyboard_roll(-1.0, delta_time); }
                    if right { player.process_keyboard_roll(1.0, delta_time); }
                    if pitch_down { player.process_keyboard_pitch(1.0, delta_time); }
                    if pitch_up { player.process_keyboard_pitch(-1.0, delta_time); }
                    let original_position = player.position.clone();
                    let mut check = player.position.clone();

                    if faster {
                        player.faster();
                    }
                    if slower {
                        player.slower();
                    }
                    if player.speed > 0.0 {
                        let dir = player.forward(1.0, delta_time);
                        check -= dir * 0.25;
                        let (height_below, check_below) = height_above_land(&check, &landscape_vec);
                        if height_below < 0.45 {
                            player.position.x = original_position.x;
                            player.position.y = original_position.y;
                            player.position.z = original_position.z;
                            faster = false;
                            if !player.hit(check_below) {
                                player_been_hit(&mut player, &mut explosions);
                            }
                        }
                    }
                    if pitch_down == false && pitch_up == false {
                        player.update_camera();
                    } else {
                        player.queue_camera_for_later();
                    }
                }

                fire_every = fire_every + 1;
                if fire && fire_every > 0 {
                    let (pos_left, pos_right, dir) = player.get_bullet_direction();
                    bullets.new_instance(dir, pos_left);
                    bullets.new_instance(dir, player.position);
                    bullets.new_instance(dir, pos_right);
                    fire_every = -30;
                }

                explosions.update_explosion(delta_time);
                bullets.update_bullets(delta_time);
                enemy_ships.update_enemy(delta_time);


                //let possible_positions = possible_locations_for_enemy(&landscape_vec);

                if enemy_ships.enemy_update() <= 2 {
                    let mut possible_positions = possible_locations_for_enemy(&landscape_vec, player.position);

                    while possible_positions.len() > 0 && enemy_ships.new_instance(possible_positions.pop().unwrap()) <= 2 {

                    }


                }


                canvas.window().gl_set_context_to_current().unwrap();

                let (view, projection) = projection_view_camera(&mut camera, &mut player);

                let mut dirs: Vec<Vector3<f32>> = Vec::new();

                for a in -10..10 {
                    dirs.push(player.get_direction_adjust_angle(a as f32 * 7.0))
                }
                for v in landscape_vec.iter_mut() {
                    for l in v.iter_mut() {
                        l.update();
                    }
                }
                unsafe {
                    gl::Enable(gl::DEPTH_TEST);
                    gl::ClearColor(0.7, 0.9, 1.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                    render_landscapes(player.position, dirs, &landscape_vec, &view, &projection);

                    player.update_player();

                    player.render(&view, &projection);

                    bullets.render(&view, &projection);

                    enemy_ships.render(&view, &projection);

                    explosions.render(&view, &projection);
                    text_draw.draw_text(format!("speed {:.2} fuel {} lives {} score {} aliens landed={}", player.speed, (player.fuel / 1000.0) as i32, player.lives, player.score, enemy_ships.landed).as_ref(), 8.0, SCR_HEIGHT as f32 - 40.0, vec3(0.0, 0.0, 0.8));

                }

                for e in enemy_ships.instances.iter_mut() {
                    if !e.landed {
                        let check = e.position.clone();
                        let (height_below, _check_below) = height_above_land(&check, &landscape_vec);
                        if height_below < 0.1 {
                            e.landed();
                        } else {}
                    }
                    for b in bullets.instances.iter_mut() {
                        let distance = b.position.distance(e.position);
                        if e.landed && distance < 1.0 || distance < 0.5 {
                            e.dead();
                            b.dead();
                            explosions.new_instance(e.position);
                            player.score=player.score+1;
                        }
                    }
                    let distance = player.position.distance(e.position);
                    if distance < 1.0 {
                        e.dead();
                        explosions.new_instance(e.position);
                        player_been_hit(&mut player, &mut explosions);
                        player.start_being_dead();
                    }

                }
                canvas.window().gl_swap_window();
            }
            _ => {}
        }


        delay = frame_control.delay();
    }
    Ok(())
}

fn possible_locations_for_enemy(landscape_vec: &Vec<Vec<Landscape>>,player_pos:Vector3<f32>) -> Vec<Vector3<f32>> {
    let mut possible_positions:Vec<Vector3<f32>> = vec![];
    let mut rng = rand::thread_rng();
    possible_positions.push(vec3(player_pos.x,player_pos.y + 3.0,player_pos.z));
    for v in landscape_vec.iter() {
        for l in v.iter() {
            let n = rng.gen_range(0, 6) as i32;
            if n >= 2 {
                possible_positions.push(vec3(l.land_position[0].x, player_pos.y+3.0, l.land_position[0].z));
            }
        }
    }
    return possible_positions;
}

fn player_been_hit(player: &mut PlayerShape, explosions: &mut Explosion) {
    play(HIT_WALL);
    let (pos_left, pos_right, pos_behind) = player.get_explosion_positions();
    explosions.new_instance(pos_left);
    explosions.new_instance(pos_behind);
    explosions.new_instance(pos_right);
    explosions.new_instance(player.position);
}

fn projection_view_camera(mut camera: &mut Camera, player: &mut PlayerShape) -> (Matrix4<f32>, Matrix4<f32>) {
    let mut move_camera_here = player.position.clone();
    let dir = player.get_for_camera() * 6.0;
    move_camera_here += dir;
    camera.Position.x = move_camera_here.x;
    camera.Position.y = move_camera_here.y;
    camera.Position.z = move_camera_here.z;
    camera.updateCameraVectors();

    let view = camera.lookAt(dir * -1.0);
    let projection: Matrix4<f32> = perspective(Deg(camera.Zoom),
                                               SCR_WIDTH as f32 / SCR_HEIGHT as f32, 0.1, FAR);
    (view, projection)
}

fn process_keyboard(event_pump: &mut EventPump, left: &mut bool, right: &mut bool, up: &mut bool, down: &mut bool, fire: &mut bool, pitch_down: &mut bool, pitch_up: &mut bool) -> bool {
    let mut quit = false;
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                quit = true;
            }
            Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                *left = true;
                *right = false;
            }
            Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                *right = true;
                *left = false;
            }
            Event::KeyDown { keycode: Some(Keycode::LShift), .. } => {
                *up = true;
                *down = false
            }
            Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
                *down = true;
                *up = false
            }
            Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                *pitch_down = true;
                *pitch_up = false
            }
            Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                *pitch_down = false;
                *pitch_up = true
            }
            Event::KeyDown { keycode: Some(Keycode::Space), .. } => { *fire = true }
            Event::KeyUp { keycode: Some(Keycode::Left), .. } => { *left = false; }
            Event::KeyUp { keycode: Some(Keycode::Right), .. } => { *right = false; }
            Event::KeyUp { keycode: Some(Keycode::LShift), .. } => { *up = false }
            Event::KeyUp { keycode: Some(Keycode::Z), .. } => { *down = false }
            Event::KeyUp { keycode: Some(Keycode::Space), .. } => { *fire = false }
            Event::KeyUp { keycode: Some(Keycode::Up), .. } => { *pitch_down = false }
            Event::KeyUp { keycode: Some(Keycode::Down), .. } => { *pitch_up = false }
            _ => {}
        }
    }
    return quit;
}

fn height_above_land(position: &Vector3<f32>, landscape_vec: &Vec<Vec<Landscape>>) -> (f32, u8) {
    let mut height_above: f32 = position.y;
    let mut what: u8 = 0;
    'outer_loop: for zz in 0..LAND_Z {
        for xx in 0..LAND_X {
            let l = &landscape_vec[zz][xx];
            if position.x > l.min.x && position.x < l.max.x &&
                position.z > l.min.y && position.z < l.max.y {
                let mut cam = position.clone();
                cam.x = (cam.x - l.min.x) / (CELL * 2.0);// * 0.5;
                cam.z = (cam.z - l.min.y) / (CELL * 2.0);// * 0.5;
                height_above = position.y - l.height[cam.z as usize][cam.x as usize];
                what = l.what[cam.z as usize][cam.x as usize];
                break 'outer_loop;
            }
        }
    }
    return (height_above, what);
}

fn render_landscapes(position: Vector3<f32>,
                     dirs: Vec<Vector3<f32>>,
                     landscape_vec: &Vec<Vec<Landscape>>, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
    let mut render_landscape_hashmap: HashMap<String, (&Landscape, Vector3<f32>)> = HashMap::new();


    let mut try_positions: Vec<Vector3<f32>> = Vec::new();
    try_positions.push(position);
    for d in dirs.iter() {
        let mut dir: Vector3<f32> = d.clone();
        dir.y = 0.0;
        for i in 1..20 {
            let project = position.clone() - dir * (i as f32 * ACTUAL_WIDTH / 4.0);
            try_positions.push(project)
        }
    }


    for test_position in try_positions.iter() {
        let mut offset = vec3(0.0, 0.0, 0.0f32);
        if test_position.x < -ACTUAL_WIDTH / 2.0 { offset.x = ACTUAL_WIDTH * LAND_X as f32; }
        if test_position.z < -ACTUAL_WIDTH / 2.0 { offset.z = ACTUAL_WIDTH * LAND_X as f32; }
        if test_position.x > ACTUAL_WIDTH * LAND_X as f32 { offset.x = -ACTUAL_WIDTH * LAND_X as f32; }
        if test_position.z > ACTUAL_WIDTH * LAND_Z as f32 { offset.z = -ACTUAL_WIDTH * LAND_Z as f32; }

        let at_pos = test_position + offset + vec3(ACTUAL_WIDTH / 2.0, ACTUAL_WIDTH / 2.0, ACTUAL_WIDTH / 2.0);
        let xxx = (at_pos.x / ACTUAL_WIDTH) as i32;
        let zzz = (at_pos.z / ACTUAL_WIDTH) as i32;
        let (l, pos) = get_landscape(zzz, xxx, offset, landscape_vec);
        let key = format!("{},{}", pos.x as i32, pos.z as i32);
        render_landscape_hashmap.insert(key, (l, pos));
    }
    for (l, p) in render_landscape_hashmap.values() {
        l.render(projection, view, *p);
    }
    //println!(" length {}", render_landscape_hashmap.len());
}

fn get_landscape(y: i32, x: i32, offset: Vector3<f32>, landscape_vec: &Vec<Vec<Landscape>>) -> (&Landscape, Vector3<f32>) {
    let mut zz = y;
    let mut xx = x;

    if zz >= LAND_Z as i32 { zz = LAND_Z as i32 - zz; }
    if xx >= LAND_X as i32 { xx = LAND_X as i32 - xx; }
    if zz < 0 { zz = LAND_Z as i32 + zz; }
    if xx < 0 { xx = LAND_X as i32 + xx; }
    //println!("get_landscape {},{} {},{}",y,x,zz,xx);

    let l: &Landscape = &landscape_vec[zz as usize][xx as usize];
    let adj_pos = l.land_position[0] - offset;

    return (l, adj_pos);
}

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

