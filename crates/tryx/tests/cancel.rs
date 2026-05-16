#![cfg(feature = "cancel")]

use tryx::cancel::{Cancel, CancelError, CancelToken};

#[derive(Debug, PartialEq, Eq)]
struct AppError;

impl From<CancelError> for AppError {
    fn from(_: CancelError) -> Self {
        Self
    }
}

fn maybe_cancel(cancel: bool) -> Cancel<u8> {
    if cancel {
        Cancel::cancelled()
    } else {
        Cancel::ok(7)
    }
}

fn result_from_cancel() -> Result<u8, AppError> {
    let value = maybe_cancel(false)?;
    Ok(value)
}

#[test]
fn umbrella_reexports_cancel_api() {
    assert_eq!(maybe_cancel(false), Cancel::Done(7));
    assert_eq!(result_from_cancel(), Ok(7));
}

#[test]
fn umbrella_cancel_feature_includes_token_support() {
    let (token, handle) = CancelToken::new();
    assert_eq!(token.check(), Cancel::Done(()));

    handle.cancel();
    assert_eq!(token.check(), Cancel::Cancelled);
}
