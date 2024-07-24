use std::f64::consts::PI;

use image::ImageBuffer;
use image::io::Reader as ImageReader;
use num_traits::{Bounded, cast, NumCast, PrimInt, WrappingAdd, Zero};
use rayon::iter::IndexedParallelIterator;
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;

use crate::coefficients::Coefficients;
use crate::image::{GreyscaleImage, Row};

struct Deriche {}

pub fn load_image() -> GreyscaleImage<u8> {
    let img = ImageReader::open("./test_images/Magician_artwork.jpg").unwrap().decode().unwrap();
    GreyscaleImage::from(&ImageBuffer::from(img))
}

fn to_t<T>(f: f64) -> T
    where T: PrimInt + Into<f64> + NumCast, f64: From<T> {
    if f < 0. {
        // T::max_value() - cast(f.abs() % T::max_value().into()).unwrap()
        T::zero()
    } else if f > T::max_value().into() {
        // cast(f % T::max_value().into()).unwrap()
        T::max_value()
    } else {
        cast(f).unwrap()
    }
}

fn g<T>(r: &mut [T], i: isize) -> T
    where T: Zero + Copy + Into<f64> {
    if i >= r.len() as isize || i < 0 {
        return T::zero();
    }
    r[i as usize]
}

fn irr_rows<T>(image: &mut GreyscaleImage<T>, y1: &mut [f64], y2: &mut [f64],
               a: [f64; 4], b: [f64; 2], c: f64)
    where T: Copy + PrimInt + Into<f64> + Send, f64: From<T> {
    let [a1, a2, a3, a4]: [f64; 4] = a;
    let [b1, b2]: [f64; 2] = b;

    let width = image.width() as isize;

    image.data.par_chunks_exact_mut(
        width as usize
    )
        .zip((y1.par_chunks_exact_mut(width as usize), y2.par_chunks_exact_mut(width as usize)))
        .map(|(a, (b, c))| (Row::new(a), (Row::new(b), Row::new(c))))
        .for_each(
            |(mut row, (mut y1_row, mut y2_row))| {
                for x in 0..(width - 1) {
                    y1_row[x] =
                        a1 * row[x].into()
                            + a2 * row[x - 1].into()
                            + b1 * y1_row[x - 1]
                            + b2 * y1_row[x - 2]
                }

                for x in (0..(width - 1)).rev() {
                    y2_row[x] = a3 * row[x + 1].into()
                        + a4 * row[x + 2].into()
                        + b1 * y2_row[x + 1]
                        + b2 * y2_row[x + 2]
                }

                for i in 0..width {
                    row[i] = to_t(
                        c * (y1_row[i] + y2_row[i])
                    )
                }
            }
        )
}

pub fn iir<T>(image: &GreyscaleImage<T>, c: Coefficients) -> GreyscaleImage<T>
    where T: Copy + Zero + PrimInt + Into<f64> + Send, f64: From<T> {
    let mut y1 = vec![0.; image.width() * image.height()];
    let mut y2 = vec![0.; image.width() * image.height()];

    let mut r: GreyscaleImage<T> = GreyscaleImage::copy_from(image);


    irr_rows(&mut r, y1.as_mut_slice(), y2.as_mut_slice(), [c.a1(), c.a2(), c.a3(), c.a4()], [c.b1(), c.b2()], c.c1());

    r.transpose();
    for i in 0..(image.width() * image.height()) {
        y1[i] = 0.;
        y2[i] = 0.;
    }

    irr_rows(&mut r, y1.as_mut_slice(), y2.as_mut_slice(), [c.a5(), c.a6(), c.a7(), c.a8()], [c.b1(), c.b2()], c.c2());
    r.transpose();

    r
}

pub fn magnitude_calculation<T>(x_derivative: &GreyscaleImage<T>, y_derivative: &GreyscaleImage<T>) -> GreyscaleImage<T>
    where T: Copy + Zero + PrimInt + Into<f64> + Send, f64: From<T> {
    let width = x_derivative.width();
    let height = x_derivative.height();
    let mut y1: GreyscaleImage<T> = GreyscaleImage::new(width, height);

    for x in 0..(width - 1) as isize {
        for y in 0..(height - 1) as isize {
            let val = (
                x_derivative[(x, y)].into().powi(2)
                    + y_derivative[(x, y)].into().powi(2)
            ).sqrt();
            y1[(x, y)] = to_t(val)
        }
    }

    y1
}

pub fn direction<T>(x_derivative: &GreyscaleImage<T>, y_derivative: &GreyscaleImage<T>) -> GreyscaleImage<T>
    where T: Copy + Zero + PrimInt + Into<f64> + Send, f64: From<T> {
    let width = x_derivative.width();
    let height = x_derivative.height();
    let mut y1: GreyscaleImage<T> = GreyscaleImage::new(width, height);

    for x in 0..(width - 1) as isize {
        for y in 0..(height - 1) as isize {
            let xd = x_derivative[(x, y)].into();
            let yd = y_derivative[(x, y)].into();

            let tt = if xd != 0. {
                let at = (yd / xd).atan();
                let sc = (at + (PI / 2.)) * 180.0 / PI;
                to_t(
                    sc
                )
            } else { T::zero() };


            y1[(x, y)] = tt;
        }
    }
    y1
}

pub fn non_maximum_suppression<T>(magnitude: &GreyscaleImage<T>, direction: &GreyscaleImage<T>) -> GreyscaleImage<T>
    where T: Copy + Zero + PartialOrd + NumCast {
    let mut result = GreyscaleImage::new(magnitude.width(), magnitude.height());

    let limits = [0.0, 22.5, 67.5, 112.5, 157.5, 180.0];
    let cast_limits: [T; 6] = limits.map(|l| cast(l).unwrap());


    let width = magnitude.width() as isize;
    let height = magnitude.height() as isize;

    for x in 0..(width - 1) {
        for y in 0..(height - 1) {
            let q_i;
            let r_i;


            let angle = direction[(x, y)];

            if (cast_limits[0] <= angle && angle < cast_limits[1]) || (cast_limits[4] <= angle && angle <= cast_limits[5]) {
                r_i = (x, y - 1);
                q_i = (x, y + 1);
            } else if cast_limits[1] <= angle && angle < cast_limits[2] {
                r_i = (x - 1, y + 1);
                q_i = (x + 1, y - 1);
            } else if cast_limits[2] <= angle && angle < cast_limits[3] {
                r_i = (x - 1, y);
                q_i = (x + 1, y);
            } else if cast_limits[3] <= angle && angle < cast_limits[4] {
                r_i = (x + 1, y + 1);
                q_i = (x - 1, y - 1);
            } else { unreachable!("never reach here!") }

            if magnitude[(x, y)] >= magnitude[q_i] && magnitude[(x, y)] >= magnitude[r_i] {
                result[(x, y)] = magnitude[(x, y)]
            } else {
                // no need to set to zero, result is already initialized as zero
            }
        }
    }
    result
}

pub fn double_thresholding<T>(image: &GreyscaleImage<T>, low_threshold: T, high_threshold: T, weak_intensity: T) -> GreyscaleImage<T>
    where T: Copy + Zero + PartialOrd + Bounded
{
    let width = image.width() as isize;
    let height = image.height() as isize;

    let mut res = GreyscaleImage::new(width as usize, height as usize);

    for x in 0..width {
        for y in 0..height {
            let val = image[(x, y)];
            res[(x, y)] = if val < low_threshold {
                T::zero()
            } else if val < high_threshold {
                weak_intensity
            } else {
                T::max_value()
            }
        }
    }

    res
}

fn hysteresis() {}