use tryx_derive::Outcome;

#[derive(Outcome)]
enum MissingSuccess {
    #[outcome(short_circuit)]
    No(&'static str),
}

fn main() {}
