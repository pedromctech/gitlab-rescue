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
        use ansi_term::Colour::Blue;
        eprintln!("{} {}", Blue.paint("[INFO]"), format!($($arg)*))
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
        match $clap_args.value_of("environment") {
            Some("All") | None => "*".to_owned(),
            Some(val) => val.to_string(),
        }
    };
}

#[macro_export]
/// Extract numeric argument from clap args
macro_rules! numeric_arg {
    ($clap_args:expr, $arg:expr, $default:expr) => {{
        match $clap_args.value_of($arg) {
            Some(v) if v.parse::<u32>().is_ok() => v.parse::<u32>().unwrap(),
            _ => $default,
        }
    }};
}
