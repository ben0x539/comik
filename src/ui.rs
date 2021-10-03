use std::path::PathBuf;
use std::sync::Arc;

use epi::egui;
use epi::egui::Color32;
use epi::egui::CtxRef;
use epi::egui::TextureId;
use image::Pixel;

use crate::providers::file_system::FileSystemCollectionProvider;
use crate::providers::CollectionProvider;

#[derive(Default)]
pub struct Ui {
    collection: Option<Arc<dyn CollectionProvider>>,
    current_comic_index: usize,
    current_page_index: usize,
    current_image: Option<image::DynamicImage>,
    current_texture: Option<TextureId>,
    current_image_size: egui::Vec2,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn reset_state(&mut self) {
        self.current_comic_index = 0;
        self.current_page_index = 0;
        self.current_image = None;
        self.current_image_size = egui::Vec2::new(0.0, 0.0);
        self.current_texture = None;
    }

    pub fn tick(&mut self, ctx: &CtxRef, frame: &mut epi::Frame<'_>) {
        if !ctx.input().raw.dropped_files.is_empty() {
            let dropped_files = ctx
                .input()
                .raw
                .dropped_files
                .clone()
                .iter()
                .map(|file| file.path.as_ref().unwrap().clone())
                .collect::<Vec<PathBuf>>();

            dbg!(&dropped_files);

            let collection =
                FileSystemCollectionProvider::new("collection name".to_string(), dropped_files)
                    .unwrap();

            self.reset_state();
            self.collection = Some(Arc::new(collection));
        }

        if let Some(collection) = self.collection.clone() {
            if self.current_image.is_none() {
                if let Some(comic) = *collection.get_comic(self.current_comic_index) {
                    if let Some(image_provider) = comic.get_page(self.current_page_index) {
                        let comic_image = image_provider.get_image();
                        self.current_image = Some(comic_image);
                    }
                }
            }
        }

        if self.current_texture.is_none() {
            if let Some((texture, size)) = self.render_current_page(frame) {
                self.current_texture = Some(texture);
                self.current_image_size = size;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            ui.centered_and_justified(|ui| {
                if let Some(texture_id) = self.current_texture {
                    ui.image(texture_id, self.current_image_size);
                }
            });
        });
    }

    fn render_current_page(&self, frame: &mut epi::Frame<'_>) -> Option<(TextureId, egui::Vec2)> {
        if let Some(comic_image) = &self.current_image {
            let image = comic_image.to_rgb8();
            let size = (image.width() as usize, image.height() as usize);

            // TODO! apply zoom and panning
            let pixels = image
                .pixels()
                .into_iter()
                .map(|pixel| {
                    let [r, g, b, a] = pixel.to_rgba().0;
                    Color32::from_rgba_unmultiplied(r, g, b, a)
                })
                .collect::<Vec<Color32>>();

            let texture = frame
                .tex_allocator()
                .alloc_srgba_premultiplied(size, &pixels);

            let size = egui::Vec2::new(size.0 as f32, size.1 as f32);

            return Some((texture, size));
        } else {
            None
        }
    }
}
