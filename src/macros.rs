#[cfg(test)]
macro_rules! trace {
    ($($arg:tt)*) => {{
        println!($($arg)*);
    }}
}

#[cfg(not(test))]
macro_rules! trace {
    ($($arg:tt)*) => {{
        // Wrap format! so we get IDE formatting of the format string, but don't actually run any
        // code unless we're testing.
        if false { format!($($arg)*); }
    }};
}
