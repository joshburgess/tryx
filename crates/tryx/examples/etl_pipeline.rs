use tryx::stage::{Stage, StageFailure};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Extract,
    Parse,
    Transform,
    Load,
}

fn extract(source: &str) -> Stage<Step, &str, &str> {
    if source.is_empty() {
        Stage::failed(Step::Extract, source)
    } else {
        Stage::done(source)
    }
}

fn parse(input: &str) -> Stage<Step, i32, &str> {
    match input.parse::<i32>() {
        Ok(value) => Stage::done(value),
        Err(_) => Stage::failed(Step::Parse, input),
    }
}

fn transform(value: i32) -> Stage<Step, i32, i32> {
    if value < 0 {
        Stage::failed(Step::Transform, value)
    } else {
        Stage::done(value * 2)
    }
}

fn load(value: i32) -> Stage<Step, i32, i32> {
    if value > 100 {
        Stage::failed(Step::Load, value)
    } else {
        Stage::done(value)
    }
}

fn run(input: &str) -> Stage<Step, i32, i32> {
    let raw = match extract(input) {
        Stage::Done(raw) => raw,
        Stage::Failed(StageFailure { stage, .. }) => return Stage::failed(stage, 0),
    };
    let parsed = match parse(raw) {
        Stage::Done(value) => value,
        Stage::Failed(StageFailure { stage, .. }) => return Stage::failed(stage, 0),
    };
    let transformed = transform(parsed)?;
    load(transformed)
}

fn main() {
    match run("80") {
        Stage::Done(value) => println!("loaded {value}"),
        Stage::Failed(StageFailure {
            stage: Step::Load,
            partial,
        }) => {
            println!("load failed at {partial}, retrying with capped value");
            println!("{:?}", load(100));
        }
        Stage::Failed(failure) => println!("failed at {:?}", failure.stage),
    }
}
