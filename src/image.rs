use std::ops::{Deref, Index, IndexMut};

use image::{EncodableLayout, ImageBuffer, ImageResult, Luma, Pixel, PixelWithColorType};
use num_traits::Zero;

#[derive(Copy, Clone)]
pub enum EdgeStrategy {
    EdgeReplication
}

pub struct GreyscaleImage<T> {
    pub(crate) data: Box<[T]>,
    width: usize,
    height: usize,
    edge_strategy: EdgeStrategy,
}

impl<T> GreyscaleImage<T>
    where T: Copy + Zero + Sized
{
    pub(crate) fn new(width: usize, height: usize) -> Self {
        let mut buf = vec![T::zero(); width * height];
        GreyscaleImage {
            data: buf.into_boxed_slice(),
            width,
            height,
            edge_strategy: EdgeStrategy::EdgeReplication,
        }
    }

    pub(crate) fn copy_from(other: &GreyscaleImage<T>) -> Self {
        let data = other.data.to_vec().into_boxed_slice();
        Self {
            width: other.width,
            height: other.height,
            data,
            edge_strategy: other.edge_strategy,
        }
    }


    #[inline]
    fn x(&self, x: isize) -> isize {
        x.max(0).min(self.width as isize - 1)
    }

    #[inline]
    fn y(&self, y: isize) -> isize {
        y.max(0).min(self.height as isize - 1)
    }


    pub fn at(&self, x: isize, y: isize) -> &T {
        match self.edge_strategy {
            EdgeStrategy::EdgeReplication => {
                &self.data[(self.y(y) * self.width as isize + self.x(x)) as usize]
            }
        }
    }

    fn at_mut(&mut self, x: isize, y: isize) -> &mut T {
        assert!(x < self.width as isize);
        assert!(y < self.height as isize);
        match self.edge_strategy {
            EdgeStrategy::EdgeReplication => {
                &mut self.data[(y.max(0) * self.width as isize + x.max(0)) as usize]
            }
        }
    }

    fn set(&mut self, x: usize, y: usize, value: T) {
        assert!(x < self.width);
        assert!(y < self.height);
        self.data[y * self.width + x] = value;
    }

    pub(crate) fn transpose(&mut self) {
        let n = self.height;
        let p = self.width;
        const BLOCK: usize = 32;

        let mut dst = vec![T::zero(); self.height * self.width].into_boxed_slice();

        for i in (0..n).step_by(BLOCK) {
            for j in 0..p {
                for b in 0..BLOCK {
                    if !(i + b < n) { break; }
                    dst[j * n + i + b] = self.data[(i + b) * p + j]
                }
            }
        }

        (self.width, self.height) = (self.height, self.width);
        self.data = dst;
    }
}

impl<T> GreyscaleImage<T> {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
}

impl<T> GreyscaleImage<T>
    where T: Copy + Clone + image::Primitive,
          [T]: EncodableLayout,
          Luma<T>: PixelWithColorType,
          [<Luma<T> as Pixel>::Subpixel]: EncodableLayout,
          Vec<T>: Deref<Target=[<Luma<T> as Pixel>::Subpixel]>
{
    pub fn save(self, file_name: &str) -> ImageResult<()> {
        let img: ImageBuffer<Luma<T>, Vec<T>> = self.into();
        img.save(file_name)
    }
}

impl<T> From<&ImageBuffer<Luma<T>, Vec<T>>> for GreyscaleImage<T>
    where T: Copy + Clone + image::Primitive {
    fn from(value: &ImageBuffer<Luma<T>, Vec<T>>) -> Self {
        let mut buf = Vec::from(value.as_raw().as_slice());
        GreyscaleImage {
            data: buf.into_boxed_slice(),
            width: value.width() as usize,
            height: value.height() as usize,
            edge_strategy: EdgeStrategy::EdgeReplication,
        }
    }
}

impl<T> Into<ImageBuffer<Luma<T>, Vec<T>>> for GreyscaleImage<T>
    where T: Copy + Clone + image::Primitive {
    fn into(self) -> ImageBuffer<Luma<T>, Vec<T>> {
        ImageBuffer::from_raw(self.width as u32, self.height as u32, Vec::from(self.data)).unwrap()
    }
}

impl<T> Index<(isize, isize)> for GreyscaleImage<T>
    where T: Copy + Clone + Zero {
    type Output = T;

    fn index(&self, index: (isize, isize)) -> &Self::Output {
        &self.at(index.0, index.1)
    }
}

impl<T> IndexMut<(isize, isize)> for GreyscaleImage<T>
    where T: Copy + Clone + Zero
{
    fn index_mut(&mut self, index: (isize, isize)) -> &mut Self::Output {
        self.at_mut(index.0, index.1)
    }
}

#[test]
fn test_transpose() {
    let mut m = GreyscaleImage::new(2, 3);
    m[(1, 0)] = 2;

    m.transpose();
    assert_eq!(m[(0, 1)], 2);
    assert_ne!(m[(1, 0)], 2);

    m.transpose();

    assert_eq!(m[(1, 0)], 2);
    assert_ne!(m[(0, 1)], 2);
}

pub struct Row<'a, T> {
    inner: &'a mut [T],
    zero: T,
}

impl<'a, T> Row<'a, T>
    where T: Zero {
    pub fn new(inner: &'a mut [T]) -> Self {
        Self {
            inner,
            zero: T::zero(),
        }
    }
}

impl<'a, T> Index<isize> for Row<'a, T>
    where T: Zero + Copy {
    type Output = T;

    fn index(&self, index: isize) -> &Self::Output {
        if index < 0 {
            unsafe { &self.zero }
        } else if index >= self.inner.len() as isize {
            &self.zero
        } else {
            &self.inner[index as usize]
        }
    }
}

impl<'a, T> IndexMut<isize> for Row<'a, T>
    where T: Zero + Copy {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        &mut self.inner[index as usize]
    }
}
