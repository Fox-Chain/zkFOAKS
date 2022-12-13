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
