#[test]
fn ui_pass() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/pass/*.rs");
}
