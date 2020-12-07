extern crate cgmath;
extern crate gl;
extern crate glfw;

use std::collections::VecDeque;
use std::ffi::CStr;

use cgmath::*;
//use cgmath::{Vector3, vec3, Matrix4, Vector2, vec2, Rad, Deg, Rotation3, Transform3, Transform, Decomposed, Quaternion, Euler};

use crate::landscape::{ACTUAL_WIDTH, AIR_STRIP, LAND_X, LAND_Z};
use crate::model::Model;
use crate::shader::Shader;

pub struct PlayerShape {
    pub our_shader: Shader,
    pub model: Model,
    pub position: Vector3<f32>,
    pub matrix: Matrix4<f32>,
    pub applied: Matrix4<f32>,
    pub camera_dir:Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub speed:f32,
    pub fuel:f32,
    pub lives:i32,
    pub score:i32,
    camera_queue: VecDeque<Vector3<f32>>,
    spinning:f32,
    pub player_crashed:i32,
}
const START_POSITION_X:f32 = 96.0;
const START_POSITION_Y:f32 =8.0;
const START_POSITION_Z:f32 =96.0;

const SLOW: f32 = 2.0;
const FUEL: f32 = 100000.0;
const LIVES: i32 = 5;

impl PlayerShape {
    pub fn new() -> PlayerShape {
        let (our_shader, our_model) = {

            let our_shader = Shader::new(
                "resources/model_loading.vs",
                "resources/plane_loading.fs");

            let our_model = Model::new("resources/objects/simpleplane/simpleplane.obj"); //invade.obj");

            (our_shader, our_model)
        };

        PlayerShape {
            our_shader: our_shader,
            model: our_model,
            position: vec3(START_POSITION_X, START_POSITION_Y, START_POSITION_Z),
            matrix: Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0)),
            applied: Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0)),
            camera_dir: vec3(0.0,0.0,1.0),
            yaw: 90.0,
            pitch: 0.0,
            roll: 90.0,
            speed: SLOW,
            fuel:FUEL,
            lives:LIVES,
            score:0,
            camera_queue: VecDeque::new(),
            spinning:0.0,
            player_crashed:0,
        }
    }
    pub fn reset_player(&mut self) {
        self.position= vec3(START_POSITION_X, START_POSITION_Y, START_POSITION_Z);
        self.matrix= Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0));
        self.applied= Matrix4::<f32>::from_translation(vec3(0.0, 0.0, 0.0));
        self.camera_dir= vec3(0.0,0.0,1.0);
        self.yaw= 90.0;
        self.pitch= 0.0;
        self.roll= 90.0;
        self.camera_queue= VecDeque::new();
        self.spinning=0.0;
        self.fuel = FUEL;
        self.speed = SLOW;
    }
    pub fn game_reset_player(&mut self) {
        self.lives = LIVES;
        self.score = 0;
        self.reset_player();
    }
    pub fn hit(&mut self, what:u8) -> bool {
        if what == AIR_STRIP && self.speed == SLOW {
            let level = self.get_level();
            if level.y.abs() <= 0.2 {
                self.fuel = FUEL;
                //println!("Landed {} yaw={},pithc={},roll={}",level.y.abs(),self.yaw,self.pitch,self.roll);
                return true;
            } else {
                //println!("CRASHED {} yaw={},pithc={},roll={}",level.y.abs(),self.yaw,self.pitch,self.roll);
            }
        }
        self.start_being_dead();
        return false;
    }

    pub fn start_being_dead(&mut self) {
        self.lives = self.lives - 1;
        self.player_crashed = 80;
    }
    pub fn player_ok(& mut self) -> bool {
        if self.player_crashed == 0 {
            return true;
        }
        self.player_crashed = self.player_crashed -1;
        if self.player_crashed == 0 {
            self.reset_player();
        }
        return false;
    }
    pub fn update_player(&mut self) {
        self.matrix = Matrix4::<f32>::from_translation(self.position);
        self.matrix = self.matrix * Matrix4::from_scale(0.03);
        self.matrix = self.matrix * self.applied;

        self.fuel = self.fuel - self.speed;
        if self.fuel < 0.0  && self.position.y > 0.0 && self.player_crashed == 0 {
            self.position.y = self.position.y - 0.10;
        }
    }
    pub fn faster(&mut self) {
        self.speed = self.speed + 0.10;
        if self.speed > 7.0 {
            self.speed = 7.0;
        }
    }
    pub fn slower(&mut self) {
        self.speed = self.speed - 0.10;
        if self.speed < SLOW {
            self.speed = SLOW;
        }
    }
    pub fn forward(&mut self, direction: f32, delta_time: f32) -> Vector3<f32> {

        let vec = vec3(0.0,0.0,1.0f32);
        let dir = self.applied.transform_vector(vec);

        if direction < 0.0 {
            self.position += dir * self.speed * delta_time;
        } else {
            self.position -= dir * self.speed * delta_time;
        }

        self.position = normalize_position_to_world(self.position);
        return dir;
    }

    pub fn get_explosion_positions(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let mut pl = self.position.clone();
        let mut pr = self.position.clone();
        let mut behind = self.position.clone();
        pl = pl + self.applied.transform_vector( vec3(-1.0f32,0.0,0.0));
        pl = pl + self.applied.transform_vector( vec3(0.0f32,-0.25,0.0));
        pr = pr + self.applied.transform_vector( vec3(1.0f32,0.0,0.0));
        pr = pr + self.applied.transform_vector( vec3(0.0f32,-0.25,0.0));
        behind = behind + self.applied.transform_vector( vec3(0.0f32,0.0,1.0));
        behind = behind + self.applied.transform_vector( vec3(0.0f32,-0.25,0.0));

        return (pl,pr,behind);
    }

    pub fn get_bullet_direction(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let mut pl = self.position.clone();
        let mut pr = self.position.clone();
        pl = pl + self.applied.transform_vector( vec3(-1.0f32,0.0,0.0));
        pl = pl + self.applied.transform_vector( vec3(0.0f32,-0.25,0.0));
        pr = pr + self.applied.transform_vector( vec3(1.0f32,0.0,0.0));
        pr = pr + self.applied.transform_vector( vec3(0.0f32,-0.25,0.0));

        let vec = vec3(0.0,0.0,1.0f32);
        return (pl,pr,self.applied.transform_vector(vec));
    }
    pub fn get_level(&self) -> Vector3<f32> {
        let level = self.applied.transform_vector( vec3(0.0f32,0.0,1.0));
        return level;
    }
    pub fn get_direction_adjust_angle(&self, adjust: f32) -> Vector3<f32> {

        let adjusted = self.camera_dir;
        let r:Basis3<f32> = Rotation3::from_angle_y( Deg(adjust)) ;
        r.rotate_vector(adjusted)
    }
    pub fn update_camera(&mut self) {
        if self.camera_queue.len() > 0 {
            self.camera_dir = self.camera_queue.pop_front().unwrap();
        } else {
            self.camera_dir = vec3(0.0,0.0,1.0f32);
            self.camera_dir = self.applied.transform_vector(self.camera_dir);
        }
        if self.camera_queue.len() > 40 {
            while self.camera_queue.len() > 40 {
                self.camera_queue.pop_front();
            }
            //println!("Big queue {}",self.camera_queue.len());
        }
    }
    pub fn queue_camera_for_later(&mut self) {
        let dir = vec3(0.0,0.0,1.0f32);
        let dir = self.applied.transform_vector(dir);

        self.camera_queue.push_back(dir);

    }
    pub fn get_for_camera(&self) -> Vector3<f32> {

        self.camera_dir
    }
    //pub fn ProcessKeyboardRotate(&mut self, rotate: f32, delta_time: f32) { self.yaw += rotate; }
    pub fn process_keyboard_pitch(&mut self, rotate: f32, _delta_time: f32) {
        self.pitch += rotate;
        self.pitch = self.wrap_around_angle(self.pitch);
        self.applied = self.applied * Matrix4::<f32>::from_angle_x(Deg(-rotate)) ;



    }
    pub fn process_keyboard_roll(&mut self, rotate: f32, _delta_time: f32) {

        self.roll += rotate;
        self.roll = self.wrap_around_angle(self.roll);
        self.applied = self.applied * Matrix4::<f32>::from_angle_z(Deg(-rotate)) ;
    }
    fn wrap_around_angle(&self,a:f32) -> f32 {
        if a >= 360.0 {
            return a - 360.0;
        }
        if a < 0.0 {
            return a + 360.0;
        }

        return a;
    }
    pub fn render(&mut self,view:&Matrix4<f32>,projection:&Matrix4<f32>) {
        self.spinning = self.spinning + 1.0;
        unsafe {
            self.our_shader.useProgram();
            self.our_shader.setMat4(c_str!("projection"), &projection);
            self.our_shader.setMat4(c_str!("view"), &view);
            self.our_shader.setVec3(c_str!("spinning"),self.spinning,0.0,0.0);

            self.our_shader.setMat4(c_str!("model"), &self.matrix);
            self.model.Draw(&self.our_shader);
        }

    }
}

pub fn normalize_position_to_world(position: Vector3<f32>) -> Vector3<f32> {
    let mut norm = position.clone();
    if norm.x >= ACTUAL_WIDTH * LAND_X as f32 { norm.x = norm.x - ACTUAL_WIDTH * LAND_X as f32; }
    if norm.z >= ACTUAL_WIDTH * LAND_Z as f32 { norm.z = norm.z - ACTUAL_WIDTH * LAND_Z as f32; }
    if norm.x < -ACTUAL_WIDTH / 2.0 { norm.x = ACTUAL_WIDTH * LAND_X as f32 + norm.x; }
    if norm.z < -ACTUAL_WIDTH / 2.0 { norm.z = ACTUAL_WIDTH * LAND_Z as f32 + norm.z; }
    return norm;
}