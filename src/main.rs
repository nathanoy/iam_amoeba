use image::io::Reader as ImageReader;
use image::{DynamicImage, Rgb, RgbImage, Rgba};
use std::error::Error;
use std::iter::repeat;
use std::sync::atomic;
use std::time::Instant;

mod util;
use crate::util::{average, color_distance, show_img, Point};

const CROSS_COLOR: [u8; 3] = [255, 50, 50];
const BACKGROUND: [u8; 3] = [30, 30, 30];
const MIN_HOLE_PERCENT_SIZE: f32 = 0.01;

#[derive(Debug, Clone, Copy)]
struct Pixel {
    red: u8,
    green: u8,
    blue: u8,
    is_part: bool,
    scan_id: usize,
}

impl Pixel {
    fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            is_part: false,
            scan_id: 0,
        }
    }

    fn as_rgb(&self) -> Rgb<u8> {
        Rgb([self.red, self.green, self.blue])
    }
}

type Img = Image<Pixel>;

struct Image<P> {
    data: Vec<P>,
    height: usize,
    width: usize,
}

impl<P> Image<P>
where
    P: Copy,
{
    // fn new(width: usize, height: usize, val: P) -> Self {
    //     Image {
    //         data: vec![val; width * height],
    //         width,
    //         height,
    //     }
    // }

    fn enumerate_pixels(&self) -> EnumeratePx<'_, P> {
        EnumeratePx { idx: 0, img: self }
    }

    fn pixels(&self) -> Pixels<'_, P> {
        Pixels { idx: 0, img: self }
    }

    fn enumerate(&self) -> EnumerateIdx {
        EnumerateIdx {
            idx: 0,
            width: self.width,
            height: self.height,
        }
    }

    // fn iter_adjacent_idx(&self) -> impl Iterator<Item = (usize, usize)> {}
    fn at(&self, x: usize, y: usize) -> &P {
        &self.data[y * self.width + x]
    }

    fn at_mut(&mut self, x: usize, y: usize) -> &mut P {
        &mut self.data[y * self.width + x]
    }

    // fn at_checked(&self, x: usize, y: usize) -> Option<&P> {
    //     self.data.get(y * self.width + x)
    // }

    fn at_mut_checked(&mut self, x: usize, y: usize) -> Option<&mut P> {
        self.data.get_mut(y * self.width + x)
    }
}

impl Image<Pixel> {
    fn from_rbg(image: &RgbImage) -> Self {
        let (width, height) = (image.width() as usize, image.height() as usize);
        let data = image
            .pixels()
            .map(|&Rgb([r, g, b])| Pixel::new(r, g, b))
            .collect::<Vec<_>>();

        assert_eq!(data.len(), width * height);

        Image {
            data,
            width,
            height,
        }
    }

    // fn as_rbg(&self) -> RgbImage {
    //     let mut img = RgbImage::new(self.width as u32, self.height as u32);
    //     self.enumerate_pixels().for_each(
    //         |(
    //             x,
    //             y,
    //             &Pixel {
    //                 red, green, blue, ..
    //             },
    //         )| img.put_pixel(x as u32, y as u32, Rgb([red, green, blue])),
    //     );
    //     img
    // }

    // fn as_rgb_debug(&self) -> RgbImage {
    //     let mut img = RgbImage::new(self.width as u32, self.height as u32);
    //     self.enumerate_pixels().for_each(|(x, y, &px)| {
    //         img.put_pixel(
    //             x as u32,
    //             y as u32,
    //             Rgb(match px {
    //                 p if p.scan_id > 9 && p.is_part => [0, 255, 0], // green: filled hole
    //                 p if p.scan_id > 9 && !p.is_part => [255, 255, 0], // gelb: hintergrund
    //                 p if p.is_part => [255, 0, 0],
    //                 p if p.scan_id != 0 => [0, 255, 255],
    //                 Pixel {
    //                     red, green, blue, ..
    //                 } => [red, green, blue],
    //             }),
    //         )
    //     });
    //     img
    // }

