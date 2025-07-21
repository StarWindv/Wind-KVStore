use chrono::Local;

fn main() {
    // 获取当前本地时间
    let now = Local::now();
    let build_time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // 将编译时间写入环境变量，供主程序使用
    println!("cargo::rustc-env=BUILD_TIME={}", build_time);

    // 告诉Cargo如果build.rs改变了需要重新编译
    println!("cargo::rerun-if-changed=build.rs");
}
