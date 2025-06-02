use core::fmt::Arguments;
use prusa_connect_uploader::{error, PrusaconnectUploaderLog, PrusaconnectUploaderTool};
use yansi::Paint;

struct PrusaconnectUploaderLogger;

impl PrusaconnectUploaderLogger {
    fn new() -> PrusaconnectUploaderLogger {
        PrusaconnectUploaderLogger {}
    }
}

impl PrusaconnectUploaderLog for PrusaconnectUploaderLogger {
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
    let logger = PrusaconnectUploaderLogger::new();

    if let Err(error) = PrusaconnectUploaderTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
