use prime_field::FieldElement;

#[derive(Debug, Clone)]
pub struct LinearPoly {
  pub a: FieldElement,
  pub b: FieldElement,
}

impl LinearPoly {
  pub fn new_single_input(arg: FieldElement) -> Self {
    Self {
      a: FieldElement::zero(),
      b: arg,
    }
  }

  pub fn zero() -> Self {
    Self {
      a: FieldElement::zero(),
      b: FieldElement::zero(),
    }
  }

  pub fn new(a: FieldElement, b: FieldElement) -> Self { Self { a, b } }

  pub fn eval(&self, x: FieldElement) -> FieldElement { self.a * x + self.b }
}

impl core::ops::Add for LinearPoly {
  type Output = Self;

  fn add(self, other: Self) -> Self::Output {
    Self {
      a: self.a + other.a,
      b: self.b + other.b,
    }
  }
}

impl core::ops::Mul for LinearPoly {
  type Output = QuadraticPoly;

  fn mul(self, other: Self) -> Self::Output {
    let a = self.a * other.a;
    let b = self.a * other.b + self.b * other.a;
    let c = self.b * other.b;
    QuadraticPoly { a, b, c }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct QuadraticPoly {
  pub a: FieldElement,
  pub b: FieldElement,
  pub c: FieldElement,
}

impl QuadraticPoly {
  pub fn zero() -> Self {
    Self {
      a: FieldElement::zero(),
      b: FieldElement::zero(),
      c: FieldElement::zero(),
    }
  }

  pub fn new(a: FieldElement, b: FieldElement, c: FieldElement) -> Self { Self { a, b, c } }

  pub fn eval(&self, x: &FieldElement) -> FieldElement { self.a * (*x * *x) + self.b * *x + self.c }
}

impl core::ops::Add for QuadraticPoly {
  type Output = Self;

  fn add(self, other: Self) -> Self::Output {
    Self {
      a: self.a + other.a,
      b: self.b + other.b,
      c: self.c + other.c,
    }
  }
}
