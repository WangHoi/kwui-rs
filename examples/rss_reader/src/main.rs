#![cfg_attr(all(target_os = "windows", not(test), not(debug_assertions)), windows_subsystem = "windows")]

use rss_reader;

fn main() {
    rss_reader::entry();
}
