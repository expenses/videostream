extern crate videostream;
extern crate image;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mut stream = videostream::VideoStream::new(&path).unwrap();
    let mut colours = Vec::new();

    for (i, mut frame) in stream.iter().enumerate() {
        println!("{}", i);

        let image = frame.as_rgb().unwrap();

        let (mut r, mut g, mut b) = image.chunks(3)
            .fold((0, 0, 0), |mut total, pixel| {
                total.0 += u32::from(pixel[0]);
                total.1 += u32::from(pixel[1]);
                total.2 += u32::from(pixel[2]);
                total
            });

        let pixels = image.width() * image.height();
        colours.push((r / pixels) as u8);
        colours.push((g / pixels) as u8);
        colours.push((b / pixels) as u8);
    }

    let image = image::RgbImage::from_raw(colours.len() as u32 / 3, 1, colours).unwrap();
    image::imageops::resize(&image, 4000, 1000, image::FilterType::Nearest).save("barcode.png").unwrap();
}