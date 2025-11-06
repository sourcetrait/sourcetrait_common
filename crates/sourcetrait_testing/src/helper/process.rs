//use crate::*;

pub trait TestOutputTrait {
    fn expect_success(&mut self, expecting: &'static str);
}

impl TestOutputTrait for std::process::Command {
    fn expect_success(&mut self, expecting: &'static str) {
        let output = self.output().expect(expecting);
        if output.status.success() {
            return;
        }
        
        let stdout = String::from_utf8_lossy(output.stdout.as_slice());
        let stderr = String::from_utf8_lossy(output.stderr.as_slice());
        
        let cmd = self.get_program().to_string_lossy();
        let args = self.get_args()
            .map(std::ffi::OsStr::to_string_lossy)
            .collect::<Vec<_>>().join(" ");
        
        eprintln!("error: Command execution failed");
        eprintln!("`{cmd} {args}`");
        eprintln!("  stdout: {stdout}");
        eprintln!("  stderr: {stderr}");
    }
}
