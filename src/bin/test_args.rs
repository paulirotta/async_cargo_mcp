//! Test binary for validating binary argument functionality

use std::env;

fn main() {
    println!("Test binary started!");
    
    let args: Vec<String> = env::args().collect();
    println!("Received {} arguments:", args.len() - 1);
    
    for (i, arg) in args.iter().skip(1).enumerate() {
        println!("  arg[{i}]: {arg}");
    }
    
    // Special behavior for testing
    if args.len() > 1 && args[1] == "--test-mode" {
        println!("TEST_MODE_ACTIVATED");
    }
    
    if args.len() > 2 && args[2] == "--special" {
        println!("SPECIAL_FLAG_DETECTED");
    }
}
