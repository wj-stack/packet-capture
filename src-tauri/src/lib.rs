use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use ipmb::{label, Options, Message, Selector};
use hook_dll_lib::{HookCommand, TamperRule, PacketMessage};
use std::sync::OnceLock;
use std::sync::Arc;
use std::thread;
#[warn(unused)]
use tauri::Emitter;

// 封包数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 在 Windows 平台的 IPMB receiver 线程中使用
struct Packet {
    id: u64,
    timestamp: u64,
    #[serde(rename = "processId")]
    process_id: u32,
    #[serde(rename = "processName")]
    process_name: String,
    protocol: String,
    direction: String,
    #[serde(rename = "srcAddr")]
    src_addr: String,
    #[serde(rename = "dstAddr")]
    dst_addr: String,
    size: u32,
    socket: Option<u64>,
    #[serde(rename = "packetFunction")]
    packet_function: Option<String>,
    #[serde(rename = "packetData")]
    packet_data: Option<String>,
}

// 抓包状态
struct CaptureState {
    is_capturing: bool,
}

// 全局状态
static CAPTURE_STATE: Mutex<CaptureState> = Mutex::new(CaptureState {
    is_capturing: false,
});

// IPMB Sender 全局变量 - 用于与 DLL 通信
// 使用 Box 存储闭包来避免类型推断问题
static IPMB_SENDER: OnceLock<Mutex<Box<dyn Fn(HookCommand) -> Result<(), String> + Send + Sync>>> = OnceLock::new();

// 初始化 IPMB sender 和 receiver
fn init_ipmb_sender(app_handle: Arc<tauri::AppHandle>) -> Result<(), String> {
    // 初始化发送器（用于发送命令到 DLL）
    // 根据 IPMB 示例，join::<T, T> 表示 Message<T>，payload 也是 T
    // 所以发送 HookCommand 应该使用 join::<HookCommand, HookCommand>
    let options = Options::new("com.solar.command", label!("moon"), "");
    let (sender, _receiver) = ipmb::join::<HookCommand, HookCommand>(options, None)
        .map_err(|e| format!("初始化 IPMB sender 失败: {}", e))?;
    
    // 创建一个闭包来发送消息
    let sender_fn: Box<dyn Fn(HookCommand) -> Result<(), String> + Send + Sync> = Box::new(move |command| {
        println!("[Tauri] IPMB 发送命令: {:?}", command);
        let selector = Selector::unicast("earth");
        let message: Message<HookCommand> = Message::new(selector, command);
        sender.send(message)
            .map_err(|e| format!("发送消息失败: {}", e))?;
        println!("[Tauri] IPMB 命令发送成功");
        Ok(())
    });
    
    IPMB_SENDER.set(Mutex::new(sender_fn))
        .map_err(|_| "IPMB sender 已初始化".to_string())?;
    
    // 初始化接收器（用于接收封包数据）
    // IPMB join 的类型参数：第一个是 Message 的类型（也是 payload 类型）
    // hook-dll 发送 PacketMessage 到 "moon"，所以这里接收 PacketMessage
    // 类型参数需要与发送端匹配：join::<PacketMessage, PacketMessage>
    let packet_options = Options::new("com.solar.capture", label!("moon"), "");
    let (_packet_sender, mut packet_receiver) = ipmb::join::<PacketMessage, PacketMessage>(packet_options, None)
        .map_err(|e| format!("初始化 IPMB packet receiver 失败: {}", e))?;
    
    // 启动线程接收封包数据
    thread::spawn(move || {
        loop {
            match packet_receiver.recv(None) {
                Ok(message) => {
                    // packet_receiver 的类型是 Receiver<PacketMessage>，recv 返回 Message<PacketMessage>
                    match message.payload {
                        PacketMessage::Packet(packet_data) => {
                            // 转换为前端格式并发送事件
                            let frontend_packet = Packet {
                                id: packet_data.id,
                                timestamp: packet_data.timestamp,
                                process_id: packet_data.process_id,
                                process_name: packet_data.process_name,
                                protocol: packet_data.protocol,
                                direction: packet_data.direction,
                                src_addr: packet_data.src_addr,
                                dst_addr: packet_data.dst_addr,
                                size: packet_data.size,
                                socket: packet_data.socket,
                                packet_function: packet_data.packet_function,
                                packet_data: packet_data.packet_data,
                            };
                            
                            let _ = app_handle.emit("packet-captured", &frontend_packet);
                        }
                    }
                }
                Err(e) => {
                    println!("接收封包数据失败: {}", e);
                    break;
                }
            }
        }
    });
    
    Ok(())
}

