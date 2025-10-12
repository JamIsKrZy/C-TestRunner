use std::collections::HashMap;

use maplit::{btreemap, hashmap};
use termion::{color, cursor, style};

use crate::record_collection::{StatusType, collection::CompiledRecord};

pub fn init_test_collection() -> CompiledRecord {
    CompiledRecord {
        test_tree: btreemap! {
            "Program1".to_string() => hashmap! {
                "Addtion".to_string() => 0,
                "Subtration".to_string() => 1
            },
            "Minecrafters".to_string() => hashmap!{
                "break block".to_string() => 2
            }
        },
        test_status: vec![StatusType::Success, StatusType::Fail, StatusType::Success],
        test_logs: vec![None, None, None],
    }
}

pub fn sample_table() {
    print!(
        "{}{}{}",
        cursor::Hide,
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    );

    // 50 - width
    const constant_width: usize = 50;
    let left_c: usize = (constant_width as f32 * 0.70).round() as usize;
    let mut right_c: usize = (constant_width as f32 * 0.30).round() as usize;

    if left_c + right_c < constant_width {
        right_c = constant_width - (left_c + right_c);
    }

    println!("┌{}┐", "─".repeat(constant_width));
    println!(
        "│{}{:<constant_width$}{}│",
        style::Bold,
        "Header",
        style::Reset
    );
    println!("├{}┤", "─".repeat(constant_width));
    println!("│{:<left_c$}{:<right_c$}│", "Program Test", "4/10");
    println!(
        "│{}{:<left_c$}{:<right_c$}{}│",
        color::Bg(color::Blue),
        "Program Test",
        "4/10",
        color::Bg(color::Reset)
    );
    println!("│{:<left_c$}{:<right_c$}│", "Program Test", "4/10");
    println!("└{}┘", "─".repeat(constant_width));
}
