mod log_macros;

use clap::Parser;
use core::fmt::Arguments;
use duct::cmd;
use std::{error::Error, fs, path::PathBuf, thread::sleep, time::Duration};

pub trait PrusaConnectUploaderLog {
    fn output(self: &Self, args: Arguments);
    fn warning(self: &Self, args: Arguments);
    fn error(self: &Self, args: Arguments);
}

pub struct PrusaConnectUploaderTool<'a> {
    log: &'a dyn PrusaConnectUploaderLog,
}

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Disable colors in output
    #[arg(long = "no-color", env = "NO_CLI_COLOR")]
    no_color: bool,

    /// Snapshot interval
    #[arg(long = "interval", default_value_t = SNAPSHOT_INTERVAL, env = "PRUSA_CONNECT_SNAPSHOT_INTERVAL")]
    interval: u64,

    /// API Token
    #[arg(long = "token", env = "PRUSA_CONNECT_CAMERA_TOKEN")]
    token: String,

    /// API Fingerprint
    #[arg(long = "fingerprint", env = "PRUSA_CONNECT_CAMERA_FINGERPRINT")]
    fingerprint: String,

    /// Debug
    #[arg(long = "debug")]
    debug: bool,
}

const HTTPS_URL: &str = "https://webcam.connect.prusa3d.com/c/snapshot";
const SNAPSHOT_INTERVAL: u64 = 10;

impl<'a> PrusaConnectUploaderTool<'a> {
    pub fn new(log: &'a dyn PrusaConnectUploaderLog) -> PrusaConnectUploaderTool<'a> {
        PrusaConnectUploaderTool { log }
    }

    pub fn run(
        self: &mut Self,
        args: impl IntoIterator<Item = std::ffi::OsString>,
    ) -> Result<(), Box<dyn Error>> {
        let cli = match Cli::try_parse_from(args) {
            Ok(m) => m,
            Err(err) => {
                output!(self.log, "{}", err.to_string());
                return Ok(());
            }
        };

        let mut delay = 0;

        loop {
            sleep(Duration::from_secs(delay));

            let jpg_path = PathBuf::from(format!("/dev/shm/camera_{}.jpg", cli.fingerprint));

            match cmd!(
                "rpicam-still",
                "--immediate",
                "--nopreview",
                "--mode",
                "2592:1944:12:P",
                "--lores-width",
                "0",
                "--lores-height",
                "0",
                "--thumb",
                "none",
                "--output",
                &jpg_path,
            )
            .stderr_to_stdout()
            .read()
            {
                Err(_) => {
                    error!(self.log, "{}", "Unable to generate snapshot");
                    delay = cli.interval * 3;
                    continue;
                }
                Ok(output) => {
                    if cli.debug {
                        output!(self.log, "{}", output);
                    }
                }
            };

            let metadata = fs::metadata(&jpg_path).unwrap();

            output!(
                self.log,
                "Captured file '{}' ({} bytes)",
                jpg_path.to_string_lossy(),
                metadata.len()
            );

            let verbose = cli.debug;
            match cmd!(
                "curl",
                "-X",
                "PUT",
                HTTPS_URL,
                "-H",
                "Accept: text/plain",
                "-H",
                "Content-type: image/jpg",
                "-H",
                format!("Fingerprint: {}", &cli.fingerprint),
                "-H",
                format!("Token: {}", &cli.token),
                "-H",
                format!("Content-length: {}", metadata.len()),
                "--data-binary",
                format!("@{}", jpg_path.to_string_lossy()),
                "--no-progress-meter",
                "--compressed",
                "--max-time",
                "5",
            )
            .stderr_to_stdout()
            .before_spawn(move |cmd| {
                if verbose {
                    cmd.arg("--verbose");
                }
                Ok(())
            })
            .read()
            {
                Err(_) => {
                    error!(self.log, "{}", "Unable to upload snapshot");
                    delay = cli.interval * 3;
                    continue;
                }
                Ok(output) => {
                    if cli.debug {
                        output!(self.log, "{}", output);
                    }
                    output!(self.log, "Successfully uploaded to '{}'", HTTPS_URL);
                }
            };

            output!(self.log, "Waiting {} seconds...", cli.interval);
            delay = cli.interval;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        struct TestLogger;

        impl TestLogger {
            fn new() -> TestLogger {
                TestLogger {}
            }
        }

        impl PrusaConnectUploaderLog for TestLogger {
            fn output(self: &Self, _args: Arguments) {}
            fn warning(self: &Self, _args: Arguments) {}
            fn error(self: &Self, _args: Arguments) {}
        }

        let logger = TestLogger::new();
        let mut tool = PrusaConnectUploaderTool::new(&logger);
        let args: Vec<std::ffi::OsString> = vec!["".into(), "--help".into()];

        tool.run(args).unwrap();
    }
}
