#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        use colored::Colorize;
        let dt = chrono::Utc::now();

        let formated_time_date: String = dt.format("%Y-%m-%d %H:%M:%S").to_string();

        println!(
            "{} [{}]: {}",
            formated_time_date,
            "INFO".blue().bold(),
            format!($($arg)*)
        );

    });
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        use colored::Colorize;
        let dt = chrono::Utc::now();

        let formated_time_date: String = dt.format("%Y-%m-%d %H:%M:%S").to_string();

        println!(
            "{} [{}]: {}",
            formated_time_date,
            "ERROR".red().bold(),
            format!($($arg)*)
        );

    });
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => ({
        use colored::Colorize;
        let dt = chrono::Utc::now();

        let formated_time_date: String = dt.format("%Y-%m-%d %H:%M:%S").to_string();

        println!(
            "{} [{}]: {}",
            formated_time_date,
            "WARN".yellow().bold(),
            format!($($arg)*)
        );

    });
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        if cfg!(debug_assertions) {
            use colored::Colorize;
            let dt = chrono::Utc::now();

            let formated_time_date: String = dt.format("%Y-%m-%d %H:%M:%S").to_string();

            println!(
                "{} [{}]: {}",
                formated_time_date,
                "DEBUG".green().bold(),
                format!($($arg)*)
            );
        }
    });
}

#[macro_export]
macro_rules! command_error {
    ($($arg:tt)*) => {{
        $crate::error!("Command error: {}", format!($($arg)*));
        std::result::Result::Err($crate::ResponseError::new(format!($($arg)*)))
    }};
}

#[macro_export]
macro_rules! command_warn {
    ($($arg:tt)*) => {{
        $crate::warning!("Command warning: {}", format!($($arg)*));
        std::result::Result::Err($crate::ResponseError::warn(format!($($arg)*)))
    }};
}

#[macro_export]
macro_rules! command_info {
    ($($arg:tt)*) => {{
        $crate::info!("Command info: {}", format!($($arg)*));
        std::result::Result::Err($crate::ResponseError::info(format!($($arg)*)))
    }};
}
