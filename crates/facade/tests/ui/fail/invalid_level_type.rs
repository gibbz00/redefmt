enum Level {
    Info,
}

fn main() {
    // Must be redefmt::Level
    redefmt::log!(Level::Info, "");
}
