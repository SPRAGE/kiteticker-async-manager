use kiteticker_async_manager::{Mode, Request};

fn main() {
    println!("🔍 KiteTicker Mode Serialization Test");
    println!("═══════════════════════════════════════");
    
    // Test different subscription requests to verify JSON format
    let tokens = vec![256265, 265, 256777];
    
    // 1. Test subscribe request
    let subscribe_req = Request::subscribe(tokens.clone());
    println!("📡 Subscribe request JSON:");
    println!("{}", subscribe_req);
    println!();
    
    // 2. Test unsubscribe request
    let unsubscribe_req = Request::unsubscribe(tokens.clone());
    println!("📡 Unsubscribe request JSON:");
    println!("{}", unsubscribe_req);
    println!();
    
    // 3. Test mode requests with different modes
    let modes = [Mode::LTP, Mode::Quote, Mode::Full];
    let mode_names = ["LTP", "Quote", "Full"];
    
    for (mode, name) in modes.iter().zip(mode_names.iter()) {
        let mode_req = Request::mode(*mode, tokens.clone());
        println!("🎯 Mode request for {} ({}): JSON:", name, mode.to_websocket_string());
        println!("{}", mode_req);
        println!();
    }
    
    // 4. Test individual mode string conversion
    println!("🔧 Mode to WebSocket string conversion:");
    for (mode, name) in modes.iter().zip(mode_names.iter()) {
        println!("   {} -> \"{}\"", name, mode.to_websocket_string());
    }
    
    println!("✅ All mode serialization tests completed!");
}
