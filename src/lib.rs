extern crate color_quant;
extern crate delta_e;
extern crate image;
extern crate itertools;
extern crate lab;
#[macro_use]
extern crate quick_error;

use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use color_quant::NeuQuant;
use delta_e::DE2000;
use image::FilterType::Gaussian;
use image::{imageops, ImageBuffer, GenericImage, DynamicImage, Rgba, Rgb, Pixel};
use itertools::Itertools;
use lab::Lab;

static MAX_SAMPLE_COUNT: u32 = 1000;
static NQ_SAMPLE_FACTION: i32 = 10;
static NQ_PALETTE_SIZE: usize = 256;
static MIN_BLACK: u8 = 8;
static MAX_WHITE: u8 = 247;
static MIN_DISTANCE_FOR_UNIQUENESS: f32 = 10.0;

quick_error! {
    #[derive(Debug)]
    pub enum DistilError {
        Io(path: String, err: image::ImageError) {
            display("Distil failed to parse the passed image: {}", err)
        }
    }
}

pub struct Distil;

impl Distil {
    pub fn from_path_str(path_str: &str, palette_size: u8) -> Result<(), DistilError> {
        match image::open(&Path::new(&path_str)) {
            Ok(img) => {
                Distil::new(img, palette_size);
                Ok(())
            }
            Err(err) => Err(DistilError::Io(path_str.to_string(), err)),
        }
    }

    pub fn new(img: DynamicImage, palette_size: u8) {
        let scaled_img = scale_img(img);
        let quantized_img = quantize(scaled_img);

        let color_count = count_colors_as_lab(quantized_img);
        let palette = remove_similar_colors(color_count);

        output_palette_as_img(palette, palette_size);
    }
}

/// Proportionally scales the passed image to a size where its total number of
/// pixels does not exceed the value of `MAX_SAMPLE_COUNT`.
fn scale_img(mut img: DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();

    if width * height > MAX_SAMPLE_COUNT {
        let (width, height) = (width as f32, height as f32);
        let ratio = width / height;

        let scaled_width = (ratio * (MAX_SAMPLE_COUNT as f32)).sqrt() as u32;

        img = img.resize(scaled_width, height as u32, Gaussian);
    }

    img
}

/// Uses the NeuQuant quantization algorithm to reduce the passed image to a
/// palette of `NQ_PALETTE_SIZE` colors.
///
/// Note: NeuQuant is designed to produce images with between 64 and 256
/// colors. As such, `NQ_PALETTE_SIZE`'s value should be kept within those
/// bounds.
fn quantize(img: DynamicImage) -> Vec<Rgb<u8>> {
    let pixels = get_pixels(img);
    let quantized = NeuQuant::new(NQ_SAMPLE_FACTION, NQ_PALETTE_SIZE, &pixels);

    quantized.color_map_rgb()
        .iter()
        .chunks(3)
        .into_iter()
        .map(|rgb_iter| {
            let rgb_slice: Vec<u8> = rgb_iter.cloned().collect();
            Rgb::from_slice(&rgb_slice).clone()
        })
        .collect()
}

/// Processes each of the pixels in the passed image, filtering out any that are
/// transparent or too light / dark to be interesting, then returns a `Vec` of the
/// `Rgba` channels of "interesting" pixels which is intended to be fed into
/// `NeuQuant`.
fn get_pixels(img: DynamicImage) -> Vec<u8> {
    let mut pixels = Vec::new();

    for (_, _, px) in img.pixels() {
        let rgba = px.to_rgba();

        if has_transparency(&rgba) || is_black(&rgba) || is_white(&rgba) {
            continue;
        }

        for channel in px.channels() {
            pixels.push(*channel);
        }
    }

    pixels
}

/// Checks if the passed pixel is opaque or not.
fn has_transparency(rgba: &Rgba<u8>) -> bool {
    let alpha_channel = rgba[3];

    alpha_channel != 255
}

/// Checks if the passed pixel is too dark to be interesting.
fn is_black(rgba: &Rgba<u8>) -> bool {
    rgba[0] < MIN_BLACK && rgba[1] < MIN_BLACK && rgba[2] < MIN_BLACK
}

