mod network_hook {
    
    #[cfg(target_os = "windows")]
    use min_hook_rs::*;
    
    use core::option::Option::None;
    use std::ffi::c_void;
    use std::ptr;
    use windows_sys::Win32::Networking::WinSock::*;
    use windows_sys::Win32::System::IO::OVERLAPPED;
    use std::sync::{Arc, Mutex, OnceLock};
    use log::*;
    use crate::{HookType, TamperAction, TamperRule};
    use crate::wildcard::wildcard_match;
    
    // ========== 回调类型定义 ==========

    /// 数据包处理结果
    #[derive(Debug, Clone)]
    pub enum PacketAction {
        /// 允许通过，使用原始数据
        Allow,
        /// 阻止数据包
        Block,
        /// 替换数据包内容
        Replace(Vec<u8>),
    }

    /// Send 回调函数类型
    /// 参数: (socket, 数据指针, 数据长度, flags)
    /// 返回: PacketAction - 决定如何处理数据包
    pub type SendCallback = dyn Fn(SOCKET, &[u8], i32) -> PacketAction + Send + Sync;

    /// Recv 回调函数类型
    /// 参数: (socket, 接收到的数据)
    /// 返回: PacketAction - 决定如何处理数据包
    pub type RecvCallback = dyn Fn(SOCKET, &[u8]) -> PacketAction + Send + Sync;

    /// SendTo 回调函数类型
    /// 参数: (socket, 数据指针, 数据长度, flags, 目标地址)
    pub type SendToCallback = dyn Fn(SOCKET, &[u8], i32, *const SOCKADDR) -> PacketAction + Send + Sync;

    /// RecvFrom 回调函数类型
    /// 参数: (socket, 接收到的数据, 来源地址)
    pub type RecvFromCallback = dyn Fn(SOCKET, &[u8], *const SOCKADDR) -> PacketAction + Send + Sync;

    /// WSASend 回调函数类型
    pub type WSASendCallback = dyn Fn(SOCKET, &[WSABUF]) -> PacketAction + Send + Sync;

    /// WSARecv 回调函数类型
    pub type WSARecvCallback = dyn Fn(SOCKET, &[u8]) -> PacketAction + Send + Sync;

    /// Hook trait - 定义所有 hook 的通用接口
    pub trait NetworkHook {
        /// Hook 的名称，用于日志和调试
        fn name(&self) -> &'static str;

        /// 启用 hook
        fn enable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>>;

        /// 禁用 hook
        fn disable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>>;

        /// 检查 hook 是否已启用
        fn is_enabled(&self) -> bool;

        /// 清理资源
        fn cleanup(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>>;
    }

    /// Send Hook - Hook send 函数
    #[cfg(target_os = "windows")]
    pub struct SendHook {
        target: *mut c_void,
        enabled: bool,
        callback: Option<Arc<Mutex<Box<SendCallback>>>>,
    }

