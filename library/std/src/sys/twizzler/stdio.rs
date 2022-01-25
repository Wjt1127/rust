use crate::io;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin
    }
}

use twizzler_abi::syscall::KernelConsoleReadError;
#[unstable(feature = "twizzler_kernel_console_read", issue = "none", reason = "not all errors have been implemented")]
impl From<KernelConsoleReadError> for io::Error {
    fn from(x: KernelConsoleReadError) -> Self {
        match x {
            KernelConsoleReadError::WouldBlock => Self::from(io::ErrorKind::WouldBlock),
            KernelConsoleReadError::IOError => Self::from(io::ErrorKind::Unsupported),
            KernelConsoleReadError::NoSuchDevice => Self::from(io::ErrorKind::Unsupported)
        }
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        twizzler_abi::syscall::sys_kernel_console_read(buf, twizzler_abi::syscall::KernelConsoleReadFlags::empty()).map_err(|e| e.into())
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        twizzler_abi::syscall::sys_kernel_console_write(buf, twizzler_abi::syscall::KernelConsoleWriteFlags::empty());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        twizzler_abi::syscall::sys_kernel_console_write(buf, twizzler_abi::syscall::KernelConsoleWriteFlags::empty());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub const STDIN_BUF_SIZE: usize = 0;

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
