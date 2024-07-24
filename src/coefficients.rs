#[derive(Copy, Clone)]
pub struct Coefficients {
    k: f64,

    a1: f64,
    a2: f64,
    a3: f64,
    a4: f64,

    a5: f64,
    a6: f64,
    a7: f64,
    a8: f64,

    b1: f64,
    b2: f64,

    c1: f64,
    c2: f64,
}

impl Coefficients {
    pub fn smoothing(alpha: f64) -> Self {
        let e_alpha = (-alpha).exp();
        let e2_alpha = (-2. * alpha).exp();
        let k = (1. - e_alpha).powi(2) / (1. + 2. * alpha * e_alpha - e2_alpha);

        let a2 = k * e_alpha * (alpha - 1.);
        let a3 = k * e_alpha * (alpha + 1.);
        let a4 = -k * e2_alpha;

        Self { k, a1: k, a2, a3, a4, a5: k, a6: a2, a7: a3, a8: a4, b1: 2. * e_alpha, b2: -e2_alpha, c1: 1., c2: 1. }
    }

    pub fn x_derivative(alpha: f64) -> Self {
        let e_alpha = (-alpha).exp();
        let e2_alpha = (-2. * alpha).exp();
        let k = (1. - e_alpha).powi(2) / (1. + 2. * alpha * e_alpha - e2_alpha);

        Self {
            k,
            a1: 0.,
            a2: 1.,
            a3: -1.,
            a4: 0.,
            a5: k,
            a6: k * e_alpha * (alpha - 1.),
            a7: k * e_alpha * (alpha + 1.),
            a8: -k * e2_alpha,
            b1: 2. * e_alpha,
            b2: -e2_alpha,
            c1: -(1. - e_alpha).powi(2),
            c2: 1.,
        }
    }

    pub fn y_derivative(alpha: f64) -> Self {
        let e_alpha = (-alpha).exp();
        let e2_alpha = (-2. * alpha).exp();
        let k = (1. - e_alpha).powi(2) / (1. + 2. * alpha * e_alpha - e2_alpha);

        Self {
            k,
            a1: k,
            a2: k * e_alpha * (alpha - 1.),
            a3: k * e_alpha * (alpha + 1.),
            a4: -k * e2_alpha,
            a5: 0.,
            a6: 1.,
            a7: -1.,
            a8: 0.,
            b1: 2. * e_alpha,
            b2: -e2_alpha,
            c1: 1.,
            c2: -(1. - e_alpha).powi(2),
        }
    }


    pub fn k(&self) -> f64 {
        self.k
    }
    pub fn a1(&self) -> f64 {
        self.a1
    }
    pub fn a2(&self) -> f64 {
        self.a2
    }
    pub fn a3(&self) -> f64 {
        self.a3
    }
    pub fn a4(&self) -> f64 {
        self.a4
    }
    pub fn a5(&self) -> f64 {
        self.a5
    }
    pub fn a6(&self) -> f64 {
        self.a6
    }
    pub fn a7(&self) -> f64 {
        self.a7
    }
    pub fn a8(&self) -> f64 {
        self.a8
    }
    pub fn b1(&self) -> f64 {
        self.b1
    }
    pub fn b2(&self) -> f64 {
        self.b2
    }
    pub fn c1(&self) -> f64 {
        self.c1
    }
    pub fn c2(&self) -> f64 {
        self.c2
    }
}

