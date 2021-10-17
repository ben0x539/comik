use anyhow::Result;
use std::{path::PathBuf, str::FromStr, sync::Arc};

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, video::WindowContext, render::TextureCreator};

use crate::{providers::{CollectionProvider, file_system::FileSystemCollectionProvider}, resource::TextureManager};

#[derive(Default)]
pub struct AppState {}

pub struct WindowingBits {
    sdl_context: sdl2::Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    texture_creator: TextureCreator<WindowContext>,
}

pub fn windowing_bits() -> WindowingBits {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("comik", 1280, 720)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();

    WindowingBits {
        sdl_context,
        canvas,
        texture_creator,
    }
}

pub struct App<'a> {
    sdl_context: &'a mut sdl2::Sdl,
    canvas: &'a mut sdl2::render::Canvas<sdl2::video::Window>,
    collection: Option<Arc<dyn CollectionProvider>>,
    comic_index: usize,
    texture_manager: TextureManager<'a, WindowContext>,
    page_index: usize,
}

impl<'a> App<'a> {
    pub fn new(windowing_bits: &'a mut WindowingBits) -> Self {
        let WindowingBits {
            sdl_context,
            canvas,
            texture_creator,
        } = windowing_bits;

        let texture_manager = TextureManager::new(texture_creator);

        Self {
            sdl_context,
            canvas,
            collection: None,
            comic_index: 0,
            page_index: 0,
            texture_manager,
        }
    }

    fn reset_state(&mut self) {
        self.comic_index = 0;
        self.page_index = 0;
        self.collection = None;
    }

    fn load_collection(&mut self, paths: Vec<PathBuf>) -> Result<()> {
        let collection = FileSystemCollectionProvider::new("".to_string(), paths).unwrap();

        self.collection = Some(Arc::new(collection));

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RGB(20, 20, 20));
        self.canvas.clear();
        self.canvas.present();

        let mut event_pump = self.sdl_context.event_pump()?;

        'running: loop {
            let mut dropped_paths: Vec<PathBuf> = vec![];

            for event in event_pump.poll_iter() {
                match event {
                    Event::DropFile { filename, .. } => {
                        let path = PathBuf::from_str(&filename).unwrap();
                        dropped_paths.push(path);
                    }
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            if dropped_paths.len() > 0 {
                self.reset_state();
                self.load_collection(dropped_paths.clone()).unwrap();
                dropped_paths.clear();
            }

            self.canvas.clear();

            if let Some(collection) = &mut self.collection {
                let mut comic = collection.get_comic(self.comic_index).unwrap();
                let page = comic.get_page(self.page_index).unwrap();
                let file_name = &page.get_file_name().unwrap();

                let has_texture = self.texture_manager.has(&file_name);

                if has_texture {
                    let texture = self.texture_manager.get(&file_name).unwrap();
                    self.canvas.copy(&texture, None, None).unwrap();
                } else {
                    let comic_image = page.get_image();
                    let texture = self.texture_manager.load(&file_name, comic_image).unwrap();

                    self.canvas.copy(&texture, None, None).unwrap();
                }
            }

            self.canvas.present();
        }

        Ok(())
    }
}
