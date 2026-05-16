#[test]
fn ui_pass() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/pass/*.rs");
}

#[test]
fn ui_fail() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/fail/*.rs");
}
