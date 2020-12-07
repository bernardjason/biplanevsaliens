extern crate gl;
extern crate glfw;

use std::ffi::CStr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use cgmath::{Matrix4, vec3, Vector3};
use image::GenericImage;

use crate::sdl_main::{SCR_HEIGHT, SCR_WIDTH};
use crate::shader::Shader;

use self::gl::types::*;


const CHARS_PER_LINE: f32 = 31.0;
const CHAR_LINES: f32 = 3.0;

pub struct DrawText {
    shader: Shader,
    vao: u32,
    texture: u32,
}

impl DrawText {
    pub fn new() -> DrawText {
        let (our_shader, vao, texture1) = unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            let our_shader = Shader::new(
                "resources/font_texture.vs",
                "resources/font_texture.fs");

            let posx: f32 = 1.0 / CHARS_PER_LINE*0.5;
            let posy: f32 = 1.0 / CHAR_LINES * 0.25;
            let cw: f32 = 1.0 / CHARS_PER_LINE;
            let ch: f32 = 1.0 / CHAR_LINES;
            let mut vertices: [f32; CHARS_PER_LINE as usize * 30 * CHAR_LINES as usize] = [0.0; CHARS_PER_LINE as usize * 30 * CHAR_LINES as usize];
            for y in 0..CHAR_LINES as usize {
                //let offset  = (CHARS_PER_LINE * 30.0) as usize * y  ; //(CHAR_LINES-1.0 - y as f32 ) as usize;
                let offset = (CHARS_PER_LINE * 30.0) as usize * (2 - y); //(CHAR_LINES-1.0 - y as f32 ) as usize;
                for x in 0..CHARS_PER_LINE as usize {
                    let ix: f32 = 0.0;
                    let iy: f32 = 0.0;
                    let imagex: f32 = x as f32 * cw;
                    let imagey: f32 = y as f32 * ch;
                    let v = [
                        ix, iy, 0.0, imagex, imagey,
                        ix + posx, iy, 0.0, imagex + cw, imagey,
                        ix + posx, iy + posy, 0.0, imagex + cw, imagey + ch,
                        ix + posx, iy + posy, 0.0, imagex + cw, imagey + ch,
                        ix, iy + posy, 0.0, imagex, imagey + ch,
                        ix, iy, 0.0, imagex, imagey,
                    ];
                    for i in 0..30 {
                        vertices[offset + i + x * 30] = v[i];
                    }
                }
            }

            let (mut vbo, mut vao) = (0, 0);
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                           &vertices[0] as *const f32 as *const c_void,
                           gl::STATIC_DRAW);

            let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
            gl::EnableVertexAttribArray(1);


            let mut texture1 = 0;
            gl::GenTextures(1, &mut texture1);
            gl::BindTexture(gl::TEXTURE_2D, texture1);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            let img = image::open(&Path::new("resources/textures/font.png")).expect("Failed to load texture");
            let data = img.flipv().raw_pixels();
            gl::TexImage2D(gl::TEXTURE_2D,
                           0,
                           gl::RGBA as i32,
                           img.width() as i32,
                           img.height() as i32,
                           0,
                           gl::RGBA,
                           gl::UNSIGNED_BYTE,
                           &data[0] as *const u8 as *const c_void);
            gl::GenerateMipmap(gl::TEXTURE_2D);

            our_shader.useProgram();

            (our_shader, vao, texture1)
        };
        DrawText{
            shader: our_shader,
            vao: vao,
            texture: texture1
        }
    }
    pub unsafe fn draw_text(&self,message: &str, x: f32, y: f32, colour:Vector3<f32>) {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, self.texture);
        self.shader.useProgram();
        self.shader.setVec3(c_str!("colour"), colour.x,colour.y,colour.z);

        gl::BindVertexArray(self.vao);

        let char_vec: Vec<char> = message.chars().collect();
        let scale_x = SCR_WIDTH as f32;
        let scale_y = SCR_HEIGHT as f32 / 2.0;

        let xx: f32 = x * 2.0;
        let yy: f32 = y / scale_y - 1.0;
        let mut letter = 0;
        for c in char_vec {
            if c as u8 > 32 {
                let another_position: [Vector3<f32>; 1] = [vec3(((xx + letter as f32 * 32.0) as f32 / scale_x) - 1.0, yy, 0.0)];
                let model: Matrix4<f32> = Matrix4::from_translation(another_position[0]);
                self.shader.setMat4(c_str!("model"), &model);

                let triangles = if c >= 'A' && c <= '_' {
                    let abcdefg = c as u8 - 'A' as u8;
                    31 * 6 + abcdefg as i32 * 6
                } else if c > '_' {
                    let abcdefg = c as u8 - '`' as u8;
                    31 * 6 * 2 + abcdefg as i32 * 6
                } else {
                    let abcdefg = c as u8 - '!' as u8;
                    abcdefg as i32 * 6
                };

                gl::DrawArrays(gl::TRIANGLES, triangles, 6);
            }
            letter = letter + 1;
        }
    }
}