    #[cfg(target_os = "windows")]
    impl SendHook {
        pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                target: ptr::null_mut(),
                enabled: false,
                callback: None,
            })
        }

        /// 设置回调函数
        pub fn set_callback<F>(&mut self, callback: F)
        where
            F: Fn(SOCKET, &[u8], i32) -> PacketAction + Send + Sync + 'static,
        {
            self.callback = Some(Arc::new(Mutex::new(Box::new(callback))));
        }

    }

    #[cfg(target_os = "windows")]
    impl NetworkHook for SendHook {
        fn name(&self) -> &'static str {
            "Send"
        }

        fn enable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                return Ok(());
            }

            let (trampoline, target) =
                create_hook_api("ws2_32", "send", hooked_send as *mut c_void)?;
            
            unsafe {
                ORIGINAL_SEND = Some(std::mem::transmute(trampoline));
                // 注册回调
                let _ = SEND_CALLBACK.set(self.callback.clone());
            }

            enable_hook(target)?;
            self.target = target;
            self.enabled = true;
            debug!("[HOOK] {} hook enabled", self.name());
            Ok(())
        }

        fn disable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if !self.enabled {
                return Ok(());
            }

            disable_hook(self.target)?;
            self.enabled = false;
            debug!("[HOOK] {} hook disabled", self.name());
            Ok(())
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn cleanup(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                self.disable()?;
            }
            if !self.target.is_null() {
                remove_hook(self.target)?;
                self.target = ptr::null_mut();
            }
            Ok(())
        }
    }

    /// SendTo Hook - Hook sendto 函数
    #[cfg(target_os = "windows")]
    pub struct SendToHook {
        target: *mut c_void,
        enabled: bool,
        callback: Option<Arc<Mutex<Box<SendToCallback>>>>,
    }

    #[cfg(target_os = "windows")]
    impl SendToHook {
        pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                target: ptr::null_mut(),
                enabled: false,
                callback: None,
            })
        }

        /// 设置回调函数
        pub fn set_callback<F>(&mut self, callback: F)
        where
            F: Fn(SOCKET, &[u8], i32, *const SOCKADDR) -> PacketAction + Send + Sync + 'static,
        {
            self.callback = Some(Arc::new(Mutex::new(Box::new(callback))));
        }
    }

    #[cfg(target_os = "windows")]
    impl NetworkHook for SendToHook {
        fn name(&self) -> &'static str {
            "SendTo"
        }

        fn enable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                return Ok(());
            }

            let (trampoline, target) =
                create_hook_api("ws2_32", "sendto", hooked_sendto as *mut c_void)?;
            
            unsafe {
                ORIGINAL_SENDTO = Some(std::mem::transmute(trampoline));
                let _ = SENDTO_CALLBACK.set(self.callback.clone());
            }

            enable_hook(target)?;
            self.target = target;
            self.enabled = true;
            debug!("[HOOK] {} hook enabled", self.name());
            Ok(())
        }

        fn disable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if !self.enabled {
                return Ok(());
            }

            disable_hook(self.target)?;
            self.enabled = false;
            debug!("[HOOK] {} hook disabled", self.name());
            Ok(())
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn cleanup(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                self.disable()?;
            }
            if !self.target.is_null() {
                remove_hook(self.target)?;
                self.target = ptr::null_mut();
            }
            Ok(())
        }
    }

    /// Recv Hook - Hook recv 函数
    #[cfg(target_os = "windows")]
    pub struct RecvHook {
        target: *mut c_void,
        enabled: bool,
        callback: Option<Arc<Mutex<Box<RecvCallback>>>>,
    }

    #[cfg(target_os = "windows")]
    impl RecvHook {
        pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                target: ptr::null_mut(),
                enabled: false,
                callback: None,
            })
        }

        /// 设置回调函数
        pub fn set_callback<F>(&mut self, callback: F)
        where
            F: Fn(SOCKET, &[u8]) -> PacketAction + Send + Sync + 'static,
        {
            self.callback = Some(Arc::new(Mutex::new(Box::new(callback))));
        }
    }

    #[cfg(target_os = "windows")]
    impl NetworkHook for RecvHook {
        fn name(&self) -> &'static str {
            "Recv"
        }

        fn enable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                return Ok(());
            }

            let (trampoline, target) =
                create_hook_api("ws2_32", "recv", hooked_recv as *mut c_void)?;
            
            unsafe {
                ORIGINAL_RECV = Some(std::mem::transmute(trampoline));
                let _ = RECV_CALLBACK.set(self.callback.clone());
            }

            enable_hook(target)?;
            self.target = target;
            self.enabled = true;
            debug!("[HOOK] {} hook enabled", self.name());
            Ok(())
        }

        fn disable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if !self.enabled {
                return Ok(());
            }

            disable_hook(self.target)?;
            self.enabled = false;
            debug!("[HOOK] {} hook disabled", self.name());
            Ok(())
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn cleanup(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                self.disable()?;
            }
            if !self.target.is_null() {
                remove_hook(self.target)?;
                self.target = ptr::null_mut();
            }
            Ok(())
        }
    }

    /// RecvFrom Hook - Hook recvfrom 函数
    #[cfg(target_os = "windows")]
    pub struct RecvFromHook {
        target: *mut c_void,
        enabled: bool,
        callback: Option<Arc<Mutex<Box<RecvFromCallback>>>>,
    }

    #[cfg(target_os = "windows")]
    impl RecvFromHook {
        pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                target: ptr::null_mut(),
                enabled: false,
                callback: None,
            })
        }

        /// 设置回调函数
        pub fn set_callback<F>(&mut self, callback: F)
        where
            F: Fn(SOCKET, &[u8], *const SOCKADDR) -> PacketAction + Send + Sync + 'static,
        {
            self.callback = Some(Arc::new(Mutex::new(Box::new(callback))));
        }
    }

    #[cfg(target_os = "windows")]
    impl NetworkHook for RecvFromHook {
        fn name(&self) -> &'static str {
            "RecvFrom"
        }

        fn enable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                return Ok(());
            }

            let (trampoline, target) =
                create_hook_api("ws2_32", "recvfrom", hooked_recvfrom as *mut c_void)?;
            
            unsafe {
                ORIGINAL_RECVFROM = Some(std::mem::transmute(trampoline));
                let _ = RECVFROM_CALLBACK.set(self.callback.clone());
            }

            enable_hook(target)?;
            self.target = target;
            self.enabled = true;
            debug!("[HOOK] {} hook enabled", self.name());
            Ok(())
        }

        fn disable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if !self.enabled {
                return Ok(());
            }

            disable_hook(self.target)?;
            self.enabled = false;
            debug!("[HOOK] {} hook disabled", self.name());
            Ok(())
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn cleanup(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                self.disable()?;
            }
            if !self.target.is_null() {
                remove_hook(self.target)?;
                self.target = ptr::null_mut();
            }
            Ok(())
        }
    }

    /// WSASend Hook - Hook WSASend 函数
    #[cfg(target_os = "windows")]
    pub struct WSASendHook {
        target: *mut c_void,
        enabled: bool,
        callback: Option<Arc<Mutex<Box<WSASendCallback>>>>,
    }

    #[cfg(target_os = "windows")]
    impl WSASendHook {
        pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                target: ptr::null_mut(),
                enabled: false,
                callback: None,
            })
        }

        /// 设置回调函数
        pub fn set_callback<F>(&mut self, callback: F)
        where
            F: Fn(SOCKET, &[WSABUF]) -> PacketAction + Send + Sync + 'static,
        {
            self.callback = Some(Arc::new(Mutex::new(Box::new(callback))));
        }
    }

    #[cfg(target_os = "windows")]
    impl NetworkHook for WSASendHook {
        fn name(&self) -> &'static str {
            "WSASend"
        }

        fn enable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                return Ok(());
            }

            let (trampoline, target) =
                create_hook_api("ws2_32", "WSASend", hooked_wsasend as *mut c_void)?;
            
            unsafe {
                ORIGINAL_WSASEND = Some(std::mem::transmute(trampoline));
                let _ = WSASEND_CALLBACK.set(self.callback.clone());
            }

            enable_hook(target)?;
            self.target = target;
            self.enabled = true;
            debug!("[HOOK] {} hook enabled", self.name());
            Ok(())
        }

        fn disable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if !self.enabled {
                return Ok(());
            }

            disable_hook(self.target)?;
            self.enabled = false;
            debug!("[HOOK] {} hook disabled", self.name());
            Ok(())
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn cleanup(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                self.disable()?;
            }
            if !self.target.is_null() {
                remove_hook(self.target)?;
                self.target = ptr::null_mut();
            }
            Ok(())
        }
    }

    /// WSARecv Hook - Hook WSARecv 函数
    #[cfg(target_os = "windows")]
    pub struct WSARecvHook {
        target: *mut c_void,
        enabled: bool,
        callback: Option<Arc<Mutex<Box<WSARecvCallback>>>>,
    }

    #[cfg(target_os = "windows")]
    impl WSARecvHook {
        pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                target: ptr::null_mut(),
                enabled: false,
                callback: None,
            })
        }

        /// 设置回调函数
        pub fn set_callback<F>(&mut self, callback: F)
        where
            F: Fn(SOCKET, &[u8]) -> PacketAction + Send + Sync + 'static,
        {
            self.callback = Some(Arc::new(Mutex::new(Box::new(callback))));
        }
    }

    #[cfg(target_os = "windows")]
    impl NetworkHook for WSARecvHook {
        fn name(&self) -> &'static str {
            "WSARecv"
        }

        fn enable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                return Ok(());
            }

            let (trampoline, target) =
                create_hook_api("ws2_32", "WSARecv", hooked_wsarecv as *mut c_void)?;
            
            unsafe {
                ORIGINAL_WSARECV = Some(std::mem::transmute(trampoline));
                let _ = WSARECV_CALLBACK.set(self.callback.clone());
            }

            enable_hook(target)?;
            self.target = target;
            self.enabled = true;
            debug!("[HOOK] {} hook enabled", self.name());
            Ok(())
        }

        fn disable(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if !self.enabled {
                return Ok(());
            }

            disable_hook(self.target)?;
            self.enabled = false;
            debug!("[HOOK] {} hook disabled", self.name());
            Ok(())
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn cleanup(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if self.enabled {
                self.disable()?;
            }
            if !self.target.is_null() {
                remove_hook(self.target)?;
                self.target = ptr::null_mut();
            }
            Ok(())
        }
    }

    /// Hook 管理器 - 统一管理所有网络 hook
    pub struct NetworkHookManager {
        hooks: Vec<Box<dyn NetworkHook>>,
        send_hook_index: Option<usize>,
        recv_hook_index: Option<usize>,
        sendto_hook_index: Option<usize>,
        recvfrom_hook_index: Option<usize>,
        wsa_send_hook_index: Option<usize>,
        wsa_recv_hook_index: Option<usize>,
        rules: Arc<Mutex<Vec<TamperRule>>>,
    }

    impl NetworkHookManager {
        pub fn new() -> Self {
            Self {
                hooks: Vec::new(),
                send_hook_index: None,
                recv_hook_index: None,
                sendto_hook_index: None,
                recvfrom_hook_index: None,
                wsa_send_hook_index: None,
                wsa_recv_hook_index: None,
                rules: Arc::new(Mutex::new(Vec::new())),
            }
        }

        /// 添加 TamperRule
        pub fn add_tamper_rule(&self, rule: TamperRule) -> std::result::Result<(), Box<dyn std::error::Error>> {
            let mut rules = self.rules.lock().map_err(|e| format!("Failed to lock rules: {}", e))?;
            // 检查是否已存在相同 ID 的规则
            if rules.iter().any(|r| r.id == rule.id) {
                return Err("Rule with same ID already exists".into());
            }
            rules.push(rule);
            Ok(())
        }

        /// 删除 TamperRule
        pub fn remove_tamper_rule(&self, id: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
            let mut rules = self.rules.lock().map_err(|e| format!("Failed to lock rules: {}", e))?;
            rules.retain(|r| r.id != id);
            Ok(())
        }

        /// 更新 TamperRule
        pub fn update_tamper_rule(&self, rule: TamperRule) -> std::result::Result<(), Box<dyn std::error::Error>> {
            let mut rules = self.rules.lock().map_err(|e| format!("Failed to lock rules: {}", e))?;
            if let Some(existing_rule) = rules.iter_mut().find(|r| r.id == rule.id) {
                *existing_rule = rule;
                Ok(())
            } else {
                Err("Rule not found".into())
            }
        }

        /// 启用 TamperRule
        pub fn enable_tamper_rule(&self, id: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
            let mut rules = self.rules.lock().map_err(|e| format!("Failed to lock rules: {}", e))?;
            if let Some(rule) = rules.iter_mut().find(|r| r.id == id) {
                rule.active = true;
                Ok(())
            } else {
                Err("Rule not found".into())
            }
        }

        /// 禁用 TamperRule
        pub fn disable_tamper_rule(&self, id: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
            let mut rules = self.rules.lock().map_err(|e| format!("Failed to lock rules: {}", e))?;
            if let Some(rule) = rules.iter_mut().find(|r| r.id == id) {
                rule.active = false;
                Ok(())
            } else {
                Err("Rule not found".into())
            }
        }

        /// 列出所有 TamperRule
        pub fn list_tamper_rules(&self) -> std::result::Result<Vec<TamperRule>, Box<dyn std::error::Error>> {
            let rules = self.rules.lock().map_err(|e| format!("Failed to lock rules: {}", e))?;
            Ok(rules.clone())
        }

        /// 清空所有规则的命中计数
        pub fn clear_all_hits(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            let mut rules = self.rules.lock().map_err(|e| format!("Failed to lock rules: {}", e))?;
            for rule in rules.iter_mut() {
                rule.hits = 0;
            }
            Ok(())
        }

        /// 获取规则列表的 Arc 引用（用于 hook 回调）
        pub fn get_rules(&self) -> Arc<Mutex<Vec<TamperRule>>> {
            self.rules.clone()
        }

        /// 应用规则到数据包
        /// 返回匹配的规则和对应的动作
        pub fn apply_rules(data: &[u8], hook_type: HookType, rules: &[TamperRule]) -> Option<(usize, PacketAction)> {
            for (idx, rule) in rules.iter().enumerate() {
                // 只处理激活的规则
                if !rule.active {
                    continue;
                }
                
                // 检查规则是否适用于当前 hook 类型
                if rule.hook != hook_type {
                    continue;
                }

                // 使用问号通配符匹配
                if wildcard_match(&rule.match_pattern, data) {
                    // 匹配成功，根据动作类型返回结果
                    match rule.action {
                        TamperAction::Block => {
                            return Some((idx, PacketAction::Block));
                        }
                        TamperAction::Replace => {
                            let replace_bytes = rule.replace.as_bytes().to_vec();
                            return Some((idx, PacketAction::Replace(replace_bytes)));
                        }
                    }
                }
            }
            None
        }

        /// 添加一个 hook
        pub fn add_hook(&mut self, hook: Box<dyn NetworkHook>) {
            let name = hook.name();
            let index = self.hooks.len();
            
            match name {
                "Send" => self.send_hook_index = Some(index),
                "Recv" => self.recv_hook_index = Some(index),
                "SendTo" => self.sendto_hook_index = Some(index),
                "RecvFrom" => self.recvfrom_hook_index = Some(index),
                "WSASend" => self.wsa_send_hook_index = Some(index),
                "WSARecv" => self.wsa_recv_hook_index = Some(index),
                _ => {}
            }
            
            self.hooks.push(hook);
        }

        /// 启用所有 hook
        pub fn enable_all(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            for hook in &mut self.hooks {
                hook.enable()?;
            }
            Ok(())
        }

        /// 禁用所有 hook
        pub fn disable_all(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            for hook in &mut self.hooks {
                hook.disable()?;
            }
            Ok(())
        }

        /// 清理所有资源
        pub fn cleanup_all(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            for hook in &mut self.hooks {
                hook.cleanup()?;
            }
            self.hooks.clear();
            Ok(())
        }

        /// 启用 Send hook
        pub fn enable_send(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.send_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.enable()?;
                }
            }
            Ok(())
        }

        /// 禁用 Send hook
        pub fn disable_send(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.send_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.disable()?;
                }
            }
            Ok(())
        }

        /// 启用 Recv hook
        pub fn enable_recv(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.recv_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.enable()?;
                }
            }
            Ok(())
        }

        /// 禁用 Recv hook
        pub fn disable_recv(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.recv_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.disable()?;
                }
            }
            Ok(())
        }

        /// 启用 SendTo hook
        pub fn enable_sendto(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.sendto_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.enable()?;
                }
            }
            Ok(())
        }

        /// 禁用 SendTo hook
        pub fn disable_sendto(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.sendto_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.disable()?;
                }
            }
            Ok(())
        }

        /// 启用 RecvFrom hook
        pub fn enable_recvfrom(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.recvfrom_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.enable()?;
                }
            }
            Ok(())
        }

        /// 禁用 RecvFrom hook
        pub fn disable_recvfrom(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.recvfrom_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.disable()?;
                }
            }
            Ok(())
        }

        /// 启用 WSASend hook
        pub fn enable_wsasend(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.wsa_send_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.enable()?;
                }
            }
            Ok(())
        }

        /// 禁用 WSASend hook
        pub fn disable_wsasend(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.wsa_send_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.disable()?;
                }
            }
            Ok(())
        }

        /// 启用 WSARecv hook
        pub fn enable_wsarecv(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.wsa_recv_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.enable()?;
                }
            }
            Ok(())
        }

        /// 禁用 WSARecv hook
        pub fn disable_wsarecv(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.wsa_recv_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    hook.disable()?;
                }
            }
            Ok(())
        }
    }

    // ========== Hook 函数实现 ==========

    // 原始函数指针存储
    type SendFn = unsafe extern "system" fn(SOCKET, *const u8, i32, i32) -> i32;
    type SendToFn = unsafe extern "system" fn(SOCKET, *const u8, i32, i32, *const SOCKADDR, i32) -> i32;
    type RecvFn = unsafe extern "system" fn(SOCKET, *mut u8, i32, i32) -> i32;
    type RecvFromFn = unsafe extern "system" fn(SOCKET, *mut u8, i32, i32, *mut SOCKADDR, *mut i32) -> i32;
    type WSASendFn = unsafe extern "system" fn(SOCKET, *const WSABUF, u32, *mut u32, u32, *mut OVERLAPPED, Option<unsafe extern "system" fn(*mut OVERLAPPED, u32, u32, *mut c_void)>) -> i32;
    type WSARecvFn = unsafe extern "system" fn(SOCKET, *mut WSABUF, u32, *mut u32, *mut u32, *mut OVERLAPPED, Option<unsafe extern "system" fn(*mut OVERLAPPED, u32, u32, *mut c_void)>) -> i32;

    static mut ORIGINAL_SEND: Option<SendFn> = None;
    static mut ORIGINAL_SENDTO: Option<SendToFn> = None;
    static mut ORIGINAL_RECV: Option<RecvFn> = None;
    static mut ORIGINAL_RECVFROM: Option<RecvFromFn> = None;
    static mut ORIGINAL_WSASEND: Option<WSASendFn> = None;
    static mut ORIGINAL_WSARECV: Option<WSARecvFn> = None;

    // 全局回调存储 - 使用 OnceLock 避免 static mut 的警告
    static SEND_CALLBACK: OnceLock<Option<Arc<Mutex<Box<SendCallback>>>>> = OnceLock::new();
    static SENDTO_CALLBACK: OnceLock<Option<Arc<Mutex<Box<SendToCallback>>>>> = OnceLock::new();
    static RECV_CALLBACK: OnceLock<Option<Arc<Mutex<Box<RecvCallback>>>>> = OnceLock::new();
    static RECVFROM_CALLBACK: OnceLock<Option<Arc<Mutex<Box<RecvFromCallback>>>>> = OnceLock::new();
    static WSASEND_CALLBACK: OnceLock<Option<Arc<Mutex<Box<WSASendCallback>>>>> = OnceLock::new();
    static WSARECV_CALLBACK: OnceLock<Option<Arc<Mutex<Box<WSARecvCallback>>>>> = OnceLock::new();
    
    // 全局规则存储
    static TAMPER_RULES: OnceLock<Arc<Mutex<Vec<TamperRule>>>> = OnceLock::new();
    
    /// 设置全局规则存储
    pub fn set_global_rules(rules: Arc<Mutex<Vec<TamperRule>>>) {
        let _ = TAMPER_RULES.set(rules);
    }
    
    /// 应用规则到数据包（内部辅助函数）
    fn apply_tamper_rules(data: &[u8], hook_type: HookType) -> Option<PacketAction> {
        if let Some(rules_arc) = TAMPER_RULES.get() {
            if let Ok(mut rules) = rules_arc.lock() {
                for rule in rules.iter_mut() {
                    if !rule.active {
                        continue;
                    }
                    if rule.hook != hook_type {
                        continue;
                    }
                    if wildcard_match(&rule.match_pattern, data) {
                        // 增加命中计数
                        rule.hits += 1;
                        
                        match rule.action {
                            TamperAction::Block => {
                                return Some(PacketAction::Block);
                            }
                            TamperAction::Replace => {
                                let replace_bytes = rule.replace.as_bytes().to_vec();
                                return Some(PacketAction::Replace(replace_bytes));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Hooked send 函数
    #[no_mangle]
    pub unsafe extern "system" fn hooked_send(
        s: SOCKET,
        buf: *const u8,
        len: i32,
        flags: i32,
    ) -> i32 {
        // 先应用 TamperRule
        let action = if !buf.is_null() && len > 0 {
            let data_slice = std::slice::from_raw_parts(buf, len as usize);
            
            // 先检查规则
            if let Some(rule_action) = apply_tamper_rules(data_slice, HookType::Send) {
                rule_action
            } else {
                // 如果没有规则匹配，调用回调函数
                if let Some(callback) = SEND_CALLBACK.get_or_init(|| None) {
                    if let Ok(cb) = callback.lock() {
                        cb(s, data_slice, flags)
                    } else {
                        PacketAction::Allow
                    }
                } else {
                    // 默认行为：打印日志
                    debug!("[HOOK] send intercepted: socket={}, len={}", s, len);
                    debug!("[HOOK] send data (first 64 bytes): {:?}", 
                        &data_slice[..data_slice.len().min(64)]);
                    PacketAction::Allow
                }
            }
        } else {
            PacketAction::Allow
        };

        // 根据回调结果处理
        match action {
            PacketAction::Block => {
                debug!("[HOOK] send blocked by rule or callback");
                return len; // 返回成功，但实际上没有发送
            }
            PacketAction::Replace(new_data) => {
                debug!("[HOOK] send data replaced: {} -> {} bytes", len, new_data.len());
                if let Some(original_fn) = ORIGINAL_SEND {
                    return original_fn(s, new_data.as_ptr(), new_data.len() as i32, flags);
                }
            }
            PacketAction::Allow => {
                // 继续使用原始数据
            }
        }

        // 调用原始函数
        match ORIGINAL_SEND {
            Some(original_fn) => original_fn(s, buf, len, flags),
            None => {
                debug!("[HOOK] Original send function not available!");
                -1
            }
        }
    }

    /// Hooked sendto 函数
    #[no_mangle]
    pub unsafe extern "system" fn hooked_sendto(
        s: SOCKET,
        buf: *const u8,
        len: i32,
        flags: i32,
        to: *const SOCKADDR,
        tolen: i32,
    ) -> i32 {
        // 先应用 TamperRule
        let action = if !buf.is_null() && len > 0 {
            let data_slice = std::slice::from_raw_parts(buf, len as usize);
            
            // 先检查规则
            if let Some(rule_action) = apply_tamper_rules(data_slice, HookType::SendTo) {
                rule_action
            } else {
                // 如果没有规则匹配，调用回调函数
                if let Some(callback) = SENDTO_CALLBACK.get_or_init(|| None) {
                    if let Ok(cb) = callback.lock() {
                        cb(s, data_slice, flags, to)
                    } else {
                        PacketAction::Allow
                    }
                } else {
                    // 默认行为：打印日志
                    debug!("[HOOK] sendto intercepted: socket={}, len={}", s, len);
                    debug!("[HOOK] sendto data (first 64 bytes): {:?}", 
                        &data_slice[..data_slice.len().min(64)]);
                    PacketAction::Allow
                }
            }
        } else {
            PacketAction::Allow
        };

        // 根据回调结果处理
        match action {
            PacketAction::Block => {
                debug!("[HOOK] sendto blocked by rule or callback");
                return len;
            }
            PacketAction::Replace(new_data) => {
                debug!("[HOOK] sendto data replaced: {} -> {} bytes", len, new_data.len());
                if let Some(original_fn) = ORIGINAL_SENDTO {
                    return original_fn(s, new_data.as_ptr(), new_data.len() as i32, flags, to, tolen);
                }
            }
            PacketAction::Allow => {}
        }

        match ORIGINAL_SENDTO {
            Some(original_fn) => original_fn(s, buf, len, flags, to, tolen),
            None => {
                debug!("[HOOK] Original sendto function not available!");
                -1
            }
        }
    }

    /// Hooked recv 函数
    #[no_mangle]
    pub unsafe extern "system" fn hooked_recv(
        s: SOCKET,
        buf: *mut u8,
        len: i32,
        flags: i32,
    ) -> i32 {
        // 先调用原始函数接收数据
        let result = match ORIGINAL_RECV {
            Some(original_fn) => original_fn(s, buf, len, flags),
            None => {
                debug!("[HOOK] Original recv function not available!");
                return -1;
            }
        };

        if result > 0 && !buf.is_null() {
            let data_slice = std::slice::from_raw_parts(buf, result as usize);
            
            // 先应用 TamperRule
            let action = if let Some(rule_action) = apply_tamper_rules(data_slice, HookType::Recv) {
                rule_action
            } else {
                // 如果没有规则匹配，调用回调函数
                if let Some(callback) = RECV_CALLBACK.get_or_init(|| None) {
                    if let Ok(cb) = callback.lock() {
                        cb(s, data_slice)
                    } else {
                        PacketAction::Allow
                    }
                } else {
                    // 默认行为：打印日志
                    debug!("[HOOK] recv intercepted: socket={}, received={} bytes", s, result);
                    debug!("[HOOK] recv data (first 64 bytes): {:?}", 
                        &data_slice[..data_slice.len().min(64)]);
                    PacketAction::Allow
                }
            };

            // 根据回调结果处理
            match action {
                PacketAction::Block => {
                    debug!("[HOOK] recv blocked by rule or callback");
                    return 0; // 返回0表示没有数据
                }
                PacketAction::Replace(new_data) => {
                    debug!("[HOOK] recv data replaced: {} -> {} bytes", result, new_data.len());
                    let copy_len = new_data.len().min(len as usize);
                    std::ptr::copy_nonoverlapping(new_data.as_ptr(), buf, copy_len);
                    return copy_len as i32;
                }
                PacketAction::Allow => {
                    // 使用原始数据
                }
            }
        }

        result
    }

    /// Hooked recvfrom 函数
    #[no_mangle]
    pub unsafe extern "system" fn hooked_recvfrom(
        s: SOCKET,
        buf: *mut u8,
        len: i32,
        flags: i32,
        from: *mut SOCKADDR,
        fromlen: *mut i32,
    ) -> i32 {
        // 先调用原始函数接收数据
        let result = match ORIGINAL_RECVFROM {
            Some(original_fn) => original_fn(s, buf, len, flags, from, fromlen),
            None => {
                debug!("[HOOK] Original recvfrom function not available!");
                return -1;
            }
        };

        if result > 0 && !buf.is_null() {
            let data_slice = std::slice::from_raw_parts(buf, result as usize);
            let from_addr = if !from.is_null() { from } else { ptr::null() };
            
            // 先应用 TamperRule
            let action = if let Some(rule_action) = apply_tamper_rules(data_slice, HookType::RecvFrom) {
                rule_action
            } else {
                // 如果没有规则匹配，调用回调函数
                if let Some(callback) = RECVFROM_CALLBACK.get_or_init(|| None) {
                    if let Ok(cb) = callback.lock() {
                        cb(s, data_slice, from_addr)
                    } else {
                        PacketAction::Allow
                    }
                } else {
                    // 默认行为：打印日志
                    debug!("[HOOK] recvfrom intercepted: socket={}, received={} bytes", s, result);
                    debug!("[HOOK] recvfrom data (first 64 bytes): {:?}", 
                        &data_slice[..data_slice.len().min(64)]);
                    PacketAction::Allow
                }
            };

            // 根据回调结果处理
            match action {
                PacketAction::Block => {
                    debug!("[HOOK] recvfrom blocked by rule or callback");
                    return 0;
                }
                PacketAction::Replace(new_data) => {
                    debug!("[HOOK] recvfrom data replaced: {} -> {} bytes", result, new_data.len());
                    let copy_len = new_data.len().min(len as usize);
                    std::ptr::copy_nonoverlapping(new_data.as_ptr(), buf, copy_len);
                    return copy_len as i32;
                }
                PacketAction::Allow => {}
            }
        }

        result
    }

    /// Hooked WSASend 函数
    #[no_mangle]
    #[allow(non_snake_case)]
    pub unsafe extern "system" fn hooked_wsasend(
        s: SOCKET,
        lpBuffers: *const WSABUF,
        dwBufferCount: u32,
        lpNumberOfBytesSent: *mut u32,
        dwFlags: u32,
        lpOverlapped: *mut OVERLAPPED,
        lpCompletionRoutine: Option<unsafe extern "system" fn(*mut OVERLAPPED, u32, u32, *mut c_void)>,
    ) -> i32 {
        // 先应用 TamperRule
        let action = if !lpBuffers.is_null() {
            let buffers = std::slice::from_raw_parts(lpBuffers, dwBufferCount as usize);
            
            // 收集所有缓冲区的数据
            let mut all_data = Vec::new();
            for buf in buffers.iter() {
                if !buf.buf.is_null() && buf.len > 0 {
                    let data_slice = std::slice::from_raw_parts(buf.buf, buf.len as usize);
                    all_data.extend_from_slice(data_slice);
                }
            }
            
            // 先检查规则
            let rule_action = if !all_data.is_empty() {
                apply_tamper_rules(&all_data, HookType::WSASend)
            } else {
                None
            };
            
            if let Some(rule_action) = rule_action {
                rule_action
            } else {
                // 如果没有规则匹配，调用回调函数
                if let Some(callback) = WSASEND_CALLBACK.get_or_init(|| None) {
                    if let Ok(cb) = callback.lock() {
                        cb(s, buffers)
                    } else {
                        PacketAction::Allow
                    }
                } else {
                    // 默认行为：打印日志
                    debug!("[HOOK] WSASend intercepted: socket={}, buffer_count={}", s, dwBufferCount);
                    for (i, buf) in buffers.iter().enumerate() {
                        if !buf.buf.is_null() && buf.len > 0 {
                            let data_slice = std::slice::from_raw_parts(buf.buf, buf.len as usize);
                            debug!("[HOOK] WSASend buffer[{}] (first 64 bytes): {:?}", 
                                i, &data_slice[..data_slice.len().min(64)]);
                        }
                    }
                    PacketAction::Allow
                }
            }
        } else {
            PacketAction::Allow
        };

        // 根据回调结果处理
        match action {
            PacketAction::Block => {
                debug!("[HOOK] WSASend blocked by rule or callback");
                if !lpNumberOfBytesSent.is_null() {
                    *lpNumberOfBytesSent = 0;
                }
                return 0;
            }
            PacketAction::Replace(new_data) => {
                debug!("[HOOK] WSASend data replaced: {} bytes", new_data.len());
                // 注意：WSASend 的替换比较复杂，这里简化处理
                // 实际应用中可能需要创建新的 WSABUF 数组
            }
            PacketAction::Allow => {}
        }

        match ORIGINAL_WSASEND {
            Some(original_fn) => original_fn(s, lpBuffers, dwBufferCount, lpNumberOfBytesSent, dwFlags, lpOverlapped, lpCompletionRoutine),
            None => {
                debug!("[HOOK] Original WSASend function not available!");
                -1
            }
        }
    }

    /// Hooked WSARecv 函数
    #[no_mangle]
    #[allow(non_snake_case)]
    pub unsafe extern "system" fn hooked_wsarecv(
        s: SOCKET,
        lpBuffers: *mut WSABUF,
        dwBufferCount: u32,
        lpNumberOfBytesRecvd: *mut u32,
        lpFlags: *mut u32,
        lpOverlapped: *mut OVERLAPPED,
        lpCompletionRoutine: Option<unsafe extern "system" fn(*mut OVERLAPPED, u32, u32, *mut c_void)>,
    ) -> i32 {
        // 先调用原始函数接收数据
        let result = match ORIGINAL_WSARECV {
            Some(original_fn) => original_fn(s, lpBuffers, dwBufferCount, lpNumberOfBytesRecvd, lpFlags, lpOverlapped, lpCompletionRoutine),
            None => {
                debug!("[HOOK] Original WSARecv function not available!");
                return -1;
            }
        };

        if result == 0 && !lpBuffers.is_null() && !lpNumberOfBytesRecvd.is_null() {
            let bytes_recvd = *lpNumberOfBytesRecvd;
            
            // 收集所有接收到的数据
            let mut all_data = Vec::new();
            let buffers = std::slice::from_raw_parts(lpBuffers, dwBufferCount as usize);
            for buf in buffers.iter() {
                if !buf.buf.is_null() && buf.len > 0 {
                    let len = (buf.len as usize).min(bytes_recvd as usize);
                    let data_slice = std::slice::from_raw_parts(buf.buf, len);
                    all_data.extend_from_slice(data_slice);
                }
            }

            // 先应用 TamperRule
            let action = if let Some(rule_action) = apply_tamper_rules(&all_data, HookType::WSARecv) {
                rule_action
            } else {
                // 如果没有规则匹配，调用回调函数
                if let Some(callback) = WSARECV_CALLBACK.get_or_init(|| None) {
                    if let Ok(cb) = callback.lock() {
                        cb(s, &all_data)
                    } else {
                        PacketAction::Allow
                    }
                } else {
                    // 默认行为：打印日志
                    debug!("[HOOK] WSARecv intercepted: socket={}, received={} bytes", s, bytes_recvd);
                    for (i, buf) in buffers.iter().enumerate() {
                        if !buf.buf.is_null() && buf.len > 0 {
                            let len = (buf.len as usize).min(bytes_recvd as usize);
                            let data_slice = std::slice::from_raw_parts(buf.buf, len);
                            debug!("[HOOK] WSARecv buffer[{}] (first 64 bytes): {:?}", 
                                i, &data_slice[..data_slice.len().min(64)]);
                        }
                    }
                    PacketAction::Allow
                }
            };

            // 根据回调结果处理
            match action {
                PacketAction::Block => {
                    debug!("[HOOK] WSARecv blocked by rule or callback");
                    *lpNumberOfBytesRecvd = 0;
                    return 0;
                }
                PacketAction::Replace(new_data) => {
                    debug!("[HOOK] WSARecv data replaced: {} -> {} bytes", bytes_recvd, new_data.len());
                    // 将替换的数据复制回缓冲区
                    let mut offset = 0;
                    for buf in buffers.iter() {
                        if offset >= new_data.len() {
                            break;
                        }
                        if !buf.buf.is_null() && buf.len > 0 {
                            let copy_len = (buf.len as usize).min(new_data.len() - offset);
                            std::ptr::copy_nonoverlapping(
                                new_data.as_ptr().add(offset),
                                buf.buf,
                                copy_len,
                            );
                            offset += copy_len;
                        }
                    }
                    *lpNumberOfBytesRecvd = new_data.len().min(bytes_recvd as usize) as u32;
                }
                PacketAction::Allow => {}
            }
        }

        result
    }
}

#[cfg(target_os = "windows")]
pub use network_hook::{
    SendHook, SendToHook, RecvHook, RecvFromHook, 
    WSASendHook, WSARecvHook
};

pub use network_hook::{
    NetworkHook, NetworkHookManager, PacketAction, set_global_rules
};
