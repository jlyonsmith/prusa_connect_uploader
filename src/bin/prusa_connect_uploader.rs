use core::fmt::Arguments;
use prusa_connect_uploader::{error, PrusaConnectUploaderLog, PrusaConnectUploaderTool};
use yansi::Paint;

struct PrusaConnectUploaderLogger;

impl PrusaConnectUploaderLogger {
    fn new() -> PrusaConnectUploaderLogger {
        PrusaConnectUploaderLogger {}
    }
}

impl PrusaConnectUploaderLog for PrusaConnectUploaderLogger {
    fn output(self: &Self, args: Arguments) {
        println!("{}", args);
    }
    fn warning(self: &Self, args: Arguments) {
        eprintln!("{}", Paint::yellow(&format!("warning: {}", args)));
    }
    fn error(self: &Self, args: Arguments) {
        eprintln!("{}", Paint::red(&format!("error: {}", args)));
    }
}

fn main() {
    let logger = PrusaConnectUploaderLogger::new();

    if let Err(error) = PrusaConnectUploaderTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
