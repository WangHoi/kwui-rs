#![cfg_attr(all(target_os = "windows", not(test), not(debug_assertions)), windows_subsystem = "windows")]

use installer;

fn main() {
    installer::entry();
}
