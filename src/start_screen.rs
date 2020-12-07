extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;

use crate::common::make_title_texture;
use crate::sdl_main::{SCR_WIDTH};

use self::sdl2::render::{Canvas, Texture};
use self::sdl2::video::Window;

pub(crate) struct StartScreen<'a> {
    bernie_soft_title: Texture<'a>,
    car_maze_title: Texture<'a>,
    instructions: Vec<&'a str>,
    pub(crate) bernie_x: i32,
}

impl<'a> StartScreen<'a> {
    pub(crate) fn new(texture_creator: &'a TextureCreator<WindowContext>, font: &Font) -> StartScreen<'a> {
        let get_pixels_surface1 = font.render("Berniesoft").blended(Color::WHITE).map_err(|e| e.to_string()).unwrap();
        let get_pixels_surface2 = font.render("Bi-plane vs Alien...").blended(Color::WHITE).map_err(|e| e.to_string()).unwrap();
        let title1 = make_title_texture(200,
                                        Color::YELLOW,
                                        &texture_creator,
                                        get_pixels_surface1);
        let title2 = make_title_texture(200,
                                        Color::GREEN,
                                        &texture_creator,
                                        get_pixels_surface2);

        let instructions = vec![
            "Fly your bi-plane and shoot as many aliens as possible. If more than 30 aliens remain",
            "on land game over.",
            "You will need to refuel by landing on the grey landing strip",
            "Arrow keys to fly, left shift faster Z slower"
        ];
        StartScreen {
            bernie_soft_title: title1,
            car_maze_title: title2,
            instructions: instructions,
            bernie_x: 0,
        }
    }
    pub fn update(&mut self) {
        self.bernie_x = self.bernie_x + 1;
    }
    pub fn draw_on_canvas(self, canvas: &mut Canvas<Window>, font: &Font, texture_creator: &'a TextureCreator<WindowContext>) -> StartScreen<'a> {
        canvas.copy(&self.bernie_soft_title, None, Some(Rect::new(self.bernie_x, 100, SCR_WIDTH, 200))).unwrap();
        canvas.copy(&self.car_maze_title, None, Some(Rect::new(0, 300, SCR_WIDTH, 200))).unwrap();

        let mut y = 450;
        for line in self.instructions.iter() {
            let font_surface = font.render(line).blended(Color::YELLOW).unwrap();
            let instructions_texture = font_surface.as_texture(&texture_creator).unwrap();

            canvas.copy(&instructions_texture, None, Some(Rect::new(64, y, font_surface.width(), 48))).unwrap();
            y = y + 48;
        }

        self
    }
}
