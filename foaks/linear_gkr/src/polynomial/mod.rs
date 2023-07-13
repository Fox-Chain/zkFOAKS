mod lib;

use prime_field::FieldElement;
#[derive(Debug, Clone)]

pub struct LinearPoly {
  pub a: FieldElement,
  pub b: FieldElement,
}

impl LinearPoly {
  //maps from FieldElement to LinearPoly //Check Original repo
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

  // Create a monomial with no variables
  pub fn new_constant_monomial(b: FieldElement) -> Self {
    Self {
      a: FieldElement::from_real(0),
      b,
    }
  }

  pub fn eval(&self, x: FieldElement) -> FieldElement { self.a * x + self.b }
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

  //todo: debug function
  pub fn eval(&self, x: &FieldElement) -> FieldElement { self.a * *x * *x + self.b * *x + self.c }

  pub fn multi(self, x: LinearPoly) -> CubicPoly {
    let a = self.a * x.a;
    let b = self.a * x.b + self.b * x.a;
    let c = self.b * x.b + self.c * x.a;
    let d = self.c * x.b;
    CubicPoly::new(a, b, c, d)
  }
}

impl core::ops::Add for QuadraticPoly {
  type Output = QuadraticPoly;

  fn add(self, x: QuadraticPoly) -> Self::Output {
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
    Self { a, b, c, d, e, f }
  }

  pub fn eval(self, x: FieldElement) -> FieldElement {
    (((((self.a * x) + self.b) * x + self.c) * x + self.d) * x + self.e) * x + self.f
  }
}

impl core::ops::Add for QuintuplePoly {
  type Output = Self;

  fn add(self, x: Self) -> Self::Output {
    let a = self.a + x.a;
    let b = self.b + x.b;
    let c = self.c + x.c;
    let d = self.d + x.d;
    let e = self.e + x.e;
    let f = self.f + x.f;
    Self { a, b, c, d, e, f }
  }
}
