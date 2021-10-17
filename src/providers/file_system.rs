use std::{cmp::Ordering, collections::HashMap, fs::File, io::Cursor, path::PathBuf, sync::Arc};

use anyhow::Result;
use compress_tools::{list_archive_files, uncompress_archive_file};
use image::io::Reader as ImageReader;

use super::{CollectionProvider, ComicProvider, PageProvider, ProviderError};

pub struct FileSystemCollectionProvider {
    collection_name: String,
    paths: Vec<PathBuf>,
}

impl FileSystemCollectionProvider {
    pub fn new(collection_name: String, paths: Vec<PathBuf>) -> Result<Self> {
        Ok(Self {
            collection_name,
            paths,
        })
    }
}

impl CollectionProvider for FileSystemCollectionProvider {
    fn get_name(&self) -> String {
        self.collection_name.clone()
    }

    fn get_comic(&self, index: usize) -> Option<Box<dyn ComicProvider>> {
        let path = self.paths.get(index).unwrap();

        let comic = FileSystemComicProvider::new(path.to_path_buf()).unwrap();

        Some(Box::new(comic))
    }

    fn get_size(&self) -> usize {
        self.paths.len()
    }
}

pub struct FileSystemComicProvider {
    title: String,
    path: PathBuf,
    archive: File,
    file_list: Vec<String>,
    cache: HashMap<usize, Vec<u8>>,
}

impl FileSystemComicProvider {
    fn new(path: PathBuf) -> Result<Self, ProviderError> {
        match path.extension() {
            Some(ext) if ext == "zip" || ext == "cbz" || ext == "rar" || ext == "cbr" => {
                let title = path.file_name().unwrap().to_str().unwrap().to_string();

                let archive = File::open(path.clone()).unwrap();

                let mut file_list = list_archive_files(&archive).unwrap();
                file_list.sort();

                Ok(Self {
                    title,
                    path,
                    archive,
                    file_list,
                    cache: HashMap::new(),
                })
            }
            _ => Err(ProviderError::InvalidArchiveType),
        }
    }
}

impl ComicProvider for FileSystemComicProvider {
    fn open(&self) -> Result<()> {
        Ok(())
    }

    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn get_page(&mut self, index: usize) -> Option<Box<dyn PageProvider>> {
        let file_name = self.file_list.get(index).unwrap();

        let img = match self.cache.contains_key(&index) {
            true => {
                let buffer = self.cache.get(&index).unwrap().clone();

                ImageReader::new(Cursor::new(buffer))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap()
            }

            false => {
                let mut buffer = Vec::new();
                uncompress_archive_file(&self.archive, &mut buffer, &file_name).unwrap();

                let image = ImageReader::new(Cursor::new(buffer.clone()))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap();

                self.cache.insert(index, buffer);

                image
            }
        };

        let page_provider = FileSystemPageProvider {
            file_name: file_name.to_string(),
            image_buffer: img,
        };

        Some(Box::new(page_provider))
    }

    fn get_length(&self) -> usize {
        self.file_list.len()
    }
}

#[derive(Debug, Clone, Eq)]
pub struct FileSystemPageProvider {
    file_name: String,
    image_buffer: image::DynamicImage,
}

impl PageProvider for FileSystemPageProvider {
    fn get_image(&self) -> image::DynamicImage {
        self.image_buffer.clone()
    }

    fn get_file_name(&self) -> Result<String> {
        Ok(self.file_name.clone())
    }
}

impl Ord for FileSystemPageProvider {
    fn cmp(&self, other: &Self) -> Ordering {
        self.file_name.cmp(&other.file_name)
    }
}

impl PartialOrd for FileSystemPageProvider {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FileSystemPageProvider {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name
    }
}
