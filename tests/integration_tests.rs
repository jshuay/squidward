// TODO: replace placeholder test with real tests
#[test]
fn placeholder_test() {
    squidward::payment_system::simulate(squidward::load_transactions_csv("test_files/valid.csv").unwrap());
}
