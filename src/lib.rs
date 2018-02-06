extern crate image;
extern crate ffmpeg;

use std::path::Path;

use ffmpeg::util::format::pixel::Pixel as PixelFormat;
use ffmpeg::util::frame::video::Video as InnerFrame;
use ffmpeg::codec::decoder::video::Video as Decoder;
use ffmpeg::media::Type;
use ffmpeg::format::context::Input;
use ffmpeg::format::context::input::PacketIter;
use ffmpeg::software::converter;
use image::{RgbImage, RgbaImage, GrayImage};

#[derive(Debug)]
pub enum Error {
    Static(&'static str),
    Ffmpeg(ffmpeg::Error)
}

impl From<ffmpeg::Error> for Error {
    fn from(error: ffmpeg::Error) -> Self {
        Error::Ffmpeg(error)
    }
}

impl From<&'static str> for Error {
    fn from(error: &'static str) -> Self {
        Error::Static(error)
    }
}

pub struct VideoStream {
    input: Input,
    stream: usize,
    decoder: Decoder
}

impl VideoStream {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        ffmpeg::init()?;

        let input = ffmpeg::format::input(&path)?;

        let (stream, decoder) = {
            let stream = input.streams().best(Type::Video).ok_or("Failed to get stream")?;
            (stream.index(), stream.codec().decoder().video()?)
        };

        Ok(Self {
            input, stream, decoder
        })
    }

    pub fn frames(&mut self) -> Frames {
        Frames {
            packets: self.input.packets(),
            stream: self.stream,
            decoder: &mut self.decoder,
        }
    }
}

pub struct Frames<'a> {
    decoder: &'a mut Decoder,
    stream: usize,
    packets: PacketIter<'a>,
}

impl<'a> Iterator for Frames<'a> {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        match self.packets.next() {
            Some((stream, packet)) => if stream.index() == self.stream {     
                let mut output = InnerFrame::empty();

                match self.decoder.decode(&packet, &mut output) {
                    Ok(_) => if output.format() != PixelFormat::None {
                        Some(Frame::new(output))
                    } else {
                        self.next()
                    },
                    Err(error) => {
                        eprintln!("{}", error);
                        None
                    }
                }
            } else {
                self.next()
            },
            None => None
        }
    }
}

pub struct Frame {
    inner: InnerFrame
}

impl Frame {
    fn new(inner: InnerFrame) -> Self {
        Self {
            inner
        }
    }

    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    pub fn as_rgba(&self) -> Result<RgbaImage, Error> {
        let vec = self.as_vec(4, PixelFormat::RGBA)?;
        RgbaImage::from_raw(self.width(), self.height(), vec).ok_or_else(|| "Failed to convert image".into())
    }
 
    pub fn as_rgb(&self) -> Result<RgbImage, Error> {
        let vec = self.as_vec(3, PixelFormat::RGB24)?;
        RgbImage::from_raw(self.width(), self.height(), vec).ok_or_else(|| "Failed to convert image".into())
    }

    pub fn as_luma(&self) -> Result<GrayImage, Error> {
        let vec = self.as_vec(1, PixelFormat::GRAY8)?;
        GrayImage::from_raw(self.width(), self.height(), vec).ok_or_else(|| "Failed to convert image".into())
    }

    fn convert(&self, format: PixelFormat) -> Result<InnerFrame, Error> {
        let mut output = InnerFrame::empty();

        converter((self.width(), self.height()), self.inner.format(), format)?
            .run(&self.inner, &mut output)
            .map_err(Error::Ffmpeg)
            .map(|_| output)
    }

    pub fn as_vec(&self, channels: u32, format: PixelFormat) -> Result<Vec<u8>, Error> {
        let output = self.convert(format)?;

        let index = 0;
        let stride = output.stride(index);
        let width = (output.width() * channels) as usize;

        // If the stride and width are equal, just convert to a vec
        if stride == width {
            Ok(output.data(index).to_vec())
        // If they aren't (because the data has some garbage at the end of each line), skip over the garbage
        } else {
            let mut offset = 0;
            let mut vec = Vec::with_capacity((self.width() * self.height() * channels) as usize);
            let data = output.data(index);

            while offset < data.len() {
                vec.extend_from_slice(&data[offset .. offset + width]);
                offset += stride;
            }

            Ok(vec)
        }
    }
}

#[test]
fn remote_url() {
    let url = "https://upload.wikimedia.org/wikipedia/commons/thumb/9/98/Aldrin_Apollo_11_original.jpg/596px-Aldrin_Apollo_11_original.jpg";
    let frame = VideoStream::new(url).unwrap().frames().next().unwrap();

    frame.as_rgb().unwrap();
    frame.as_rgba().unwrap();
    frame.as_luma().unwrap();
}