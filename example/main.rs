use image::{ImageBuffer, Luma};
use deriche::coefficients::Coefficients;
use deriche::detector::{load_image, magnitude_calculation, iir, direction, non_maximum_suppression, double_thresholding};

fn main() {
    let img = load_image();

    let alpha = 4.0;
    let c = Coefficients::smoothing(alpha);
    let smoothed2 = iir(&img, c);

    let c_derivative = Coefficients::x_derivative(alpha);
    let x_derivative = iir(&smoothed2, c_derivative);

    let cy_derivative = Coefficients::y_derivative(alpha);
    let y_derivative = iir(&smoothed2, cy_derivative);

    let mag = magnitude_calculation(&x_derivative, &y_derivative);
    let dir = direction(&x_derivative, &y_derivative);

    let non_max_suppressed = non_maximum_suppression(&mag, &dir);

    let d = double_thresholding(&non_max_suppressed, 50, 90, 25);

    type D = u8;

    let x: ImageBuffer<Luma<D>, Vec<D>> = x_derivative.into();
    let y: ImageBuffer<Luma<D>, Vec<D>> = y_derivative.into();

    x.save("xder.png").unwrap();
    y.save("yder.png").unwrap();

    let res: ImageBuffer<Luma<D>, Vec<D>> = mag.into();
    let res2: ImageBuffer<Luma<D>, Vec<D>> = smoothed2.into();
    let res3: ImageBuffer<Luma<D>, Vec<D>> = dir.into();

    let nm: ImageBuffer<Luma<D>, Vec<D>> = non_max_suppressed.into();


    res.save("mag.png").unwrap();
    res2.save("smooth.png").unwrap();
    res3.save("dir.png").unwrap();

    nm.save("nm.png").unwrap();
    d.save("double_thresh.png").unwrap();
}