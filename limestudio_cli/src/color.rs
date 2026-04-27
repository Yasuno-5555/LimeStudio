//! Minimal ANSI Color utility.
//! Replaces 'colored' crate to minimize dependencies.

#[allow(dead_code)]
pub trait Colorize {
    fn red(self) -> String;
    fn green(self) -> String;
    fn yellow(self) -> String;
    fn magenta(self) -> String;
    fn cyan(self) -> String;
    fn bold(self) -> String;
    fn dimmed(self) -> String;
}

impl Colorize for &str {
    fn red(self) -> String { format!("\x1b[31m{}\x1b[0m", self) }
    fn green(self) -> String { format!("\x1b[32m{}\x1b[0m", self) }
    fn yellow(self) -> String { format!("\x1b[33m{}\x1b[0m", self) }
    fn magenta(self) -> String { format!("\x1b[35m{}\x1b[0m", self) }
    fn cyan(self) -> String { format!("\x1b[36m{}\x1b[0m", self) }
    fn bold(self) -> String { format!("\x1b[1m{}\x1b[0m", self) }
    fn dimmed(self) -> String { format!("\x1b[2m{}\x1b[0m", self) }
}

impl Colorize for String {
    fn red(self) -> String { self.as_str().red() }
    fn green(self) -> String { self.as_str().green() }
    fn yellow(self) -> String { self.as_str().yellow() }
    fn magenta(self) -> String { self.as_str().magenta() }
    fn cyan(self) -> String { self.as_str().cyan() }
    fn bold(self) -> String { self.as_str().bold() }
    fn dimmed(self) -> String { self.as_str().dimmed() }
}
