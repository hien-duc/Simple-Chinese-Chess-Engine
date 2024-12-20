mod board;
mod moves;
mod evaluation;
mod search;
mod uci;

use uci::UCIEngine;
use std::io::{self, Write};

fn main() {
    
    println!("XiangqiEngine starting up...");
    println!("Type 'uci' to initialize the engine");
    println!("Available commands:");
    println!("  uci      - Initialize the engine");
    println!("  isready   - Check if engine is ready");
    println!("  position  - Set up a position");
    println!("  go        - Start calculating");
    println!("  quit      - Exit the engine");
    
    let mut engine = UCIEngine::new();
    
    
    io::stdout().flush().unwrap();
    
    engine.main_loop();
}