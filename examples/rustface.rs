extern crate image;
extern crate rustface;
extern crate videostream;

fn main() {
    let file = std::env::args().nth(1).unwrap();
    let mut stream = videostream::VideoStream::new(&file).unwrap();

    let mut detector =
        rustface::create_detector("examples/assets/seeta_fd_frontal_v1.0.bin").unwrap();
    detector.set_min_face_size(20);
    detector.set_score_thresh(2.0);
    detector.set_pyramid_scale_factor(0.8);
    detector.set_slide_window_step(4, 4);

    for (i, mut frame) in stream.iter().enumerate() {
        println!("{}", i);

        let luma = frame.as_luma().unwrap();
        let width = luma.width();
        let height = luma.height();
        let mut image = rustface::ImageData::new(luma.as_ptr(), width, height);

        let colour = image::Rgb([255, 0, 0]);

        let faces = detector.detect(&mut image);

        if !faces.is_empty() {
            let mut rgb = frame.as_rgb().unwrap();

            for face in faces {
                let bbox = face.bbox();
                let left = bbox.x().max(0) as u32;
                let right = (left + bbox.width()).min(rgb.width() - 1);
                let top = bbox.y().max(0) as u32;
                let bottom = (top + bbox.height()).min(rgb.height() - 1);

                for x in left..right {
                    rgb.put_pixel(x, top, colour);
                    rgb.put_pixel(x, bottom, colour);
                }

                for y in top..bottom {
                    rgb.put_pixel(left, y, colour);
                    rgb.put_pixel(right, y, colour);
                }
            }

            rgb.save(&format!("{}.png", i)).unwrap();
        }
    }
}
