use std::error::Error;
use std::ffi::c_void;
use std::ptr;
use std::sync::{Mutex, OnceLock};
use ipmb::{MessageBox, label, Options, Message, Selector};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{HINSTANCE, TRUE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::CreateThread;
#[cfg(target_os = "windows")]
use windows_sys::core::BOOL;
use log::*;
use std::io::Write;

pub mod network_hook;
pub mod wildcard;

// IPMB Sender 全局变量 - 用于发送封包数据到 src-tauri
// 使用闭包来避免类型推断问题
static PACKET_SENDER: OnceLock<Mutex<Option<Box<dyn Fn(PacketData) + Send + Sync>>>> = OnceLock::new();

// 初始化封包发送器
fn init_packet_sender() -> Result<(), Box<dyn Error>> {
    let options = Options::new("com.solar.capture", label!("earth"), "");
    // IPMB join 的类型参数：第一个是 Message 的类型（也是 payload 类型），第二个可能是路由类型
    // hook-dll 发送 PacketMessage 到 "moon"（Tauri）
    // 根据示例代码，join::<String, String> 意味着 Message<String>，payload 也是 String
    // 所以这里应该是 join::<PacketMessage, PacketMessage> 或 join::<PacketMessage, String>？
    // 但根据错误信息，sender.send() 期望 Message<第一个类型参数>
    // 所以应该是 join::<PacketMessage, PacketMessage>，Message::new(selector, PacketMessage::Packet(packet))
    let (sender, _receiver) = ipmb::join::<PacketMessage, PacketMessage>(options, None)?;
    
    // 创建一个闭包来发送消息
    let sender_fn: Box<dyn Fn(PacketData) + Send + Sync> = Box::new(move |packet| {
        let selector = Selector::unicast("moon");
        let message = Message::new(selector, PacketMessage::Packet(packet));
        if let Err(e) = sender.send(message) {
            error!("发送封包数据失败: {}", e);
        }
    });
    
    PACKET_SENDER.set(Mutex::new(Some(sender_fn)))
        .map_err(|_| -> Box<dyn Error> { "Packet sender 已初始化".into() })?;
    
    Ok(())
}

// 发送封包数据到 src-tauri
pub fn send_packet_data(packet: PacketData) {
    if let Some(sender_guard) = PACKET_SENDER.get() {
        if let Ok(sender_opt) = sender_guard.lock() {
            if let Some(sender_fn) = sender_opt.as_ref() {
                sender_fn(packet);
            }
        }
    }
}

// 辅助函数：将字节数组转换为十六进制字符串（空格分隔）
fn bytes_to_hex_string(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

// 封包计数器（用于生成唯一 ID）
static PACKET_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(target_os = "windows")]
/// 从 SOCKADDR 解析地址字符串
pub unsafe fn sockaddr_to_string(addr: *const windows_sys::Win32::Networking::WinSock::SOCKADDR) -> Option<String> {
    if addr.is_null() {
        return None;
    }
    
    let sockaddr = *addr;
    if sockaddr.sa_family == windows_sys::Win32::Networking::WinSock::AF_INET as u16 {
        let addr_in = addr as *const windows_sys::Win32::Networking::WinSock::SOCKADDR_IN;
        let ip = u32::from_be((*addr_in).sin_addr.S_un.S_addr);
        let ip_bytes = ip.to_be_bytes();
        let port = u16::from_be((*addr_in).sin_port);
        Some(format!("{}.{}.{}.{}:{}", ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3], port))
    } else if sockaddr.sa_family == windows_sys::Win32::Networking::WinSock::AF_INET6 as u16 {
        // IPv6 支持（简化处理）
        Some("::1:0".to_string())
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
/// 从 SOCKET 获取本地地址（src_addr）
pub unsafe fn get_socket_local_addr(socket: windows_sys::Win32::Networking::WinSock::SOCKET) -> Option<String> {
    use windows_sys::Win32::Networking::WinSock::{SOCKADDR_IN, SOCKADDR, getsockname};
    
    let mut addr: SOCKADDR_IN = std::mem::zeroed();
    let mut addr_len = std::mem::size_of::<SOCKADDR_IN>() as i32;
    
    if getsockname(socket, &mut addr as *mut _ as *mut SOCKADDR, &mut addr_len) == 0 {
        sockaddr_to_string(&addr as *const _ as *const SOCKADDR)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
/// 从 SOCKET 获取远程地址（dst_addr）
pub unsafe fn get_socket_remote_addr(socket: windows_sys::Win32::Networking::WinSock::SOCKET) -> Option<String> {
    use windows_sys::Win32::Networking::WinSock::{SOCKADDR_IN, SOCKADDR, getpeername};
    
    let mut addr: SOCKADDR_IN = std::mem::zeroed();
    let mut addr_len = std::mem::size_of::<SOCKADDR_IN>() as i32;
    
    if getpeername(socket, &mut addr as *mut _ as *mut SOCKADDR, &mut addr_len) == 0 {
        sockaddr_to_string(&addr as *const _ as *const SOCKADDR)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
/// 从 SOCKET 获取协议类型（TCP/UDP）
pub unsafe fn get_socket_protocol(socket: windows_sys::Win32::Networking::WinSock::SOCKET) -> String {
    use windows_sys::Win32::Networking::WinSock::{getsockopt, SOL_SOCKET, SO_TYPE, SOCK_STREAM, SOCK_DGRAM};
    
    let mut protocol: i32 = 0;
    let mut len = std::mem::size_of::<i32>() as i32;
    
    // 获取 socket 类型和协议
    if getsockopt(
        socket,
        SOL_SOCKET as i32,
        SO_TYPE as i32,
        &mut protocol as *mut _ as *mut u8,
        &mut len,
    ) == 0 {
        match protocol {
            SOCK_STREAM => "TCP".to_string(),
            SOCK_DGRAM => "UDP".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    } else {
        // 默认返回 TCP
        "TCP".to_string()
    }
}

#[cfg(not(target_os = "windows"))]
/// 非 Windows 平台的占位函数
pub unsafe fn sockaddr_to_string(_addr: *const std::ffi::c_void) -> Option<String> {
    None
}

#[cfg(not(target_os = "windows"))]
pub unsafe fn get_socket_local_addr(_socket: u64) -> Option<String> {
    None
}

#[cfg(not(target_os = "windows"))]
pub unsafe fn get_socket_remote_addr(_socket: u64) -> Option<String> {
    None
}

#[cfg(not(target_os = "windows"))]
pub unsafe fn get_socket_protocol(_socket: u64) -> String {
    "TCP".to_string()
}

// 捕获并发送封包数据
pub fn capture_and_send_packet(
    data: &[u8],
    hook_type: HookType,
    socket: u64,
    protocol: String,
    src_addr: String,
    dst_addr: String,
) {
    if data.is_empty() {
        return;
    }
    
    let (process_id, process_name) = (std::process::id(), "unknown".to_string());
    let packet_id = PACKET_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    // 确定方向
    let direction = match hook_type {
        HookType::Send | HookType::SendTo | HookType::WSASend => "send",
        HookType::Recv | HookType::RecvFrom | HookType::WSARecv => "receive",
    };
    
    let packet = PacketData {
        id: packet_id,
        timestamp,
        process_id,
        process_name,
        protocol,
        direction: direction.to_string(),
        src_addr,
        dst_addr,
        size: data.len() as u32,
        socket: Some(socket),
        packet_function: Some(format!("{:?}", hook_type)),
        packet_data: Some(bytes_to_hex_string(data)),
    };
    
    send_packet_data(packet);
}



/// TamperRule - 数据包篡改规则
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(TypeUuid)]
#[uuid = "7b07473e-9659-4d47-a502-8245d71c0078"]
pub struct TamperRule {
    pub id: String,
    pub name: String,
    pub match_pattern: String,
    pub replace: String,
    pub action: TamperAction,
    pub active: bool,
    pub hits: u64,
    pub hook: HookType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[derive(TypeUuid)]
#[uuid = "7b07473e-9659-4d47-a502-8245d71c0080"]
pub enum HookType {
    Send,
    Recv,
    SendTo,
    RecvFrom,
    WSASend,
    WSARecv,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[derive(TypeUuid)]
#[uuid = "7b07473e-9659-4d47-a502-8245d71c0081"]
pub enum TamperAction {
    Replace,
    Block,
}


/// DLL入口点
/// 当DLL被加载或卸载时，Windows会调用此函数
#[no_mangle]
#[allow(non_snake_case)]
#[cfg(target_os = "windows")]
pub extern "system" fn DllMain(
    hinst_dll: HINSTANCE,
    fdw_reason: u32,
    _lpv_reserved: *mut c_void,
) -> BOOL {
    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_PROCESS_DETACH: u32 = 0;
    
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
     
            unsafe {
                let thread_handle = CreateThread(
                    ptr::null_mut(),
                    0,
                    Some(worker_thread_proc),
                    hinst_dll,
                    0,
                    ptr::null_mut(),
                );

                debug!("thread_handle: {:?}", thread_handle);
                // if !thread_handle.is_null() {
                //     // 不等待线程，立即关闭句柄
                //     // 线程会继续运行
                //     CloseHandle(thread_handle);
                // }
            }
            debug!("DLL_PROCESS_ATTACH");
        }
        DLL_PROCESS_DETACH => {
            // 清理资源
            debug!("DLL_PROCESS_DETACH");
        }
        _ => {}
    }
    TRUE
}

/// 工作线程过程函数
/// 这个函数在独立的线程中运行，不在 DllMain 的加载器锁中
extern "system" fn worker_thread_proc(_lp_param: *mut c_void) -> u32 {
    // 现在可以安全地执行初始化操作
    perform_business_logic();
    0
}

/// 实际的业务逻辑函数
fn perform_business_logic() {
    // 带时间以及caller 以及文件名和行号
    env_logger::builder().format(|buf, record| {
        writeln!(buf, "{} [{}] {}:{}:{}", record.level(), record.target(), record.file().unwrap(), record.line().unwrap(), record.args())
    }).filter(None, LevelFilter::Info).init();
    std::thread::spawn(|| {
        let result = fmain();
        if let Err(e) = result {
            error!("Error: {}", e);
        }
    });
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(TypeUuid)]
#[uuid = "7b07473e-9659-4d47-a502-8245d71c0083"]
pub enum HookCommand {
    Send(bool),
    Recv(bool),
    SendTo(bool),
    RecvFrom(bool),
    WSASend(bool),
    WSARecv(bool),
    // TamperRule 管理命令 - 使用元组包装以符合 MessageBox 要求
    AddTamperRule(TamperRule),
    RemoveTamperRule(String), // id
    UpdateTamperRule(TamperRule),
    EnableTamperRule(String), // id
    DisableTamperRule(String), // id
    ListTamperRules(()),
    ClearAllHits(()), // 清空所有规则的命中计数
}

/// PacketData - 封包数据消息（从 DLL 发送到 src-tauri）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(TypeUuid)]
#[uuid = "7b07473e-9659-4d47-a502-8245d71c0079"]
pub struct PacketData {
    pub id: u64,
    pub timestamp: u64,
    pub process_id: u32,
    pub process_name: String,
    pub protocol: String,
    pub direction: String,
    pub src_addr: String,
    pub dst_addr: String,
    pub size: u32,
    pub socket: Option<u64>,
    pub packet_function: Option<String>,
    pub packet_data: Option<String>, // 十六进制字符串，空格分隔
}

#[derive(MessageBox)]
#[derive(Debug, Clone)]
pub enum PacketMessage {
    Packet(PacketData),
}

pub fn test_main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().format(|buf, record| {
        writeln!(buf, "{} [{}] {}:{}:{}", record.level(), record.target(), record.file().unwrap(), record.line().unwrap(), record.args())
    }).filter(None, LevelFilter::Info).init();
    fmain()
}

fn fmain () -> Result<(), Box<dyn Error>> {
    use crate::network_hook::*;
    
    // 初始化封包发送器
    init_packet_sender().map_err(|e| format!("Failed to initialize packet sender: {}", e))?;
    
    info!("Packet sender initialized");
    
    let mut manager = NetworkHookManager::new();
    // hook network
    #[cfg(target_os = "windows")]
    {
        use min_hook_rs::initialize;
        info!("Initializing MinHook...");
        initialize()?;    
        let send_hook = SendHook::new()?;
        let recv_hook = RecvHook::new()?;
        let sendto_hook = SendToHook::new()?;
        let recvfrom_hook = RecvFromHook::new()?;
        let wsa_send_hook = WSASendHook::new()?;
        let wsa_recv_hook = WSARecvHook::new()?;
        manager.add_hook(Box::new(send_hook));
        manager.add_hook(Box::new(recv_hook));
        manager.add_hook(Box::new(sendto_hook));
        manager.add_hook(Box::new(recvfrom_hook));
        manager.add_hook(Box::new(wsa_send_hook));
        manager.add_hook(Box::new(wsa_recv_hook));
    }
    manager.enable_all()?;
    
    // 设置全局规则存储
    network_hook::set_global_rules(manager.get_rules());
    
    info!("network hook enabled");

    // Join your bus 
    // 根据 IPMB 示例，join::<T, T> 表示 Message<T>，payload 也是 T
    // 所以接收 HookCommand 应该使用 join::<HookCommand, HookCommand>
    let options = ipmb::Options::new("com.solar.command", label!("earth"), "");
    let (_sender, mut receiver) = ipmb::join::<HookCommand, HookCommand>(options, None).map_err(|e| format!("Failed to join bus: {}", e))?;
    
    // Receive messages
    while let Ok(message) = receiver.recv(None) {
        info!("Received command: {:?}", message.payload);
        match message.payload {
            HookCommand::Send(enable) => {
                if enable {
                    manager.enable_send()?;
                    info!("Send hook enabled");
                } else {
                    manager.disable_send()?;
                    info!("Send hook disabled");
                }
            }
            HookCommand::Recv(enable) => {
                if enable {
                    manager.enable_recv()?;
                    info!("Recv hook enabled");
                } else {
                    manager.disable_recv()?;
                    info!("Recv hook disabled");
                }
            }
            HookCommand::SendTo(enable) => {
                if enable {
                    manager.enable_sendto()?;
                    info!("SendTo hook enabled");
                } else {
                    manager.disable_sendto()?;
                    info!("SendTo hook disabled");
                }
            }
            HookCommand::RecvFrom(enable) => {
                if enable {
                    manager.enable_recvfrom()?;
                    info!("RecvFrom hook enabled");
                } else {
                    manager.disable_recvfrom()?;
                    info!("RecvFrom hook disabled");    
                }
            }
            HookCommand::WSASend(enable) => {
                if enable {
                    manager.enable_wsasend()?;
                    info!("WSASend hook enabled");
                } else {
                    manager.disable_wsasend()?;
                    info!("WSASend hook disabled");
                }
            }
            HookCommand::WSARecv(enable) => {
                if enable {
                    manager.enable_wsarecv()?;
                    info!("WSARecv hook enabled");
                } else {
                    manager.disable_wsarecv()?;
                    info!("WSARecv hook disabled");
                }
            }
            HookCommand::AddTamperRule(rule) => {
                match manager.add_tamper_rule(rule) {
                    Ok(_) => info!("TamperRule added successfully"),
                    Err(e) => error!("Failed to add TamperRule: {}", e),
                }
            }
            HookCommand::RemoveTamperRule(id) => {
                match manager.remove_tamper_rule(&id) {
                    Ok(_) => info!("TamperRule {} removed successfully", id),
                    Err(e) => error!("Failed to remove TamperRule: {}", e),
                }
            }
            HookCommand::UpdateTamperRule(rule) => {
                match manager.update_tamper_rule(rule) {
                    Ok(_) => info!("TamperRule updated successfully"),
                    Err(e) => error!("Failed to update TamperRule: {}", e),
                }
            }
            HookCommand::EnableTamperRule(id) => {
                match manager.enable_tamper_rule(&id) {
                    Ok(_) => info!("TamperRule {} enabled", id),
                    Err(e) => error!("Failed to enable TamperRule: {}", e),
                }
            }
            HookCommand::DisableTamperRule(id) => {
                match manager.disable_tamper_rule(&id) {
                    Ok(_) => info!("TamperRule {} disabled", id),
                    Err(e) => error!("Failed to disable TamperRule: {}", e),
                }
            }
            HookCommand::ListTamperRules(_) => {
                match manager.list_tamper_rules() {
                    Ok(rules) => {
                        info!("Total TamperRules: {}", rules.len());
                        for rule in rules {
                            info!("  - {}: {} (active: {}, hits: {})", 
                                rule.id, rule.name, rule.active, rule.hits);
                        }
                    }
                    Err(e) => error!("Failed to list TamperRules: {}", e),
                }
            }
            HookCommand::ClearAllHits(_) => {
                match manager.clear_all_hits() {
                    Ok(_) => info!("All rule hits cleared successfully"),
                    Err(e) => error!("Failed to clear all hits: {}", e),
                }
            }
        }
    }
   
    Ok(())
}

