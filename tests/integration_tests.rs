use std::process::Command;

const PROGRAM_PATH: &str = "target/debug/squidward";
const ACCOUNTS_HEADERS: &str = "client,available,held,total,locked\n";

#[test]
fn run_program_with_valid_csv_properly_outputs_accounts_csv() {
    let output = Command::new(PROGRAM_PATH).arg("test_files/valid.csv").output();

    assert!(!output.is_err());

    let expected = format!(
        "{}\
            1,1,0,1,false\n",
        ACCOUNTS_HEADERS
    );

    assert_eq!(expected, String::from_utf8_lossy(&output.unwrap().stdout));
}

#[test]
fn run_program_with_whitespaces_csv_properly_ignores_extraneous_whitespaces() {
    let output = Command::new(PROGRAM_PATH).arg("test_files/whitespaces.csv").output();

    assert!(!output.is_err());

    let expected = format!(
        "{}\
            1,1.5,0,1.5,false\n\
            2,0.1,0,0.1,false\n",
        ACCOUNTS_HEADERS
    );

    assert_eq!(expected, String::from_utf8_lossy(&output.unwrap().stdout));
}

#[test]
fn run_program_with_invalid_file_path_exits_program_with_error() {
    let exit_status = Command::new(PROGRAM_PATH)
        .arg("test_files/invalid_file_path.csv")
        .status();

    assert!(!exit_status.is_err());

    assert!(!exit_status.unwrap().success());
}

#[test]
fn run_program_with_invalid_format_csv_outputs_just_the_accounts_headers() {
    let output = Command::new(PROGRAM_PATH).arg("test_files/invalid.csv").output();

    assert_eq!(ACCOUNTS_HEADERS, String::from_utf8_lossy(&output.unwrap().stdout));
}

#[test]
fn run_program_with_invalid_casing_csv_ignores_transactions_with_incorrect_casing() {
    let output = Command::new(PROGRAM_PATH).arg("test_files/invalid_casing.csv").output();

    let expected = format!(
        "{}\
            1,1,0,1,false\n",
        ACCOUNTS_HEADERS
    );

    assert_eq!(expected, String::from_utf8_lossy(&output.unwrap().stdout));
}

#[test]
fn run_program_with_extra_decimals_csv_rounds_amounts_to_4_digits() {
    let output = Command::new(PROGRAM_PATH).arg("test_files/extra_decimals.csv").output();

    let expected = format!(
        "{}\
            1,1.0005,2.0005,3.001,false\n",
        ACCOUNTS_HEADERS
    );

    assert_eq!(expected, String::from_utf8_lossy(&output.unwrap().stdout));
}

#[test]
fn run_program_with_negative_balance_due_to_chargeback_csv_correctly_records_negative_balance() {
    let output = Command::new(PROGRAM_PATH)
        .arg("test_files/negative_balance_due_to_chargeback.csv")
        .output();

    let expected = format!(
        "{}\
            1,-5,0,-5,true\n",
        ACCOUNTS_HEADERS
    );

    assert_eq!(expected, String::from_utf8_lossy(&output.unwrap().stdout));
}

#[test]
fn run_program_with_disputing_multiple_times_csv_correctly_allows_multiple_disputes() {
    let output = Command::new(PROGRAM_PATH)
        .arg("test_files/disputing_multiple_times.csv")
        .output();

    let expected = format!(
        "{}\
            1,0,0,0,true\n",
        ACCOUNTS_HEADERS
    );

    assert_eq!(expected, String::from_utf8_lossy(&output.unwrap().stdout));
}

#[test]
fn run_program_with_malicious_transactions_csv_correctly_ignores_malicious_transactions() {
    let output = Command::new(PROGRAM_PATH)
        .arg("test_files/malicious_transactions.csv")
        .output();

    let expected = format!(
        "{}\
            1,100,0,100,true\n",
        ACCOUNTS_HEADERS
    );

    assert_eq!(expected, String::from_utf8_lossy(&output.unwrap().stdout));
}

#[test]
fn run_program_with_transactions_after_lock_csv_correctly_ignores_subsequent_transactions() {
    let output = Command::new(PROGRAM_PATH)
        .arg("test_files/transactions_after_lock.csv")
        .output();

    let expected = format!(
        "{}\
            1,0,0,0,true\n",
        ACCOUNTS_HEADERS
    );

    assert_eq!(expected, String::from_utf8_lossy(&output.unwrap().stdout));
}

#[test]
fn run_program_with_resolve_chargeback_before_dispute_csv_correctly_ignores_resolve_and_chargeback() {
    let output = Command::new(PROGRAM_PATH)
        .arg("test_files/resolve_chargeback_before_dispute.csv")
        .output();

    let expected = format!(
        "{}\
            1,0,100,100,false\n",
        ACCOUNTS_HEADERS
    );

    assert_eq!(expected, String::from_utf8_lossy(&output.unwrap().stdout));
}
