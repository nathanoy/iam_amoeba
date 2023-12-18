use std::error::Error;
use std::iter::Sum;
use std::ops::{Add, Div};

#[allow(unused_imports)]
use std::process::Command;

use image::{DynamicImage, Rgb};

#[derive(Debug)]
pub struct Point<T>(pub T, pub T);

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

/// Fails when itms is empty or when the length of itms overflows W!
pub fn average<'a, T, W>(itms: &'a [T]) -> Option<T>
where
    T: Sum<&'a T> + Div<W, Output = T>,
    W: TryFrom<usize>,
{
    if itms.is_empty() {
        return None;
    }
    Some(itms.iter().sum::<T>() / itms.len().try_into().ok()?)
}

pub fn show_img(img: &DynamicImage) -> Result<&str, Box<dyn Error>> {
    const PATH: &str = "out.png";

    img.save(PATH)?;
    // println!("Saved output to: {}", PATH);

    #[cfg(target_os = "windows")]
    Command::new("cmd").args(["/C", "start", PATH]).spawn()?;

    #[cfg(target_os = "macos")]
    Command::new("open").arg(PATH).spawn()?;

    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(PATH).spawn()?;

    Ok(PATH)
}

pub fn color_distance(Rgb(c1): Rgb<u8>, Rgb(c2): Rgb<u8>) -> f32 {
    // Alternative color distance function
    // let [r0, g0, b0] = c1;
    // let [r, g, b] = c2;

    // // Pytagoras3d
    // (((r0 as i32 - r as i32) * (r0 as i32 - r as i32)
    //     + (g0 as i32 - g as i32) * (g0 as i32 - g as i32)
    //     + (b0 as i32 - b as i32) * (b0 as i32 - b as i32)) as f32)
    //     .sqrt()

    delta_e::DE2000::from_rgb(&c1, &c2)
}
