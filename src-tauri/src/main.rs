// 发布构建时隐藏控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    checkin_exe_lib::run()
}
