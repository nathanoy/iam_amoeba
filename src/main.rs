use image::imageops::{resize, FilterType};
use image::io::Reader as ImageReader;
use image::{DynamicImage, Rgb};
use std::error::Error;
use std::iter::Sum;
use std::ops::{Add, Div};
use std::process::Command;

use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let t0 = Instant::now();
    let mut img = ImageReader::open("img.jpg")?.decode()?.to_rgb8();
    println!("Read in: {:.2}s", t0.elapsed().as_secs_f32());
    let t0 = Instant::now();
    let mut img = resize(&img, 800, 600, FilterType::Triangle);
    println!("Resized in: {:.2}s", t0.elapsed().as_secs_f32());

    let t0 = Instant::now();

    let p0 = *img.pixels().next().unwrap();

    const BACKGROUND_COL: Rgb<u8> = Rgb([40, 40, 50]);
    // Color key
    for px in img.pixels_mut() {
        let pi = *px;
        let d = distance(p0, pi);
        if d < 25.0 {
            *px = BACKGROUND_COL;
        }
    }

    let need_a_name: Vec<_> = img
        .enumerate_pixels()
        .filter_map(|(x, y, px)| {
            if *px != BACKGROUND_COL {
                Some(Point(x, y))
            } else {
                None
            }
        })
        .collect();
    let Point(cogx, cogy) = average(&need_a_name[..]).unwrap();

    // Draw y
    for x in 0..img.width() {
        let px = img.get_pixel_mut(x, cogy);
        *px = Rgb([255, 0, 0]);
    }

    // Draw x
    for y in 0..img.height() {
        let px = img.get_pixel_mut(cogx, y);
        *px = Rgb([255, 0, 0]);
    }

    show_img(&DynamicImage::ImageRgb8(img))?;

    println!("Work in: {:.2}s", t0.elapsed().as_secs_f32());

    Ok(())
}

#[derive(Debug)]
struct Point<T>(T, T);

impl<'a, T> Sum<&'a Self> for Point<T>
where
    T: Add<Output = T> + Default + Copy,
{
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        iter.fold(Self(Default::default(), Default::default()), |acc, b| {
            Self(acc.0 + b.0, acc.1 + b.1)
        })
    }
}

impl<T> Div<T> for Point<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Self;
    fn div(self, divisor: T) -> Self {
        Self(self.0 / divisor, self.1 / divisor)
    }
}

fn show_img(img: &DynamicImage) -> Result<(), Box<dyn Error>> {
    const PATH: &str = "tmp.png";
    img.save(PATH)?;
    Command::new("cmd").args(["/C", "start", PATH]).spawn()?;
    Ok(())
}

fn distance(c1: Rgb<u8>, c2: Rgb<u8>) -> f32 {
    // let Rgb([r0, g0, b0]) = c1;
    // let Rgb([r, g, b]) = c2;

    // // Pytagoras3d
    // (((r0 as i32 - r as i32) * (r0 as i32 - r as i32)
    //     + (g0 as i32 - g as i32) * (g0 as i32 - g as i32)
    //     + (b0 as i32 - b as i32) * (b0 as i32 - b as i32)) as f32)
    //     .sqrt()
    let Rgb(c1) = c1;
    let Rgb(c2) = c2;
    delta_e::DE2000::from_rgb(&c1, &c2)
}

/// Fails when itms is empty or when the length of itms overflows W!
fn average<'a, T, W>(itms: &'a [T]) -> Option<T>
where
    T: Sum<&'a T> + Div<W, Output = T>,
    W: TryFrom<usize>,
{
    if itms.is_empty() {
        return None;
    }
    Some(itms.iter().sum::<T>() / itms.len().try_into().ok()?)
}