    fn as_rgb_output(&self) -> RgbImage {
        let mut img = RgbImage::new(self.width as u32, self.height as u32);
        self.enumerate_pixels().for_each(|(x, y, &px)| {
            fn scale(base: f32, from: u8) -> u8 {
                (base * 0.2) as u8 + (from as f32 * 0.8) as u8
            }
            fn fade(p: Pixel) -> [u8; 3] {
                [scale(0.0, p.red), scale(0.0, p.red), scale(255.0, p.red)]
            }
            img.put_pixel(
                x as u32,
                y as u32,
                Rgb(match px {
                    p if p.is_part => fade(p),
                    _ => BACKGROUND,
                }),
            )
        });
        img
    }
}

struct EnumeratePx<'a, P> {
    idx: usize,
    img: &'a Image<P>,
}

impl<'a, P> Iterator for EnumeratePx<'a, P> {
    type Item = (usize, usize, &'a P);

    fn next(&mut self) -> Option<Self::Item> {
        // Should just stop when there are no more pixels, use with caution
        let ret = self.img.data.get(self.idx)?;
        let ret = (self.idx % self.img.width, self.idx / self.img.width, ret);
        self.idx += 1;
        Some(ret)
    }
}

struct EnumerateIdx {
    idx: usize,
    width: usize,
    height: usize,
}

impl Iterator for EnumerateIdx {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.idx += 1;
        if self.idx >= self.height * self.width {
            return None;
        }
        let (x, y) = (self.idx % self.width, self.idx / self.width);
        Some((x, y))
    }
}

fn get_sorrunding_indecies(x0: usize, y0: usize) -> impl Iterator<Item = (usize, usize)> {
    fn cmo(a: usize) -> usize {
        if a == 0 {
            0
        } else {
            a - 1
        }
    }
    (cmo(y0)..=(y0 + 1))
        .flat_map(move |yi| (cmo(x0)..=(x0 + 1)).zip(repeat(yi)))
        .filter(move |&a| a != (x0, y0))
}

#[test]
fn test_get_idx() {
    assert_eq!(
        get_sorrunding_indecies(0, 0).collect::<Vec<_>>(),
        [(1, 0), (0, 1), (1, 1)]
    );
    assert_eq!(
        get_sorrunding_indecies(1, 1).collect::<Vec<_>>(),
        [
            (0, 0),
            (1, 0),
            (2, 0),
            (0, 1),
            (2, 1),
            (0, 2),
            (1, 2),
            (2, 2),
        ]
    );
}
struct Pixels<'a, P> {
    idx: usize,
    img: &'a Image<P>,
}

impl<'a, P> Iterator for Pixels<'a, P> {
    type Item = &'a P;

    fn next(&mut self) -> Option<Self::Item> {
        // Should just stop when there are no more pixels, use with caution
        let ret = self.img.data.get(self.idx);
        self.idx += 1;
        ret
    }
}

#[inline(always)]
fn get_scan_id() -> usize {
    static SCAN_ID: atomic::AtomicUsize = atomic::AtomicUsize::new(10);
    SCAN_ID.fetch_add(1, atomic::Ordering::SeqCst)
}

fn for_all_of_kind_adjacent<F: Fn(&mut Pixel)>(
    img: &mut Img,
    x0: usize,
    y0: usize,
    fun: F,
    scan_id: usize,
) -> usize {
    let p0 = img.at_mut(x0, y0);
    let kind = p0.is_part;
    // println!("{x0}:{y0} p0: {p0:?}");
    let mut nodes = vec![(x0, y0)];

    let mut count = 0;
    while let Some((xi, yi)) = nodes.pop() {
        for (x, y) in get_sorrunding_indecies(xi, yi) {
            if let Some(px) = img.at_mut_checked(x, y) {
                if px.is_part == kind && px.scan_id != scan_id {
                    fun(px);
                    px.scan_id = scan_id;
                    nodes.push((x, y));
                    count += 1;
                }
            }
        }
    }
    count
}

fn count_adjacent(img: &mut Img, x0: usize, y0: usize, scan_id: usize) -> usize {
    for_all_of_kind_adjacent(img, x0, y0, |_| {}, scan_id)
}

