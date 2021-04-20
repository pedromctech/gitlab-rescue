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
        $clap_args
            .value_of("url")
            .map_or_else(|| env::var("GITLAB_URL").unwrap_or(String::from("https://gitlab.com")), |s| s.to_owned())
    };
}

#[macro_export]
/// Extract GITLAB_API_TOKEN from clap args
macro_rules! extract_token {
    ($clap_args:expr) => {
        $clap_args
            .value_of("token")
            .map_or_else(|| env::var("GITLAB_API_TOKEN").unwrap_or(String::new()), |s| s.to_owned())
    };
}

#[macro_export]
/// Extract GitLab environment from clap args
macro_rules! extract_environment {
    ($clap_args:expr) => {
        $clap_args.value_of("environment").map_or_else(|| "All".to_owned(), |v| v.to_owned())
    };
}
