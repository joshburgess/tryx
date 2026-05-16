use tryx::checked::{Checked, Finite};

fn quadratic_roots(a: f64, b: f64, c: f64) -> Checked<(Finite, Finite)> {
    let a = Finite::new(a)?;
    let b = Finite::new(b)?;
    let c = Finite::new(c)?;

    let four = Finite::new(4.0)?;
    let two = Finite::new(2.0)?;
    let discriminant = (b * b)? - ((four * a)? * c)?;
    let root = discriminant?.sqrt()?;
    let denominator = (two * a)?;
    let neg_b = Finite::new(-b.get())?;

    let first = ((neg_b + root)? / denominator)?;
    let second = ((neg_b - root)? / denominator)?;

    Checked::Done((first, second))
}

fn main() {
    match quadratic_roots(1.0, -3.0, 2.0) {
        Checked::Done((a, b)) => println!("roots: {}, {}", a.get(), b.get()),
        Checked::Failed(failure) => println!("failed: {failure}"),
    }
}
