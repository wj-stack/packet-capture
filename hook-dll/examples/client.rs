use std::{error::Error, sync::Mutex};

use hook_dll_lib::HookCommand;
use ipmb::{Message, Options, Selector, label};
use std::sync::OnceLock;

static IPMB_SENDER: OnceLock<Mutex<Box<dyn Fn(HookCommand) -> Result<(), String> + Send + Sync>>> = OnceLock::new();

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::new("com.solar.command", label!("moon"), "");
    let (sender, _receiver) = ipmb::join::<HookCommand, HookCommand>(options, None)
        .map_err(|e| format!("初始化 IPMB sender 失败: {}", e))?;
    
    // 创建一个闭包来发送消息
    let sender_fn: Box<dyn Fn(HookCommand) -> Result<(), String> + Send + Sync> = Box::new(move |command| {
        println!("[Tauri] IPMB 发送命令: {:?}", command);
        let selector = Selector::unicast("earth");
        let message = Message::new(selector, command);
        sender.send(message)
            .map_err(|e| format!("发送消息失败: {}", e))?;
        println!("[Tauri] IPMB 命令发送成功");
        Ok(())
    });
    
    IPMB_SENDER.set(Mutex::new(sender_fn))
        .map_err(|_| "IPMB sender 已初始化".to_string())?;
    
    IPMB_SENDER.get().unwrap().lock().unwrap()(HookCommand::Send(true)).unwrap();
    IPMB_SENDER.get().unwrap().lock().unwrap()(HookCommand::Recv(true)).unwrap();
    IPMB_SENDER.get().unwrap().lock().unwrap()(HookCommand::SendTo(true)).unwrap();
    IPMB_SENDER.get().unwrap().lock().unwrap()(HookCommand::RecvFrom(true)).unwrap();
    IPMB_SENDER.get().unwrap().lock().unwrap()(HookCommand::WSASend(true)).unwrap();
    IPMB_SENDER.get().unwrap().lock().unwrap()(HookCommand::WSARecv(true)).unwrap();
    
    Ok(())
}