use std::{error::Error, sync::Mutex};

use hook_dll_lib::HookCommand;
use ipmb::{Message, Options, Selector, label};
use std::sync::OnceLock;
fn main () -> Result<(), Box<dyn Error>> {
        // Join your bus 
    // 根据 IPMB 示例，join::<T, T> 表示 Message<T>，payload 也是 T
    // 所以接收 HookCommand 应该使用 join::<HookCommand, HookCommand>
    let options = ipmb::Options::new("com.solar.command", label!("earth"), "");
    let (_sender, mut receiver) = ipmb::join::<HookCommand, HookCommand>(options, None).map_err(|e| format!("Failed to join bus: {}", e))?;
    while let Ok(message) = receiver.recv(None) {
        println!("Received command: {:?}", message.payload);
    }
    Ok(())
    // hook_dll_lib::test_main()
}

