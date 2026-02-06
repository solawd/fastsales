pub const CURRENCY: &str = match option_env!("CURRENCY") {
    Some(c) => c,
    None => "GHS",
};
