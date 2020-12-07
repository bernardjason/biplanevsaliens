extern crate cgmath;
extern crate gl;
extern crate glfw;

use std::ffi::CStr;
use std::mem;
use std::os::raw::c_void;
use std::ptr;

use cgmath::*;

use crate::shader::Shader;

use self::gl::types::*;
use rand::Rng;


pub struct ExplosionInstance {
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub clicks: i32,
}

impl ExplosionInstance {
    pub fn _dead(&mut self) {
        self.clicks = 99999999;
    }
}

pub struct Explosion {
    our_shader: Shader,
    vao: u32,
    colour: Vector3<f32>,
    pub instances: Vec<ExplosionInstance>,
}

impl Explosion {
    pub fn new() -> Explosion {
        let (our_shader, _vbo, vao) = unsafe {
            let our_shader = Shader::new(
                "resources/bullet_texture.vs",
                "resources/bullet_texture.fs");

            let size = 0.06;
            let vertices: [f32; 180] = [
// positions       // texture coords
                -size, -size, -size, 0.0, 0.0,
                size, -size, -size, 1.0, 0.0,
                size, size, -size, 1.0, 1.0,
                size, size, -size, 1.0, 1.0,
                -size, size, -size, 0.0, 1.0,
                -size, -size, -size, 0.0, 0.0,
                -size, -size, size, 0.0, 0.0,
                size, -size, size, 1.0, 0.0,
                size, size, size, 1.0, 1.0,
                size, size, size, 1.0, 1.0,
                -size, size, size, 0.0, 1.0,
                -size, -size, size, 0.0, 0.0,
                -size, size, size, 1.0, 0.0,
                -size, size, -size, 1.0, 1.0,
                -size, -size, -size, 0.0, 1.0,
                -size, -size, -size, 0.0, 1.0,
                -size, -size, size, 0.0, 0.0,
                -size, size, size, 1.0, 0.0,
                size, size, size, 1.0, 0.0,
                size, size, -size, 1.0, 1.0,
                size, -size, -size, 0.0, 1.0,
                size, -size, -size, 0.0, 1.0,
                size, -size, size, 0.0, 0.0,
                size, size, size, 1.0, 0.0,
                -size, -size, -size, 0.0, 1.0,
                size, -size, -size, 1.0, 1.0,
                size, -size, size, 1.0, 0.0,
                size, -size, size, 1.0, 0.0,
                -size, -size, size, 0.0, 0.0,
                -size, -size, -size, 0.0, 1.0,
                -size, size, -size, 0.0, 1.0,
                size, size, -size, 1.0, 1.0,
                size, size, size, 1.0, 0.0,
                size, size, size, 1.0, 0.0,
                -size, size, size, 0.0, 0.0,
                -size, size, -size, 0.0, 1.0
            ];
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


            our_shader.useProgram();

            (our_shader, vbo, vao)
        };

        Explosion {
            our_shader: our_shader,
            vao,
            colour: vec3(1.0, 1.0, 0.0),
            instances: Vec::<ExplosionInstance>::new(),
        }
    }

    pub fn new_instance(&mut self, position: Vector3<f32>) {
        let mut rng = rand::thread_rng();
        let middle = 2;
        for yy in 1..4 {
            for xx in 0..middle * 2 {
                for zz in 0..middle * 2 {
                    let dir: Vector3<f32> = vec3(
                        (xx - middle) as f32 / 0.2 * rng.gen_range(0.2, 0.8) * yy as f32,
                        -1.0f32 * rng.gen_range(4.0, 8.0) * yy as f32,
                        (zz - middle) as f32 / 0.2 * rng.gen_range(0.2, 0.8) * yy as f32);
                    let instance = ExplosionInstance {
                        position,
                        direction: dir,
                        clicks: rng.gen_range(0,50)
                    };
                    self.instances.push(instance);
                }
            }
        }
    }

    pub fn update_explosion(&mut self, delta_time: f32) {
        let speed = 0.02f32;

        let mut i = self.instances.len();
        while i >= 1 {
            i = i - 1;
            let mut b = self.instances.get_mut(i).unwrap();
            b.position -= b.direction * delta_time * speed;
            b.clicks = b.clicks + 1;
            if b.clicks > 100 {
                self.instances.remove(i);
            }
        }
    }
    pub fn render(&mut self, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        let mut rng = rand::thread_rng();
        unsafe {
            self.our_shader.useProgram();
            self.our_shader.setMat4(c_str!("view"), view);
            self.our_shader.setMat4(c_str!("projection"), projection);
            gl::BindVertexArray(self.vao);
        }
        self.colour.y = self.colour.y + rng.gen_range(0.1, 0.4);
        if self.colour.y > 0.5 { self.colour.y = 0.0; }


        for i in 0..self.instances.len() {
            let b = self.instances.get(i).unwrap();
            let matrix = Matrix4::<f32>::from_translation(b.position);
            if rng.gen_range(1, 11) > 8 {
                self.colour.x = 0.0;//self.colour.x + rng.gen_range(0.1, 0.4);
                self.colour.z = 0.0;//self.colour.x + rng.gen_range(0.1, 0.4);
            } else {
                self.colour.x = 0.8;//self.colour.x + rng.gen_range(0.1, 0.4);
                self.colour.z = 0.8;//self.colour.x + rng.gen_range(0.1, 0.4);
            }
            unsafe {
                self.our_shader.setMat4(c_str!("model"), &matrix);
                self.our_shader.setVec3(c_str!("colour"), self.colour.x, self.colour.y, self.colour.z);

                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }
        }
    }
}