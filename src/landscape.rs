extern crate gl;
extern crate glfw;

use std::ffi::CStr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

use cgmath::{Matrix4, vec2, vec3, Vector2, Vector3};
use image::GenericImage;
use noise::{NoiseFn, Value};
use rand::Rng;

use crate::shader::Shader;

use self::gl::types::*;

/*
landscape either water,grass,earth,concrete

 */

pub const SQUARES: usize = 96;
pub const CELL: f32 = 0.25;
pub const ACTUAL_WIDTH: f32 = SQUARES as f32 * CELL * 2.0;
pub const LAND_X: usize = 4;
pub const LAND_Z: usize = 4;
pub const WATER: u8 = 0;
pub const LAND: u8 = 1;
pub const AIR_STRIP: u8 = 2;
pub const SAND: u8 = 3;

pub struct Landscape {
    our_shader: Shader,
    vao: u32,
    texture1: u32,
    pub land_position: [Vector3<f32>; 1],
    pub height: [[f32; SQUARES + 1]; SQUARES + 1],
    pub what: [[u8; SQUARES + 1]; SQUARES + 1],
    pub min: Vector2<f32>,
    pub max: Vector2<f32>,
    pub centre: Vector2<f32>,
    wave: Vector3<f32>,
}

static mut SINE: f32 = 0.0;

