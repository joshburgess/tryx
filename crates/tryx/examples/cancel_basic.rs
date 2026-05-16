use tryx::cancel::Cancel;

fn check(cancel: bool) -> Cancel<()> {
    if cancel {
        Cancel::Cancelled
    } else {
        Cancel::Done(())
    }
}

fn cancellable_loop(limit: u32, cancel_at: Option<u32>) -> Cancel<u32> {
    let mut total = 0;

    for i in 0..limit {
        check(cancel_at == Some(i))?;
        total += i;
    }

    Cancel::Done(total)
}

fn main() {
    match cancellable_loop(10, Some(4)) {
        Cancel::Done(total) => println!("done: {total}"),
        Cancel::Cancelled => println!("cancelled"),
    }
}
