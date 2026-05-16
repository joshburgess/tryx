use tryx_derive::Outcome;

#[derive(Outcome)]
enum MultipleSuccess {
    #[outcome(success)]
    A(u8),
    #[outcome(success)]
    B(u8),
    #[outcome(short_circuit)]
    No(&'static str),
}

fn main() {}
