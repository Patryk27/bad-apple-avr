use crate::*;
use ::image::io::Reader as ImageReader;
pub use std::path::Path;

pub struct Source {
    images: Vec<RgbImage>,
}

impl Source {
    pub fn from_dir(path: impl AsRef<Path>) -> Result<Self> {
        let pattern = path.as_ref().join("*.*");
        let paths = glob::glob(&*pattern.to_string_lossy()).context("Couldn't find frames")?;

        let images = paths.into_iter().map(|frame| {
            let path = frame.context("Couldn't find frame")?;

            let image = ImageReader::open(&path)
                .with_context(|| format!("Couldn't open frame: {}", path.display()))?
                .decode()
                .with_context(|| format!("Couldn't decode frame: {}", path.display()))?;

            Ok(image.to_rgb8())
        });

        Ok(Self {
            images: images.collect::<Result<_>>()?,
        })
    }

    pub fn images(&self) -> impl Iterator<Item = &RgbImage> {
        self.images.iter()
    }
}
