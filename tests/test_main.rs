mod utils;
use assert_cmd::Command;
use httpmock::Method::GET;
use httpmock::{Mock, MockServer};
use predicates::prelude::*;
use utils::{setup_tmp_directory, teardown_tmp_directory};

#[test]
/// send the function a file to which we dont have permission in order to execute error branch
fn main_use_root_owned_file_as_wordlist() -> Result<(), Box<dyn std::error::Error>> {
    let srv = MockServer::start();

    let mock = Mock::new()
        .expect_method(GET)
        .expect_path("/")
        .return_status(200)
        .return_body("this is a test")
        .create_on(&srv);

    Command::cargo_bin("feroxbuster")
        .unwrap()
        .arg("--url")
        .arg(srv.url("/"))
        .arg("--wordlist")
        .arg("/etc/shadow")
        .arg("-vvvv")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "ERROR main::get_unique_words_from_wordlist Permission denied (os error 13)",
        ));

    // connectivity test hits it once
    assert_eq!(mock.times_called(), 1);
    Ok(())
}

#[test]
/// send the function an empty file
fn main_use_empty_wordlist() -> Result<(), Box<dyn std::error::Error>> {
    let srv = MockServer::start();
    let (tmp_dir, file) = setup_tmp_directory(&[])?;

    let mock = Mock::new()
        .expect_method(GET)
        .expect_path("/")
        .return_status(200)
        .return_body("this is a test")
        .create_on(&srv);

    Command::cargo_bin("feroxbuster")
        .unwrap()
        .arg("--url")
        .arg(srv.url("/"))
        .arg("--wordlist")
        .arg(file.as_os_str())
        .arg("-vvvv")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ERROR main::scan Did not find any words in",
        ));

    assert_eq!(mock.times_called(), 1);

    teardown_tmp_directory(tmp_dir);
    Ok(())
}

#[test]
/// send nothing over stdin, expect heuristics to be upset during connectivity test
fn main_use_empty_stdin_targets() -> Result<(), Box<dyn std::error::Error>> {
    let (tmp_dir, file) = setup_tmp_directory(&[])?;

    // get_targets is called before scan, so the empty wordlist shouldn't trigger
    // the 'Did not find any words' error
    Command::cargo_bin("feroxbuster")
        .unwrap()
        .arg("--stdin")
        .arg("--wordlist")
        .arg(file.as_os_str())
        .arg("-vvv")
        .pipe_stdin(file)
        .unwrap()
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Could not connect to any target provided")
                .and(predicate::str::contains("ERROR"))
                .and(predicate::str::contains("heuristics::connectivity_test"))
                .and(predicate::str::contains("Target Url"))
                .not(), // no target url found
        );

    teardown_tmp_directory(tmp_dir);

    Ok(())
}