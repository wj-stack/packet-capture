use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use ipmb::{label, Options, Message, Selector};
use hook_dll_lib::{HookCommand, TamperRule, PacketMessage};
use std::sync::OnceLock;
use std::sync::Arc;
use std::thread;
#[warn(unused)]
use tauri::Emitter;
use sysinfo::System;

// 进程信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

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

    println!("[Tauri] 发送 StartCapture");

    state.is_capturing = true;
    Ok(())
}

// 非 Windows 平台的占位实现
#[tauri::command]
#[cfg(not(windows))]
async fn start_capture(_app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("[Tauri] 发送 StartCapture");
    Ok(())
}

// 停止抓包命令 - 真实实现
#[tauri::command]
#[cfg(windows)]
async fn stop_capture() -> Result<(), String> {
    let mut state = CAPTURE_STATE.lock().map_err(|e| format!("锁定状态失败: {}", e))?;
    
    if !state.is_capturing {
        return Err("抓包未在运行".to_string());
    }

    println!("[Tauri] 发送 StopCapture");
 
    state.is_capturing = false;
    Ok(())
}

// 非 Windows 平台的占位实现
#[tauri::command]
#[cfg(not(windows))]
async fn stop_capture() -> Result<(), String> {
    println!("[Tauri] 发送 StopCapture");
    Ok(())
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

// 获取进程列表 - 跨平台实现
#[tauri::command]
async fn get_processes() -> Result<Vec<ProcessInfo>, String> {
    let mut system = System::new_all();
    system.refresh_all();
    
    let mut processes: Vec<ProcessInfo> = Vec::new();
    
    for (pid, process) in system.processes() {
        let process_name = process.name().to_string();
        
        // 过滤掉系统进程（可选，根据需求调整）
        // 在 macOS 和 Windows 上，可以根据进程名或路径进行过滤
        
        processes.push(ProcessInfo {
            pid: pid.as_u32(),
            name: process_name,
            icon: None, // 图标功能可以后续实现
        });
    }
    
    // 按进程名排序，便于查找
    processes.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(processes)
}

// DLL 注入命令 - Windows 真实实现
#[tauri::command]
#[cfg(windows)]
async fn inject_dll(process_id: u32) -> Result<String, String> {
    use dll_syringe::{process::OwnedProcess, Syringe};
    use std::path::PathBuf;
    
    println!("[Tauri] 开始注入 DLL 到进程 PID: {}", process_id);
    
    // 打开目标进程
    let target_process = OwnedProcess::from_pid(process_id)
        .map_err(|e| format!("无法打开进程 {}: {}", process_id, e))?;
    
    println!("[Tauri] 成功打开进程 PID: {}", process_id);
    
    // 创建注入器
    let syringe = Syringe::for_process(target_process);
    
    // 获取 DLL 路径
    // 首先尝试从当前可执行文件所在目录查找 DLL
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("无法获取当前可执行文件路径: {}", e))?;
    let exe_dir = exe_path.parent()
        .ok_or("无法获取可执行文件目录")?;
    
    // 尝试多个可能的 DLL 名称
    let dll_names = vec![
        "hook_dll_lib.dll",
        "hook-dll.dll",
        "hook.dll",
    ];
    
    let mut dll_path: Option<PathBuf> = None;
    
    // 首先在当前目录查找
    for dll_name in &dll_names {
        let path = exe_dir.join(dll_name);
        if path.exists() {
            println!("[Tauri] 在当前目录找到 DLL: {:?}", path);
            dll_path = Some(path);
            break;
        }
    }
    
    // 如果当前目录没找到，尝试在 hook-dll 的构建目录查找（开发环境）
    if dll_path.is_none() {
        let hook_dll_path = PathBuf::from("../hook-dll/target/x86_64-pc-windows-msvc/debug/hook_dll_lib.dll");
        if hook_dll_path.exists() {
            dll_path = Some(hook_dll_path);
            println!("[Tauri] 在构建目录找到 DLL: {:?}", dll_path);
        }
    }
    
    // 如果还是没找到，尝试 release 目录
    if dll_path.is_none() {
        let hook_dll_path = PathBuf::from("../hook-dll/target/x86_64-pc-windows-msvc/release/hook_dll_lib.dll");
        if hook_dll_path.exists() {
            dll_path = Some(hook_dll_path);
            println!("[Tauri] 在 release 目录找到 DLL: {:?}", dll_path);
        }
    }
    
    let dll_path = dll_path.ok_or_else(|| {
        format!("无法找到 DLL 文件。请确保以下文件之一存在:\n- {}\n- ../hook-dll/target/x86_64-pc-windows-msvc/debug/hook_dll_lib.dll", 
                dll_names.iter().map(|n| exe_dir.join(n).display().to_string()).collect::<Vec<_>>().join("\n- "))
    })?;
    
    println!("[Tauri] 使用 DLL 路径: {:?}", dll_path);
    
    // 注入 DLL
    let _injected_payload = syringe.inject(&dll_path)
        .map_err(|e| format!("DLL 注入失败: {}", e))?;
    
    println!("[Tauri] DLL 注入成功，进程 ID: {}", process_id);
    
    Ok(format!("DLL 注入成功，进程 ID: {}", process_id))
}