// 发送消息到 DLL
fn send_to_dll(command: HookCommand) -> Result<(), String> {
    let sender_fn = IPMB_SENDER.get()
        .ok_or("IPMB sender 未初始化")?;
    
    let sender_guard = sender_fn.lock()
        .map_err(|e| format!("锁定 sender 失败: {}", e))?;
    
    sender_guard(command)
}



// 开始抓包命令 - 真实实现
#[tauri::command]
#[cfg(windows)]
async fn start_capture(_app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut state = CAPTURE_STATE.lock().map_err(|e| format!("锁定状态失败: {}", e))?;
    
    if state.is_capturing {
        return Err("抓包已在运行中".to_string());
    }

    // 启用所有网络 hook 以开始捕获封包
    println!("[Tauri] 发送 Send(true)");
    send_to_dll(HookCommand::Send(true)).map_err(|e| format!("发送 Send 命令失败: {}", e))?;
    println!("[Tauri] 发送 Recv(true)");
    send_to_dll(HookCommand::Recv(true)).map_err(|e| format!("发送 Recv 命令失败: {}", e))?;
    println!("[Tauri] 发送 SendTo(true)");
    send_to_dll(HookCommand::SendTo(true)).map_err(|e| format!("发送 SendTo 命令失败: {}", e))?;
    println!("[Tauri] 发送 RecvFrom(true)");
    send_to_dll(HookCommand::RecvFrom(true)).map_err(|e| format!("发送 RecvFrom 命令失败: {}", e))?;
    println!("[Tauri] 发送 WSASend(true)");
    send_to_dll(HookCommand::WSASend(true)).map_err(|e| format!("发送 WSASend 命令失败: {}", e))?;
    println!("[Tauri] 发送 WSARecv(true)");
    send_to_dll(HookCommand::WSARecv(true)).map_err(|e| format!("发送 WSARecv 命令失败: {}", e))?;

    state.is_capturing = true;
    Ok(())
}

// 非 Windows 平台的占位实现
#[tauri::command]
#[cfg(not(windows))]
async fn start_capture(_app_handle: tauri::AppHandle) -> Result<(), String> {
    Err("抓包功能仅在 Windows 平台可用".to_string())
}

// 停止抓包命令 - 真实实现
#[tauri::command]
#[cfg(windows)]
async fn stop_capture() -> Result<(), String> {
    let mut state = CAPTURE_STATE.lock().map_err(|e| format!("锁定状态失败: {}", e))?;
    
    if !state.is_capturing {
        return Err("抓包未在运行".to_string());
    }

    // 禁用所有网络 hook 以停止捕获封包
    println!("[Tauri] 发送 Send(false)");
    send_to_dll(HookCommand::Send(false)).map_err(|e| format!("发送 Send 命令失败: {}", e))?;
    println!("[Tauri] 发送 Recv(false)");
    send_to_dll(HookCommand::Recv(false)).map_err(|e| format!("发送 Recv 命令失败: {}", e))?;
    println!("[Tauri] 发送 SendTo(false)");
    send_to_dll(HookCommand::SendTo(false)).map_err(|e| format!("发送 SendTo 命令失败: {}", e))?;
    println!("[Tauri] 发送 RecvFrom(false)");
    send_to_dll(HookCommand::RecvFrom(false)).map_err(|e| format!("发送 RecvFrom 命令失败: {}", e))?;
    println!("[Tauri] 发送 WSASend(false)");
    send_to_dll(HookCommand::WSASend(false)).map_err(|e| format!("发送 WSASend 命令失败: {}", e))?;
    println!("[Tauri] 发送 WSARecv(false)");
    send_to_dll(HookCommand::WSARecv(false)).map_err(|e| format!("发送 WSARecv 命令失败: {}", e))?;

    state.is_capturing = false;
    Ok(())
}

// 非 Windows 平台的占位实现
#[tauri::command]
#[cfg(not(windows))]
async fn stop_capture() -> Result<(), String> {
    Err("抓包功能仅在 Windows 平台可用".to_string())
}

