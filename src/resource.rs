use std::{collections::HashMap, rc::Rc};

use image::{DynamicImage, EncodableLayout, GenericImageView};
use sdl2::render::{Texture, TextureAccess, TextureCreator};

pub trait ResourceLoader<'l, R> {
    fn load(&'l self, image: DynamicImage) -> Result<R, String>;
}

pub struct TextureManager<'l, T> {
    texture_creator: &'l TextureCreator<T>,
    cache: HashMap<String, Rc<Texture<'l>>>,
}

impl<'l, T> TextureManager<'l, T> {
    pub fn new(texture_creator: &'l TextureCreator<T>) -> Self {
        TextureManager {
            texture_creator,
            cache: HashMap::new(),
        }
    }

    pub fn load(&mut self, key: &str, img: DynamicImage) -> Result<Rc<Texture>, String> {
        self.cache.get(&key.to_string()).cloned().map_or_else(
            move || {
                let texture_data = self.texture_creator.load(img).unwrap();

                let texture = Rc::new(texture_data);

                self.cache.insert(key.to_string(), texture.clone());

                Ok(texture)
            },
            Ok,
        )
    }

    pub fn get(&mut self, key: &str) -> Option<Rc<Texture<'_>>> {
        self.cache.get(&key.to_string()).cloned()
    }

    pub fn has(&self, key: &str) -> bool {
        self.cache.contains_key(&key.to_string())
    }
}

impl<'l, T> ResourceLoader<'l, Texture<'l>> for TextureCreator<T> {
    fn load(&'l self, img: DynamicImage) -> Result<Texture, String> {
        println!("LOADED A TEXTURE");

        let img_rgb8 = img.to_rgb8();
        let img_raw = img_rgb8.as_bytes();

        let mut texture = self
            .create_texture(None, TextureAccess::Static, img.width(), img.height())
            .map_err(|e| e.to_string())?;

        texture.update(None, &img_raw, 3).unwrap();

        Ok(texture)
    }
}