fn close_hole(img: &mut Img, x0: usize, y0: usize, scan_id: usize) -> usize {
    let kind = !img.at_mut(x0, y0).is_part; // we want to invert the hole
                                            // println!(
                                            //     "kind = {kind} {x0}:{y0} {}",
                                            //     for_all_of_kind_adjacent(img, x0, y0, |_| {}, 1)
                                            // );
    for_all_of_kind_adjacent(img, x0, y0, |px| px.is_part = kind, scan_id)
}

fn detect_shape(img: &mut RgbImage) -> Img {
    // TODO: user selectable pixel, currently just the first one
    let p0 = *img
        .pixels()
        .next()
        .expect("at least one pixel in the src image");

    // To keep track of what pixels belong to the Object. All pixels with the color IS_PART_COLOR are part
    let mut img = Image::from_rbg(img);

    // Color key
    for (x, y) in img.enumerate() {
        let px = img.at_mut(x, y);
        px.is_part = color_distance(p0, px.as_rgb()) >= 35.0;
    }
    // let provisional_shape_pixel_count = img.pixels().filter(|x| x.is_part).count();
    let image_size = (img.height * img.width) as f32;
    // Fix small holes
    for (x, y) in img.enumerate() {
        if img.at(x, y).scan_id != 0 {
            continue;
        }
        let count = count_adjacent(&mut img, x, y, get_scan_id());
        if count != 0 {
            let percent = count as f32 / image_size;
            println!("P: {percent:.6}");
            if percent < 0.001 {
                let _filled = close_hole(&mut img, x, y, get_scan_id());
            }
        }
    }

    img
}

fn main() -> Result<(), Box<dyn Error>> {
    let filename = std::env::args()
        .nth(1)
        .expect("exactly one command line argument: the input path");

    let mut t0 = Instant::now();
    let t00 = t0;
    let mut img = ImageReader::open(filename)?.decode()?.to_rgb8();
    println!("Read Image in: {:.2}s", t0.elapsed().as_secs_f32());

    let factor = 800.0 / img.width() as f32;

    t0 = Instant::now();
    img = image::imageops::resize(
        &img,
        800,
        (img.height() as f32 * factor) as u32,
        image::imageops::FilterType::Nearest,
    );
    println!("Resized in: {:.2}s", t0.elapsed().as_secs_f32());

    // Start
    t0 = Instant::now();

    let img = detect_shape(&mut img);

    // Collect all points of the shape, in order to average them
    let pixels_of_shape: Vec<_> = img
        .enumerate_pixels()
        .filter_map(
            |(x, y, px)| {
                if px.is_part {
                    Some(Point(x, y))
                } else {
                    None
                }
            },
        )
        .collect();

    let Point(cogx, cogy) = average(&pixels_of_shape[..]).ok_or("No Shape detected".to_owned())?;
    let (cogx, cogy) = (cogx as u32, cogy as u32);

    let img = img.as_rgb_output();

    let mut overlay =
        image::load_from_memory(include_bytes!("..\\assets\\crosshair.png"))?.to_rgba8();
    overlay.pixels_mut().for_each(|px| {
        if px[3] != 0 {
            *px = Rgba([CROSS_COLOR[0], CROSS_COLOR[1], CROSS_COLOR[2], 255])
        }
    });

    let nh = img.height() as f32 / 20.0;
    let nw = (overlay.width() as f32 * (nh / overlay.height() as f32)) as u32;
    let nh = nh as u32;
    let overlay = image::imageops::resize(&overlay, nh, nw, image::imageops::FilterType::Nearest);

    let mut img = DynamicImage::ImageRgb8(img).to_rgba8();
    image::imageops::overlay(
        &mut img,
        &overlay,
        (cogx - nw / 2) as i64,
        (cogy - nh / 2) as i64,
    );
    println!("Proccessed in: {:.2}s", t0.elapsed().as_secs_f32());

    t0 = Instant::now();
    let img = DynamicImage::ImageRgba8(img);
    let path = show_img(&img)?;
    println!("Saved and displayed in: {:.2}s", t0.elapsed().as_secs_f32());
    println!("Saved to: '{path}'");
    println!("Total: {:.2}", t00.elapsed().as_secs_f32());
    Ok(())
}
