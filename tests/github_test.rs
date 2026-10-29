use std::process::Command;

#[test]
fn malformed_github_token_returns_an_error_without_panicking() {
    // Exercise both the required-auth and optional-auth client constructors.
    for command in ["create-pr", "check-diff"] {
        let output = Command::new(env!("CARGO_BIN_EXE_clu"))
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .arg(command)
            .env("GITHUB_TOKEN", "first-line\nsecond-line")
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{command}: {stderr}");
        assert!(
            stderr.contains("GITHUB_TOKEN contains control characters"),
            "{stderr}"
        );
        assert!(
            !stderr.contains("first-line"),
            "token must not be disclosed"
        );
        assert!(
            !stderr.contains("second-line"),
            "token must not be disclosed"
        );
    }
}
