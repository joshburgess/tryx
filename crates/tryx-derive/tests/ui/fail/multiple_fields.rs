use tryx_derive::Outcome;

#[derive(Outcome)]
enum MultipleFields {
    #[outcome(success)]
    Yes(u8),
    #[outcome(short_circuit)]
    No(&'static str, &'static str),
}

fn main() {}
