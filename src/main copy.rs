// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


fn main() {
    let doc = flock_lib::parser::parser::Document::new("test.txt");
    doc.write("out.json", "defs.json");

    //flock_lib::run()
}
