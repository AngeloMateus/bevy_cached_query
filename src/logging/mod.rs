pub const PERFORMANCE_LOG_THRESHOLD_IN_MICROSECONDS: u128 = 40;

#[macro_export]
macro_rules! function_name {
    () => {{
        fn f() {
        }
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }};
}

#[cfg(any(debug_assertions, target_os = "macos"))]
#[macro_export]
macro_rules! debug_end {
    ($start:expr, $t:expr) => {{
        fn f() {
        }
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let type_name = type_name_of(f);
        let name = if let Some(stripped_name) = type_name.strip_prefix("membooster::") {
            stripped_name
        } else {
            type_name
        };

        let end = std::time::SystemTime::now();
        let duration = end.duration_since($start).unwrap();
        if duration.as_micros() > $t {
            if duration.as_micros() < 100 {
                color_print::cprintln!(
                    "proto <magenta>{} μs / {} s</magenta> {}",
                    duration.as_micros(),
                    duration.as_secs_f32(),
                    name.strip_suffix("::f").unwrap(),
                );
            } else if duration.as_micros() < 240 {
                color_print::cprintln!(
                    "proto <yellow>{} μs / {} s</yellow> {}",
                    duration.as_micros(),
                    duration.as_secs_f32(),
                    name.strip_suffix("::f").unwrap(),
                );
            } else {
                color_print::cprintln!(
                    "proto <red>{} μs / {} s</red> {}",
                    duration.as_micros(),
                    duration.as_secs_f32(),
                    name.strip_suffix("::f").unwrap(),
                );
            }
        }
    }};
}

#[cfg(all(not(debug_assertions), target_os = "ios"))]
#[macro_export]
macro_rules! debug_end {
    ($start:expr, $t:expr) => {{}};
}

#[cfg(any(debug_assertions, target_os = "macos"))]
#[macro_export]
macro_rules! proto {
    ($($args:tt)+) => {{
        std::println!("proto {}\n-> {}:{}:{}", format_args!($($args)*), file!(), line!(), column!());
    }};
}

#[cfg(all(not(debug_assertions), target_os = "ios"))]
#[macro_export]
macro_rules! proto {
    ($($args:tt)+) => {{}};
}
