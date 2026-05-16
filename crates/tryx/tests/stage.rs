#![cfg(feature = "stage")]

use tryx::stage::{Stage, StageFailure};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Parse,
}

fn parse(input: &str) -> Stage<Step, u32, &str> {
    match input.parse::<u32>() {
        Ok(value) => Stage::done(value),
        Err(_) => Stage::failed(Step::Parse, input),
    }
}

#[test]
fn umbrella_reexports_stage_api() {
    fn run() -> Stage<Step, u32, &'static str> {
        let value = parse("bad")?;
        Stage::done(value)
    }

    assert_eq!(
        run(),
        Stage::Failed(StageFailure {
            stage: Step::Parse,
            partial: "bad"
        })
    );
}