impl Landscape {
    pub fn new(noise: &Value, left: Option<[[f32; SQUARES + 1]; SQUARES + 1]>, below: Option<[[f32; SQUARES + 1]; SQUARES + 1]>, centre_x: f32, centre_z: f32) -> Landscape {
        let (our_shader, _vbo, vao, texture1, height, what) = unsafe {
            let mut rng = rand::thread_rng();

            let get_sine = |radius: f32| -> f32  { (radius).to_radians().sin() };
            let get_cosine = |radius: f32| -> f32  { (radius).to_radians().cos() };
            let our_shader = Shader::new(
                "resources/ground.vs",
                "resources/ground.fs");
            let mut height: [[f32; SQUARES + 1]; SQUARES + 1] = [[0.0; SQUARES + 1]; SQUARES + 1];
            let mut what: [[u8; SQUARES + 1]; SQUARES + 1] = [[WATER; SQUARES + 1]; SQUARES + 1];
            left.map(|left_height| {
                for zz in 0..SQUARES + 1 {
                    height[zz][0] = left_height[zz][SQUARES];
                }
            });
            below.map(|below_height| {
                for xx in 0..SQUARES + 1 {
                    height[0][xx] = below_height[SQUARES][xx];
                }
            });

            let start_sine = SINE;
            let from_side = rng.gen_range(20, SQUARES / 2);
            for zz in from_side..SQUARES - from_side {
                SINE = start_sine + zz as f32;
                for xx in from_side..SQUARES - from_side {
                    let v: f64 = noise.get([xx as f64, zz as f64]) / 2.0;

                    if v > 0.49 {
                        let mut start_height = rng.gen_range(2.0, 3.0);
                        height[zz][xx] = start_height;
                        what[zz][xx] = LAND;
                        for scale in 1..SQUARES {
                            for r in 0..45 {
                                let x = get_cosine(r as f32 * 16.0) * scale as f32 + xx as f32;
                                let y = get_sine(r as f32 * 16.0) * scale as f32 + zz as f32;
                                if x > 1.0 && x < SQUARES as f32 - 1.0 && y > 1.0 && y < SQUARES as f32 - 1.0 {
                                    if height[y as usize][x as usize] < start_height {
                                        height[y as usize][x as usize] = start_height;
                                        what[y as usize][x as usize] = LAND;
                                    }
                                }
                            }
                            let less: f32 = (noise.get([xx as f64, zz as f64]).abs() / 16.0) as f32;
                            start_height = start_height - less * scale as f32 / 5.0;
                        }
                    }
                }
            }
            for zz in 2..SQUARES-3 {
                for xx in 2..SQUARES-3 {
                    if what[zz][xx] == LAND  {
                        for zzz in zz-2..zz+2 {
                            for xxx in xx-2..xx+2 {
                                if what[zzz][xxx] == WATER {
                                    what[zzz][xxx] = SAND;
                                }
                            }
                        }
                    }
                }
            }

            let landing_width = 6;
            let landing_length = SQUARES / 3;
            let centre = SQUARES / 2;
            let landing_height = height[centre][centre];
            if landing_height > 1.0 && rng.gen_range(0, 5) >= 2 {
                Landscape::landing_strip(&mut height, &mut what, landing_width, landing_length, centre, landing_height)
            }

            let mut vertices: [f32; SQUARES * SQUARES * 30] = [0.0; SQUARES * SQUARES * 30];
            let image_add = 1.0 / 10.0;
            let mut col: f32 = 0.0;
            let mut water_flip = 1.0;
            for zz in 0..SQUARES {
                let offset = SQUARES * 30 * zz;
                let z: f32 = CELL * 2.0 * (zz as f32 - (SQUARES as f32 / 2.0)) as f32;
                for xx in 0..SQUARES {
                    water_flip = water_flip + 0.3;
                    let x: f32 = CELL * 2.0 * (xx as f32 - (SQUARES as f32 / 2.0)) as f32;
                    let py1: f32 = height[zz][xx];
                    let py2: f32 = height[zz][xx + 1];
                    let ny1: f32 = height[zz + 1][xx];
                    let ny2: f32 = height[zz + 1][xx + 1];

                    col = Landscape::next_tex(xx,zz,col, image_add);
                    //if py1 == 0.0 && py2 == 0.0 {
                    if what[zz][xx] == WATER {
                        col = 5.0 * image_add + (image_add * (water_flip as i32 % 3 + 0) as f32);
                    }
                    //if py1 >0.0 && py1 < 0.99 && py2 > 0.0 && py2 < 0.99 { col = 0.4  ; }
                    if what[zz][xx] == SAND {
                        col = 0.4  ;
                    }
                    if what[zz][xx] == AIR_STRIP {
                        col = 8.0 * image_add + (image_add * (water_flip as i32 % 2 + 0) as f32);
                    }

                    let c: [f32; 30] = [
                        x + -CELL, py1, z + -CELL, col, 0.0,
                        x + CELL, py2, z + -CELL, col + image_add, 0.0,
                        x + CELL, ny2, z + CELL, col + image_add, 1.0,
                        x + CELL, ny2, z + CELL, col + image_add, 1.0,
                        x + -CELL, ny1, z + CELL, col, 1.0,
                        x + -CELL, py1, z + -CELL, col, 0.0,
                    ];
                    for i in 0..30 {
                        vertices[offset + i + xx * 30] = c[i];
                    }
                }
                water_flip = water_flip + 0.3;
                col = Landscape::next_tex(0,zz,col, image_add);
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
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            let img = image::open(&Path::new("resources/textures/landscape.png")).expect("Failed to load texture");
            let data = img.raw_pixels();
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
            our_shader.setInt(c_str!("texture1"), 0);

            (our_shader, vbo, vao, texture1, height, what)
        };
        let land_position: [Vector3<f32>; 1] = [vec3(centre_x, 0.0, centre_z)];
        let min = vec2(centre_x - SQUARES as f32 * CELL, centre_z - SQUARES as f32 * CELL);
        let max = vec2(centre_x + SQUARES as f32 * CELL, centre_z + SQUARES as f32 * CELL);
        let centre = vec2(centre_x + SQUARES as f32 * CELL / 2.0, centre_z + SQUARES as f32 * CELL / 2.0);

        //println!("Created landscape {},{}", centre_x, centre_z);
        Landscape {
            our_shader,
            vao,
            texture1,
            land_position,
            height,
            what,
            min,
            max,
            centre,
            wave: vec3(1.0, 0.0, 0.0),
        }
    }

    fn landing_strip(height: &mut [[f32; 97]; 97], what: &mut [[u8; 97]; 97], landing_width: i32, landing_length: usize, centre: usize, landing_height: f32) {
        let mut rng = rand::thread_rng();
        let angle: f32 = rng.gen_range(0, 360) as f32;
        let r = 1.0;
        let add_x: f32 = angle.to_radians().sin() * r;
        let add_z: f32 = angle.to_radians().cos() * r;
        let i_add_x: f32 = (270.0 + angle).to_radians().sin() * r;
        let i_add_z: f32 = (270.0 + angle).to_radians().cos() * r;


        let mut xx = centre as f32 - (add_x * landing_length as f32 / 2.0);
        let mut zz = centre as f32 - (add_z * landing_length as f32 / 2.0);
        for _i in 0..landing_length {
            let z_index = zz as usize;
            let x_index = xx as usize;
            height[z_index][x_index] = landing_height;
            what[z_index][x_index] = AIR_STRIP;
            {
                let mut xx = xx;
                let mut zz = zz;
                for _ii in 0..landing_width {
                    for round_error in 0..2 { // make sure no gaps
                        height[round_error + zz as usize - 1][round_error + xx as usize - 1] = landing_height;
                        what[round_error + zz as usize - 1][round_error + xx as usize - 1] = AIR_STRIP;
                    }
                    xx = xx + i_add_x;
                    zz = zz + i_add_z
                }
            }
            xx = xx + add_x;
            zz = zz + add_z
        }
    }

    fn next_tex(_x:usize,_z:usize,current_col: f32, image_add: f32) -> f32 {
        let mut col: f32 = current_col;
        col = col + image_add ; //* x as f32 /z as f32 ;
        if col >= image_add * 4.0 {
            col = 0.0;
        }
        //println!("Col is {}", col);
        return col;
    }
    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();
        self.wave.y = self.wave.y + (self.wave.x * rng.gen_range(0.0, 0.005));
        if self.wave.y > 0.2 {
            self.wave.x = -1.0;
        }
        if self.wave.y <= 0.0 {
            self.wave.x = 1.0;
        }
    }

    pub fn render(&self, projection: &Matrix4<f32>, view: &Matrix4<f32>, at_position: Vector3<f32>) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture1);
            self.our_shader.useProgram();
            self.our_shader.setMat4(c_str!("projection"), projection);
            self.our_shader.setMat4(c_str!("view"), view);
            self.our_shader.setVec3(c_str!("wave"), self.wave.x, self.wave.y, self.wave.z);
            gl::BindVertexArray(self.vao);
            let model: Matrix4<f32> = Matrix4::from_translation(at_position);
            self.our_shader.setMat4(c_str!("model"), &model);
            gl::DrawArrays(gl::TRIANGLES, 0, (SQUARES * SQUARES) as i32 * 6);
        }
    }
}