/// Checks if the passed pixel is too light to be interesting.
fn is_white(rgba: &Rgba<u8>) -> bool {
    rgba[0] > MAX_WHITE && rgba[1] > MAX_WHITE && rgba[2] > MAX_WHITE
}

/// Maps each unique Lab color in the passed `Vec` of pixels to the total
/// number of times that color appears in the `Vec`.
fn count_colors_as_lab(pixels: Vec<Rgb<u8>>) -> Vec<(Lab, usize)> {
    let color_count_map = pixels.iter()
        .fold(BTreeMap::new(), |mut acc, px| {
            *acc.entry(px.channels()).or_insert(0) += 1;
            acc
        });

    let mut color_count_vec = color_count_map.iter()
        .fold(Vec::new(), |mut acc, (color, count)| {
            let rgb = Rgb::from_slice(&color).to_owned();
            acc.push((Lab::from_rgb(&[rgb[0], rgb[1], rgb[2]]), *count as usize));
            acc
        });

    color_count_vec.sort_by(|&(_, a), &(_, b)| b.cmp(&a));

    color_count_vec
}

fn remove_similar_colors(palette: Vec<(Lab, usize)>) -> Vec<(Lab, usize)> {
    let mut similars = Vec::new();
    let mut refined_palette: Vec<(Lab, usize)> = Vec::new();

    for &(lab_x, count_x) in palette.iter() {
        let mut is_similar = false;

        for (i, &(lab_y, _)) in refined_palette.iter().enumerate() {
            let delta = DE2000::new(lab_x.into(), lab_y.into());

            if delta < MIN_DISTANCE_FOR_UNIQUENESS {
                similars.push((i, lab_x, count_x));
                is_similar = true;
                break;
            }
        }

        if !is_similar {
            refined_palette.push((lab_x, count_x));
        }
    }

    for &(i, lab_y, count) in &similars {
        let lab_x = refined_palette[i].0;
        let (lx, ax, bx) = (lab_x.l, lab_x.a, lab_x.b);
        let (ly, ay, by) = (lab_y.l, lab_y.a, lab_y.b);

        let count_x = refined_palette[i].1 as f32;
        let count_y = count as f32;

        let balanced_l = (lx * count_x + ly * count_y) / (count_x + count_y);
        let balanced_a = (ax * count_x + ay * count_y) / (count_x + count_y);
        let balanced_b = (bx * count_x + by * count_y) / (count_x + count_y);

        refined_palette[i].0 = Lab {
            l: balanced_l,
            a: balanced_a,
            b: balanced_b,
        };

        refined_palette[i].1 += count_y as usize;
    }

    refined_palette.sort_by(|&(_, a), &(_, b)| b.cmp(&a));

    refined_palette
}

fn output_palette_as_img(palette: Vec<(Lab, usize)>, palette_size: u8) {
    let colors_img_width;

    if palette.len() < palette_size as usize {
        colors_img_width = 80 * palette.len();
    } else {
        colors_img_width = 80 * palette_size as usize;
    }

    let mut colors_img_buf = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(colors_img_width as u32, 80);

    for (i, &(color, _)) in palette.iter().enumerate() {
        let x_offset = (80 * i) as u32;
        let mut sub_img = imageops::crop(&mut colors_img_buf, x_offset, 0, 80, 80);
        let as_rgb = Lab::to_rgb(&color);
        let rgb = Rgb::from_channels(as_rgb[0], as_rgb[1], as_rgb[2], 255);

        for (_, _, px) in sub_img.pixels_mut() {
            px.data = rgb.data;
        }

        if i == palette_size as usize - 1 {
            break;
        }
    }

    let filename = format!("fout.png");

    if let Ok(ref mut fout) = File::create(&Path::new(&filename)) {
        let _ = image::ImageRgb8(colors_img_buf).save(fout, image::PNG);
    } else {
        println!("Failed to save the palette as an image.");
    };
}

#[cfg(test)]
mod tests {
    // use std::path::Path;

    // use image;
    use super::Distil;

    #[test]
    fn from_path_str() {
        let path_str = "/Users/elliot/dev/distil/images/img-1.jpg";

        match Distil::from_path_str(path_str, 5) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}
