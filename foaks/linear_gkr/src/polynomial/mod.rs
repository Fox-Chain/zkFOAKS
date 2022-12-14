use prime_field::FieldElement;

pub struct LinearPoly {
    pub a: FieldElement,
    pub b: FieldElement,
}

impl LinearPoly {
    pub fn new(a: FieldElement, b: FieldElement) -> Self {
        Self { a, b }
    }
    // Create a monomial with no variables
    pub fn new_constant_monomial(b: FieldElement) -> Self { 
        Self {
            a: FieldElement::from_real(0),
            b,
        }
    }

    pub fn eval(x: FieldElement) -> FieldElement {
        self.a * x + self.b
    }
}
impl core::ops::Add for LinearPoly {
    type Output = Self;

    fn add(self, x: Self) -> Self::Output {
        let a = self.a + x.a;
        let b = self.b + x.b;
        Self { a, b }
    }
}

impl core::ops::Mul for LinearPoly {
    type Output = QuadraticPoly;

    fn mul(self, x: Self) -> Self::Output {
        let a = self.a * x.a;
        let b = self.a * x.b + self.b * x.a;
        let c = self.b * x.b;
        QuadraticPoly::new(a, b, c)
    }
}

pub struct QuadraticPoly {
    a: FieldElement,
    b: FieldElement,
    c: FieldElement,
}

impl QuadraticPoly {
    pub fn new(a: FieldElement, b: FieldElement, c: FieldElement) -> Self {
        Self { a, b, c }
    }

    pub fn mul(self, x: LinearPoly) -> CubicPoly {
        let a = self.a * x.a;
        let b = self.a * x.b + self.b * x.a;
        let c = self.b * x.b + self.c * x.a;
        let d = self.c * x.b;
        CubicPoly::new(a, b, c, d)
    }
}

impl core::ops::Add for QuadraticPoly {
    type Output = Self;

    fn add(self, x: Self) -> Self::Output {
        let a = self.a + x.a;
        let b = self.b + x.b;
        let c = self.c + x.c;
        Self { a, b, c }
    }
}

pub struct CubicPoly {
    pub a: FieldElement,
    pub b: FieldElement,
    pub c: FieldElement,
    pub d: FieldElement,
}

impl CubicPoly {
    pub fn new(a: FieldElement, b: FieldElement, c: FieldElement, d: FieldElement) -> Self {
        Self { a, b, c, d }
    }
    pub fn eval(self, x: FieldElement) -> FieldElement {
        ((self.a * x + self.b) * x + self.c) * x + self.d
    }
}

impl core::ops::Add for CubicPoly {
    type Output = Self;

    fn add(self, x: Self) -> Self::Output {
        let a = self.a + x.a;
        let b = self.b + x.b;
        let c = self.c + x.c;
        let d = self.d + x.d;
        Self { a, b, c, d }
    }
}
pub struct QuadruplePoly {
    pub a: FieldElement,
    pub b: FieldElement,
    pub c: FieldElement,
    pub d: FieldElement,
    pub e: FieldElement,
}

impl QuadruplePoly {
    pub fn new(
        a: FieldElement,
        b: FieldElement,
        c: FieldElement,
        d: FieldElement,
        e: FieldElement,
    ) -> Self {
        Self { a, b, c, d, e }
    }
    pub fn eval(self, x: FieldElement) -> FieldElement {
        (((self.a * x + self.b) * x + self.c) * x + self.d) * x + self.e
    }
}

impl core::ops::Add for QuadruplePoly {
    type Output = Self;

    fn add(self, x: Self) -> Self::Output {
        let a = self.a + x.a;
        let b = self.b + x.b;
        let c = self.c + x.c;
        let d = self.d + x.d;
        let e = self.e + x.e;
        Self { a, b, c, d, e }
    }
}
pub struct QuintuplePoly {
    pub a: FieldElement,
    pub b: FieldElement,
    pub c: FieldElement,
    pub d: FieldElement,
    pub e: FieldElement,
    pub f: FieldElement,
}

impl QuintuplePoly {
    pub fn new(
        a: FieldElement,
        b: FieldElement,
        c: FieldElement,
        d: FieldElement,
        e: FieldElement,
        f: FieldElement,
    ) -> Self {
        return Self { a = aa, b = bb, c = cc, d = dd, e = ee, f = ff }
    }

    // pub fn operator(x: Self) {
    //     return Self {
    //         a + x.a, b + x.b, c + x.c, d + x.d, e + x.e, f + x.f
    //     }
    // }

    pub fn eval(x: FieldElement) {
        return (((((self.a * x) + self.b) * x + self.c) * x + self.d) * x + self.e) * x + self.f;
    }
}

impl core::ops::Add for QuintuplePoly {
    type Output = Self;

    fn add(self, x: Self) -> Self::Output {
        let a = self.a + x.a;
        let b = self.b + x.b;
        let c = self.c + x.c;
        let d = self.d + x.d;
        let f = self.f + x.f;
        Self { a, b, c, d, f }
    }
}