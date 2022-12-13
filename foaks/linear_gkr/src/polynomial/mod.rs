use prime_field::FieldElement;

pub struct LinearPoly {
    pub a: FieldElement,
    pub b: FieldElement,
}

impl LinearPoly {
    pub fn new(a: FieldElement, b: FieldElement) -> Self {
        Self { a, b }
    }

    pub fn new(b: FieldElement) -> Self {
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
    type QuadraticOutput = QuadraticPoly;

    fn mul(self, x: Self) -> Self::QuadraticOutput {
        let a = self.a + x.a;
        let b = self.a * x.b + self.b * x.a;
        let c = self.b + x.b;
        Self { a, b, c }
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

    pub fn operator(x: Self) {
        return Self {
            a + x.a, b + x.b, c + x.c, d + x.d, e + x.e, f + x.f
        }
    }

    pub fn eval(x: FieldElement) {
        return (((((self.a * x) + self.b) * x + self.c) * x + self.d) * x + self.e) * x + self.f;
    }
}
