use std::error::Error;
use std::ffi::c_void;
use std::ptr;
use ipmb::{MessageBox, label};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use windows_sys::Win32::Foundation::{HINSTANCE, TRUE};
use windows_sys::Win32::System::Threading::CreateThread;
use windows_sys::core::BOOL;
use log::*;

pub mod network_hook;
pub mod wildcard;



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
    env_logger::init();
    std::thread::spawn(|| {
        let result = fmain();
        if let Err(e) = result {
            error!("Error: {}", e);
        }
    });
}


#[derive(MessageBox)]
#[derive(Debug, Clone)]
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

fn fmain () -> Result<(), Box<dyn Error>> {
    use crate::network_hook::*;
    
    let mut manager = NetworkHookManager::new();
    // hook network
    #[cfg(target_os = "windows")]
    {
        let send_hook: SendHook = SendHook::new()?;
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
    let options = ipmb::Options::new("com.solar", label!("earth"), "");
    let (_sender, mut receiver) = ipmb::join::<String, HookCommand>(options, None)?;

    // Receive messages
    while let Ok(message) = receiver.recv(None) {
        match message.payload {
            HookCommand::Send(enable) => {
                if enable {
                    manager.enable_send()?;
                } else {
                    manager.disable_send()?;
                }
            }
            HookCommand::Recv(enable) => {
                if enable {
                    manager.enable_recv()?;
                } else {
                    manager.disable_recv()?;
                }
            }
            HookCommand::SendTo(enable) => {
                if enable {
                    manager.enable_sendto()?;
                } else {
                    manager.disable_sendto()?;
                }
            }
            HookCommand::RecvFrom(enable) => {
                if enable {
                    manager.enable_recvfrom()?;
                } else {
                    manager.disable_recvfrom()?;
                }
            }
            HookCommand::WSASend(enable) => {
                if enable {
                    manager.enable_wsasend()?;
                } else {
                    manager.disable_wsasend()?;
                }
            }
            HookCommand::WSARecv(enable) => {
                if enable {
                    manager.enable_wsarecv()?;
                } else {
                    manager.disable_wsarecv()?;
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