// 获取抓包状态
#[tauri::command]
async fn get_capture_status() -> Result<String, String> {
    let state = CAPTURE_STATE.lock().map_err(|e| format!("锁定状态失败: {}", e))?;
    Ok(if state.is_capturing {
        "capturing".to_string()
    } else {
        "idle".to_string()
    })
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ========== Hook Command API - 与 DLL 通信 ==========

// Hook 开关命令
#[tauri::command]
#[cfg(windows)]
async fn hook_send(enable: bool) -> Result<(), String> {
    send_to_dll(HookCommand::Send(enable))
}

#[tauri::command]
#[cfg(windows)]
async fn hook_recv(enable: bool) -> Result<(), String> {
    send_to_dll(HookCommand::Recv(enable))
}

#[tauri::command]
#[cfg(windows)]
async fn hook_sendto(enable: bool) -> Result<(), String> {
    send_to_dll(HookCommand::SendTo(enable))
}

#[tauri::command]
#[cfg(windows)]
async fn hook_recvfrom(enable: bool) -> Result<(), String> {
    send_to_dll(HookCommand::RecvFrom(enable))
}

#[tauri::command]
#[cfg(windows)]
async fn hook_wsasend(enable: bool) -> Result<(), String> {
    send_to_dll(HookCommand::WSASend(enable))
}

#[tauri::command]
#[cfg(windows)]
async fn hook_wsarecv(enable: bool) -> Result<(), String> {
    send_to_dll(HookCommand::WSARecv(enable))
}

// TamperRule 管理命令
#[tauri::command]
async fn add_tamper_rule(rule: TamperRule) -> Result<(), String> {
    println!("[Tauri] 发送 AddTamperRule: {:?}", rule);
    send_to_dll(HookCommand::AddTamperRule(rule))
}

#[tauri::command]
async fn remove_tamper_rule(id: String) -> Result<(), String> {
    println!("[Tauri] 发送 RemoveTamperRule: {:?}", id);
    send_to_dll(HookCommand::RemoveTamperRule(id))
}

#[tauri::command]
async fn update_tamper_rule(rule: TamperRule) -> Result<(), String> {
    println!("[Tauri] 发送 UpdateTamperRule: {:?}", rule);
    send_to_dll(HookCommand::UpdateTamperRule(rule))
}

#[tauri::command]
async fn enable_tamper_rule(id: String) -> Result<(), String> {
    println!("[Tauri] 发送 EnableTamperRule: {:?}", id);
    send_to_dll(HookCommand::EnableTamperRule(id))
}

#[tauri::command]
async fn disable_tamper_rule(id: String) -> Result<(), String> {
    println!("[Tauri] 发送 DisableTamperRule: {:?}", id);
    send_to_dll(HookCommand::DisableTamperRule(id))
}

#[tauri::command]
async fn list_tamper_rules() -> Result<(), String> {
    println!("[Tauri] 发送 ListTamperRules");
    send_to_dll(HookCommand::ListTamperRules(()))
}

#[tauri::command]
async fn clear_all_hits() -> Result<(), String> {
    println!("[Tauri] 发送 ClearAllHits");
    send_to_dll(HookCommand::ClearAllHits(()))
}

// 非 Windows 平台的占位实现
#[tauri::command]
#[cfg(not(windows))]
async fn hook_send(_enable: bool) -> Result<(), String> {
    Err("DLL 通信仅在 Windows 平台可用".to_string())
}

#[tauri::command]
#[cfg(not(windows))]
async fn hook_recv(_enable: bool) -> Result<(), String> {
    Err("DLL 通信仅在 Windows 平台可用".to_string())
}

#[tauri::command]
#[cfg(not(windows))]
async fn hook_sendto(_enable: bool) -> Result<(), String> {
    Err("DLL 通信仅在 Windows 平台可用".to_string())
}

#[tauri::command]
#[cfg(not(windows))]
async fn hook_recvfrom(_enable: bool) -> Result<(), String> {
    Err("DLL 通信仅在 Windows 平台可用".to_string())
}

#[tauri::command]
#[cfg(not(windows))]
async fn hook_wsasend(_enable: bool) -> Result<(), String> {
    Err("DLL 通信仅在 Windows 平台可用".to_string())
}

#[tauri::command]
#[cfg(not(windows))]
async fn hook_wsarecv(_enable: bool) -> Result<(), String> {
    Err("DLL 通信仅在 Windows 平台可用".to_string())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化 IPMB sender (仅在 Windows 平台)
    #[cfg(windows)]
    {
        // 创建 AppHandle 的 Arc 引用（需要在 Builder 之后获取）
        // 这里先创建一个临时变量，稍后在 setup hook 中初始化
    }
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 初始化 IPMB sender 和 receiver
            
            let app_handle = Arc::new(app.handle().clone());
            if let Err(e) = init_ipmb_sender(app_handle) {
                println!("警告: IPMB 初始化失败: {}", e);
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_capture,
            stop_capture,
            get_capture_status,
            // Hook 命令
            hook_send,
            hook_recv,
            hook_sendto,
            hook_recvfrom,
            hook_wsasend,
            hook_wsarecv,
            // TamperRule 命令
            add_tamper_rule,
            remove_tamper_rule,
            update_tamper_rule,
            enable_tamper_rule,
            disable_tamper_rule,
            list_tamper_rules,
            clear_all_hits,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
