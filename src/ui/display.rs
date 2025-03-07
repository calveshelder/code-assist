use colored::{Colorize, ColoredString};

pub fn format_code(code: &str) -> ColoredString {
    code.bright_white()
}

pub fn format_success(message: &str) -> ColoredString {
    message.bright_green()
}

pub fn format_error(message: &str) -> ColoredString {
    message.bright_red()
}

pub fn format_info(message: &str) -> ColoredString {
    message.bright_blue()
}
