#[macro_export]
macro_rules! app_success {
    ($($arg:tt)*) => ({
        use ansi_term::Colour::Green;
        eprintln!("{} {}", Green.paint("[SUCCESS]"), format!($($arg)*))
    })
}

#[macro_export]
macro_rules! app_info {
    ($($arg:tt)*) => ({
        use ansi_term::Colour::Cyan;
        eprintln!("{} {}", Cyan.paint("[INFO]"), format!($($arg)*))
    })
}

#[macro_export]
macro_rules! app_warning {
    ($($arg:tt)*) => ({
        use ansi_term::Colour::Yellow;
        eprintln!("{} {}", Yellow.paint("[WARNING]"), format!($($arg)*))
    })
}

#[macro_export]
/// Extract GITLAB_URL from clap args
macro_rules! extract_url {
    ($clap_args:expr) => {
        match $clap_args.value_of("url") {
            Some(s) => s.to_owned(),
            None => env::var("GITLAB_URL").unwrap_or(String::from("https://gitlab.com")),
        }
    };
}

#[macro_export]
/// Extract GITLAB_API_TOKEN from clap args
macro_rules! extract_token {
    ($clap_args:expr) => {
        match $clap_args.value_of("token") {
            Some(s) => s.to_owned(),
            None => env::var("GITLAB_API_TOKEN").unwrap_or(String::new()),
        }
    };
}

#[macro_export]
/// Extract GitLab environment from clap args
macro_rules! extract_environment {
    ($clap_args:expr) => {
        $clap_args.value_of("environment").map_or_else(|| "All".to_owned(), |v| v.to_owned())
    };
}

#[macro_export]
/// Ceil division between two numbers
macro_rules! ceil_div {
    ($dividend:expr, $divider:expr) => {
        ($dividend as f64 / $divider as f64).ceil() as usize
    };
}

#[macro_export]
/// Floor division between two numbers
macro_rules! floor_div {
    ($dividend:expr, $divider:expr) => {
        ($dividend as f64 / $divider as f64).floor() as usize
    };
}
