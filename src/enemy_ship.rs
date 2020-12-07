extern crate cgmath;
extern crate gl;
extern crate glfw;

use std::ffi::CStr;

use cgmath::*;

use crate::model::Model;
use crate::shader::Shader;

pub struct EnemyShipInstance {
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub clicks: i32,
    pub landed: bool,
}

impl EnemyShipInstance {
    pub fn dead(&mut self) {
        self.clicks = 99999999;
    }
    pub fn landed(&mut self) {
        if self.position.y > 0.1 {
            self.landed = true;
        } else {
            self.clicks = 99999999;
        }
    }
    pub fn new(position: Vector3<f32>) -> EnemyShipInstance {
        EnemyShipInstance {
            position,
            direction: vec3(0.0, -1.0f32, 0.0),
            clicks: 0,
            landed: false,
        }
    }
}

pub struct EnemyShip {
    our_shader: Shader,
    model: Model,
    pub instances: Vec<EnemyShipInstance>,
    alive:i32,
    pub(crate) landed:i32,
}

impl EnemyShip {
    pub fn new() -> EnemyShip {
        let (our_shader, our_model) = {
            //gl::Enable(gl::DEPTH_TEST);

            let our_shader = Shader::new(
                "resources/enemy.vs",
                "resources/enemy.fs");

            let our_model = Model::new("resources/objects/alien.obj");


            (our_shader, our_model)
        };

        EnemyShip {
            our_shader,
            model: our_model,
            instances: Vec::<EnemyShipInstance>::new(),
            alive:0,
            landed:0,
        }
    }
    pub fn reset(&mut self) {
        self.instances.clear();
        self.alive = 0;
        self.landed = 0;
    }

    pub fn enemy_update(&mut self ) -> i32 {
        self.alive = 0;
        self.landed = 0;
        for x in self.instances.iter() {
            if x.landed {
                self.landed = self.landed + 1;
            } else {
                self.alive = self.alive + 1;
            }
        }
        return self.alive;
    }


    pub(crate) fn new_instance(&mut self, position: Vector3<f32>) -> usize {
        let instance = EnemyShipInstance::new(position);
        //println!("new alien {} {},{},{}",self.instances.len(),instance.position.x,instance.position.y,instance.position.z);
        self.instances.push(instance);
        return self.instances.len();
    }

    pub fn update_enemy(&mut self, delta_time: f32) {
        let speed = 0.3f32;

        let mut i = self.instances.len();
        while i >= 1 {
            i = i - 1;
            let mut b = self.instances.get_mut(i).unwrap();
            if !b.landed {
                b.position += b.direction * delta_time * speed;
                b.clicks = b.clicks + 1;
            }
            if b.clicks > 3000 || b.position.y < 0.0 {
                self.instances.remove(i);
            }
        }
    }
    pub fn render(&self, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        unsafe {
            self.our_shader.useProgram();
            self.our_shader.setMat4(c_str!("view"), view);
            self.our_shader.setMat4(c_str!("projection"), projection);
        }
        for i in 0..self.instances.len() {
            let b = self.instances.get(i).unwrap();
            let scale = if b.landed  {
                0.05
            } else {
                0.02
            };
            let matrix = Matrix4::<f32>::from_translation(b.position) * Matrix4::from_scale(scale);
            unsafe {
                self.our_shader.setMat4(c_str!("model"), &matrix);
                self.model.Draw(&self.our_shader);
            }
        }
    }
}