// DLL 注入命令 - 非 Windows 平台 mock 实现
#[tauri::command]
#[cfg(not(windows))]
async fn inject_dll(process_id: u32) -> Result<String, String> {
    println!("[Tauri] Mock DLL 注入到进程 PID: {} (非 Windows 平台)", process_id);
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(format!("Mock DLL 注入成功 (非 Windows 平台)，进程 ID: {}", process_id))
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

// 重放数据包命令 - Windows 实现
#[tauri::command]
#[cfg(windows)]
async fn replay_packet(
    hook_type: String,
    socket: u64,
    data: String, // 十六进制字符串，空格分隔
    dst_addr: Option<String>,
) -> Result<(), String> {
    use hook_dll_lib::HookType;
    
    // 转换 hook_type 字符串到枚举
    let hook_type_enum = match hook_type.as_str() {
        "send" => HookType::Send,
        "sendto" => HookType::SendTo,
        "WSASend" => HookType::WSASend,
        _ => return Err(format!("Unsupported hook type for replay: {}", hook_type)),
    };
    
    // 将十六进制字符串转换为字节数组
    let normalized: String = data.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    
    if normalized.len() % 2 != 0 {
        return Err("十六进制字符串长度必须是偶数".to_string());
    }
    
    let mut bytes = Vec::new();
    let mut idx = 0;
    let normalized_bytes = normalized.as_bytes();
    
    while idx + 1 < normalized_bytes.len() {
        let hex_str = unsafe { std::str::from_utf8_unchecked(&normalized_bytes[idx..idx + 2]) };
        match u8::from_str_radix(hex_str, 16) {
            Ok(byte) => {
                bytes.push(byte);
                idx += 2;
            }
            Err(e) => return Err(format!("无效的十六进制字符: {}", e)),
        }
    }
    
    send_to_dll(HookCommand::ReplayPacket {
        hook_type: hook_type_enum,
        socket,
        data: bytes,
        dst_addr,
    })
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
    println!("[Tauri] 发送 HookSend(_enable: bool)");
    Ok(())
}

#[tauri::command]
#[cfg(not(windows))]
async fn hook_recv(_enable: bool) -> Result<(), String> {
    println!("[Tauri] 发送 HookRecv(_enable: bool)");
    Ok(())
}

#[tauri::command]
#[cfg(not(windows))]
async fn hook_sendto(_enable: bool) -> Result<(), String> {
    println!("[Tauri] 发送 HookSendTo(_enable: bool)");
    Ok(())
}

#[tauri::command]
#[cfg(not(windows))]
async fn hook_recvfrom(_enable: bool) -> Result<(), String> {
    println!("[Tauri] 发送 HookRecvFrom(_enable: bool)");
    Ok(())
}

#[tauri::command]
#[cfg(not(windows))]
async fn hook_wsasend(_enable: bool) -> Result<(), String> {
    println!("[Tauri] 发送 HookWSASend(_enable: bool)");
    Ok(())
}

#[tauri::command]
#[cfg(not(windows))]
async fn hook_wsarecv(_enable: bool) -> Result<(), String> {
    println!("[Tauri] 发送 HookWSARecv(_enable: bool)");
    Ok(())
}

// 重放数据包命令 - 非 Windows 平台占位实现
#[tauri::command]
#[cfg(not(windows))]
async fn replay_packet(
    _hook_type: String,
    _socket: u64,
    _data: String,
    _dst_addr: Option<String>,
) -> Result<(), String> {
    println!("[Tauri] Replay packet (not supported on non-Windows platform)");
    Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {    
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
            replay_packet,
            greet,
            start_capture,
            stop_capture,
            get_capture_status,
            get_processes,
            inject_dll,
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
