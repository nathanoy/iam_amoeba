use image::io::Reader as ImageReader;
use image::{DynamicImage, Rgb, RgbImage};
use std::error::Error;
use std::time::Instant;

mod util;
use crate::util::{average, color_distance, show_img, Point};

const IS_PART_COLOR: Rgb<u8> = Rgb([255, 255, 255]);
const CROSS_COLOR: Rgb<u8> = Rgb([255, 255, 255]);
const IS_BACKGROUND_SCANNED: Rgb<u8> = Rgb([255, 0, 0]);
const BACKGROUND: Rgb<u8> = Rgb([0, 0, 0]);

fn fill_adjacent(img: &mut RgbImage, x0: u32, y0: u32) -> u32 {
    let mut nodes = vec![];
    let px = *img.get_pixel_mut(x0, y0);
    if px == BACKGROUND {
        nodes.push((x0, y0))
    }

    /// checked minus one
    fn cm(a: u32) -> u32 {
        if a == 0 {
            0
        } else {
            a - 1
        }
    }

    let mut count = 0;
    while let Some((xi, yi)) = nodes.pop() {
        for y in cm(yi)..=(yi + 1) {
            for x in cm(xi)..=(xi + 1) {
                if let Some(px) = img.get_pixel_mut_checked(x, y) {
                    if *px == BACKGROUND {
                        *px = IS_BACKGROUND_SCANNED;
                        nodes.push((x, y));
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

fn add_adjacent_to_shape(img: &mut RgbImage, x0: u32, y0: u32) {
    let mut nodes = vec![];
    let px = *img.get_pixel_mut(x0, y0);
    if px == IS_BACKGROUND_SCANNED {
        nodes.push((x0, y0))
    }

    /// checked minus one
    fn cm(a: u32) -> u32 {
        if a == 0 {
            0
        } else {
            a - 1
        }
    }

    while let Some((xi, yi)) = nodes.pop() {
        for y in cm(yi)..=(yi + 1) {
            for x in cm(xi)..=(xi + 1) {
                if let Some(px) = img.get_pixel_mut_checked(x, y) {
                    if *px == IS_BACKGROUND_SCANNED {
                        *px = IS_PART_COLOR;
                        nodes.push((x, y));
                    }
                }
            }
        }
    }
}

/// Finds the Center of Gravity of a differently colored object in an Image.
/// returns None when no object is detected
fn find_center_of_gravity(img: &mut RgbImage) -> Option<Point<u32>> {
    // TODO: user selectable pixel, currently just the first one
    let p0 = *img
        .pixels()
        .next()
        .expect("at least one pixel in the src image");

    // To keep track of what pixels belong to the Object. All pixels with the color IS_PART_COLOR are part
    let mut piece_map = RgbImage::new(img.width(), img.height());
    piece_map.pixels_mut().for_each(|x| *x = IS_PART_COLOR);

    // Color key
    for (x, y, px) in img.enumerate_pixels_mut() {
        if color_distance(p0, *px) < 35.0 {
            // *px = BACKGROUND_COL;
            *piece_map.get_pixel_mut(x, y) = BACKGROUND;
        }
    }

    let provisional_shape_pixel_count = piece_map.pixels().filter(|&&x| x != BACKGROUND).count();

    // Fix small holes
    const MIN_HOLE_PERCENT_SIZE: f32 = 0.01;
    for y in 0..piece_map.height() {
        for x in 0..piece_map.width() {
            if *piece_map.get_pixel(x, y) == BACKGROUND {
                let filled = fill_adjacent(&mut piece_map, x, y);
                let p = filled as f32 / provisional_shape_pixel_count as f32;
                if p < MIN_HOLE_PERCENT_SIZE {
                    add_adjacent_to_shape(&mut piece_map, x, y);
                }
            }
        }
    }

    // Darken the background
    // piece_map.enumerate_pixels().for_each(|(x, y, px)| {
    //     if *px != IS_PART_COLOR {
    //         let px = img.get_pixel_mut(x, y);
    //         let Rgb([r, g, b]) = *px;
    //         const DD: u8 = 3; // Darkness divident
    //         *px = Rgb([r / DD, g / DD, b / DD]);
    //     }
    // });
    piece_map.enumerate_pixels().for_each(|(x, y, px)| {
        if *px != IS_PART_COLOR {
            let px = img.get_pixel_mut(x, y);
            *px = Rgb([30, 30, 40]);
        }
    });

    // Collect all points of the shape, in order to average them
    let pixels_of_shape: Vec<_> = piece_map
        .enumerate_pixels()
        .filter_map(|(x, y, px)| {
            if *px == IS_PART_COLOR {
                Some(Point(x, y))
            } else {
                None
            }
        })
        .collect();

    average(&pixels_of_shape[..])
}

fn main() -> Result<(), Box<dyn Error>> {
    let filename = std::env::args()
        .nth(1)
        .expect("exactly one command line argument: the input path");

    let mut t0 = Instant::now();
    let mut img = ImageReader::open(filename)?.decode()?.to_rgb8();
    println!("Read Image in: {:.2}s", t0.elapsed().as_secs_f32());

    t0 = Instant::now();
    img = image::imageops::resize(&img, 800, 600, image::imageops::FilterType::Triangle);
    println!("Resized in: {:.2}s", t0.elapsed().as_secs_f32());

    // Start
    t0 = Instant::now();

    let Point(cogx, cogy) =
        find_center_of_gravity(&mut img).ok_or("No Shape detected".to_owned())?;

    // Draw y
    for x in 0..img.width() {
        *img.get_pixel_mut(x, cogy + 1) = CROSS_COLOR;
        *img.get_pixel_mut(x, cogy) = CROSS_COLOR;
        *img.get_pixel_mut(x, cogy - 1) = CROSS_COLOR;
    }

    // Draw x
    for y in 0..img.height() {
        *img.get_pixel_mut(cogx + 1, y) = CROSS_COLOR;
        *img.get_pixel_mut(cogx, y) = CROSS_COLOR;
        *img.get_pixel_mut(cogx - 1, y) = CROSS_COLOR;
    }

    println!("Proccessed in: {:.2}s", t0.elapsed().as_secs_f32());

    t0 = Instant::now();
    show_img(&DynamicImage::ImageRgb8(img))?;
    println!("Saved and displayed in: {:.2}s", t0.elapsed().as_secs_f32());

    Ok(())
}
