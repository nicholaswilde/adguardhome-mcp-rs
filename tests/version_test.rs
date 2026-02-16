use std::process::Command;

#[test]
fn test_version_flag() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = format!("adguardhome-mcp-rs {}", env!("CARGO_PKG_VERSION"));
    assert!(stdout.contains(&expected));
}

#[test]
fn test_version_flag_short() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "-V"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = format!("adguardhome-mcp-rs {}", env!("CARGO_PKG_VERSION"));
    assert!(stdout.contains(&expected));
}
