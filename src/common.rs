#![allow(non_snake_case)]
#![allow(dead_code)]
extern crate glfw;
extern crate sdl2;

use std::fs::File;
use std::io::{self, BufRead};
/// Common code that the original tutorials repeat over and over and over and over

use std::os::raw::c_void;
use std::path::Path;
use std::sync::mpsc::Receiver;

use gl;
use image;
use image::DynamicImage::*;
use image::GenericImage;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{BlendMode, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::camera::Camera;
use crate::camera::Camera_Movement::{BACKWARD, FORWARD, LEFT, RIGHT};
use crate::sdl_main::SCR_WIDTH;

use self::glfw::{Action, Key};
use self::sdl2::render::Texture;


/// Event processing function as introduced in 1.7.4 (Camera Class) and used in
/// most later tutorials
pub fn process_events(events: &Receiver<(f64, glfw::WindowEvent)>,
                  firstMouse: &mut bool,
                  lastX: &mut f32,
                  lastY: &mut f32,
                  camera: &mut Camera) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::CursorPos(xpos, ypos) => {
                let (xpos, ypos) = (xpos as f32, ypos as f32);
                if *firstMouse {
                    *lastX = xpos;
                    *lastY = ypos;
                    *firstMouse = false;
                }

                let xoffset = xpos - *lastX;
                let yoffset = *lastY - ypos; // reversed since y-coordinates go from bottom to top

                *lastX = xpos;
                *lastY = ypos;

                camera.ProcessMouseMovement(xoffset, yoffset, true);
            }
            glfw::WindowEvent::Scroll(_xoffset, yoffset) => {
                camera.ProcessMouseScroll(yoffset as f32);
            }
            _ => {}
        }
    }
}

/// Input processing function as introduced in 1.7.4 (Camera Class) and used in
/// most later tutorials
pub fn processInput(window: &mut glfw::Window, delta_time: f32, camera: &mut Camera) {
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true)
    }

    if window.get_key(Key::Up) == Action::Press {
        camera.ProcessKeyboard(FORWARD, delta_time);
    }
    if window.get_key(Key::Down) == Action::Press {
        camera.ProcessKeyboard(BACKWARD, delta_time);
    }
    if window.get_key(Key::Left) == Action::Press {
        camera.ProcessKeyboard(LEFT, delta_time);
    }
    if window.get_key(Key::Right) == Action::Press {
        camera.ProcessKeyboard(RIGHT, delta_time);
    }
}

/// utility function for loading a 2D texture from file
/// ---------------------------------------------------
#[allow(dead_code)]
pub unsafe fn loadTexture(path: &str) -> u32 {
    let mut textureID = 0;

    gl::GenTextures(1, &mut textureID);
    let img = image::open(&Path::new(path)).expect("Texture failed to load");
    let format = match img {
        ImageLuma8(_) => gl::RED,
        ImageLumaA8(_) => gl::RG,
        ImageRgb8(_) => gl::RGB,
        ImageRgba8(_) => gl::RGBA,
    };

    let data = img.raw_pixels();

    gl::BindTexture(gl::TEXTURE_2D, textureID);
    gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, img.width() as i32, img.height() as i32,
        0, format, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const c_void);
    gl::GenerateMipmap(gl::TEXTURE_2D);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    textureID
}


#[derive(PartialEq, Eq)]
pub enum Event {
    None,
    Up,
    Down,
    Left,
    Right,
    Shift,
    Space,
    OffUp,
    OffDown,
    OffLeft,
    OffRight,
    OffShift,
    OffSpace,
}

pub fn make_title_texture<'a>(height: u32, colour: Color,
                              texture_creator: &'a TextureCreator<WindowContext>, get_pixels_surface: Surface) -> Texture<'a> {
    let size = get_pixels_surface.width() * get_pixels_surface.height();
    let mut pixel_buffer = Vec::with_capacity(size as usize);
    pixel_buffer.resize(size as usize, 0);

    let off_top = 11;
    get_pixels_surface.with_lock(|buffer: &[u8]| {
        for y in off_top..get_pixels_surface.height() {
            for x in 0..get_pixels_surface.width() {
                let index = (y * get_pixels_surface.pitch() + x * 4) as usize;
                let val = buffer[index + 3];
                if val > 0 {
                    let index = ((y - off_top) * get_pixels_surface.width() + x) as usize;
                    pixel_buffer[index] = 1;
                }
            }
        }
    }
    );

    let mut title_by: Texture =
        texture_creator.create_texture_streaming(PixelFormatEnum::RGBA8888, SCR_WIDTH as u32, height).expect("texture");
    title_by.set_blend_mode(BlendMode::Blend);
        title_by.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            let gap = 4;
            let mut slant = 0;
            for oppy in 1..get_pixels_surface.height() {
                let y = get_pixels_surface.height() - oppy - 1;
                for x in 0..get_pixels_surface.width() {
                    let index = y * get_pixels_surface.width() + x;
                    let val = pixel_buffer[index as usize];
                    if val == 1 {
                        let size = gap / 2;
                        for yy in y * gap..y * gap + size {
                            for xx in x * gap..x * gap + size {
                                let offset = ((yy) * pitch as u32 + (xx + slant) * 4) as usize;
                                buffer[offset] = 255;
                                buffer[offset + 1] = colour.r;
                                buffer[offset + 2] = colour.g;
                                buffer[offset + 3] = colour.b;
                            }
                        }
                    }
                }
                slant = slant + 2;
            }
        }).unwrap();

    title_by
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
