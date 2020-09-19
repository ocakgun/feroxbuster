mod utils;
use utils::{teardown_tmp_directory, setup_tmp_directory};
use assert_cmd::prelude::*;
use httpmock::Method::GET;
use httpmock::{Mock, MockServer};
use predicates::prelude::*;
use std::process::Command;


#[test]
fn test_single_request_scan() -> Result<(), Box<dyn std::error::Error>> {
    let srv = MockServer::start();
    let (tmp_dir, file) = setup_tmp_directory(&["LICENSE".to_string()])?;

    let mock = Mock::new()
        .expect_method(GET)
        .expect_path("/LICENSE")
        .return_status(200)
        .return_body("this is a test")
        .create_on(&srv);

    let cmd = Command::cargo_bin("feroxbuster")
        .unwrap()
        .arg("--url")
        .arg(srv.url("/"))
        .arg("--wordlist")
        .arg(file.as_os_str())
        .unwrap();

    cmd.assert().success().stdout(
        predicate::str::contains("/LICENSE")
            .and(predicate::str::contains("200 OK"))
            .and(predicate::str::contains("[14 bytes]"))
    );

    assert_eq!(mock.times_called(), 1);
    teardown_tmp_directory(tmp_dir);
    Ok(())
}

// #[test]
// fn test_si() -> Result<(), Box<dyn std::error::Error>> {
//
// }