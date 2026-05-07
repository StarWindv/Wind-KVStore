use chrono::Local;

fn main() {
    let now = Local::now();
    let build_time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    println!("cargo::rustc-env=BUILD_TIME={}", build_time);

    println!("cargo::rerun-if-changed=build.rs");
}
