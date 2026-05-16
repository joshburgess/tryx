use tryx_core::TryxResidual;

struct Residual;

impl TryxResidual for Residual {}

fn accepts_residual<R: TryxResidual>() {}

fn main() {
    accepts_residual::<Residual>();
}
