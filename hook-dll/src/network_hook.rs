mod network_hook {
    
    #[cfg(target_os = "windows")]
    use min_hook_rs::*;
    #[cfg(target_os = "windows")]
    use windows_sys::Win32::Networking::WinSock::*;
    #[cfg(target_os = "windows")]
    use windows_sys::Win32::System::IO::OVERLAPPED;
    
    use core::option::Option::None;
    use std::ffi::c_void;
    use std::ptr;
    use std::sync::{Arc, Mutex, OnceLock};
    use log::*;
    use crate::{HookType, TamperAction, TamperRule, capture_and_send_packet};
    use crate::wildcard::{wildcard_match, wildcard_find};
    
    // ========== 回调类型定义 ==========

    /// 数据包处理结果
    #[derive(Debug, Clone)]
    pub enum PacketAction {
        /// 允许通过，使用原始数据
        Allow,
        /// 阻止数据包
        Block,
        /// 替换数据包内容
        /// Replace(替换后的完整数据包)
        Replace(Vec<u8>),
    }

    
    /// Send 回调函数类型
    /// 参数: (socket, 数据指针, 数据长度, flags)
    /// 返回: PacketAction - 决定如何处理数据包
    #[cfg(target_os = "windows")]
    pub type SendCallback = dyn Fn(SOCKET, &[u8], i32) -> PacketAction + Send + Sync;

    /// Recv 回调函数类型
    /// 参数: (socket, 接收到的数据)
    /// 返回: PacketAction - 决定如何处理数据包
    #[cfg(target_os = "windows")]
    pub type RecvCallback = dyn Fn(SOCKET, &[u8]) -> PacketAction + Send + Sync;

    /// SendTo 回调函数类型
    /// 参数: (socket, 数据指针, 数据长度, flags, 目标地址)
    #[cfg(target_os = "windows")]
    pub type SendToCallback = dyn Fn(SOCKET, &[u8], i32, *const SOCKADDR) -> PacketAction + Send + Sync;

    /// RecvFrom 回调函数类型
    /// 参数: (socket, 接收到的数据, 来源地址)
    #[cfg(target_os = "windows")]
    pub type RecvFromCallback = dyn Fn(SOCKET, &[u8], *const SOCKADDR) -> PacketAction + Send + Sync;

    /// WSASend 回调函数类型
    #[cfg(target_os = "windows")]
    pub type WSASendCallback = dyn Fn(SOCKET, &[WSABUF]) -> PacketAction + Send + Sync;

    /// WSARecv 回调函数类型
    #[cfg(target_os = "windows")]
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

            // 只在第一次创建 hook（target 为空时）
            if self.target.is_null() {
                let (trampoline, target) =
                    create_hook_api("ws2_32", "send", hooked_send as *mut c_void)?;
                
                unsafe {
                    ORIGINAL_SEND = Some(std::mem::transmute(trampoline));
                    // 注册回调
                    let _ = SEND_CALLBACK.set(self.callback.clone());
                }
                
                self.target = target;
            }

            enable_hook(self.target)?;
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

            // 只在第一次创建 hook（target 为空时）
            if self.target.is_null() {
                let (trampoline, target) =
                    create_hook_api("ws2_32", "sendto", hooked_sendto as *mut c_void)?;
                
                unsafe {
                    ORIGINAL_SENDTO = Some(std::mem::transmute(trampoline));
                    let _ = SENDTO_CALLBACK.set(self.callback.clone());
                }
                
                self.target = target;
            }

            enable_hook(self.target)?;
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

            // 只在第一次创建 hook（target 为空时）
            if self.target.is_null() {
                let (trampoline, target) =
                    create_hook_api("ws2_32", "recv", hooked_recv as *mut c_void)?;
                
                unsafe {
                    ORIGINAL_RECV = Some(std::mem::transmute(trampoline));
                    let _ = RECV_CALLBACK.set(self.callback.clone());
                }
                
                self.target = target;
            }

            enable_hook(self.target)?;
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

            // 只在第一次创建 hook（target 为空时）
            if self.target.is_null() {
                let (trampoline, target) =
                    create_hook_api("ws2_32", "recvfrom", hooked_recvfrom as *mut c_void)?;
                
                unsafe {
                    ORIGINAL_RECVFROM = Some(std::mem::transmute(trampoline));
                    let _ = RECVFROM_CALLBACK.set(self.callback.clone());
                }
                
                self.target = target;
            }

            enable_hook(self.target)?;
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

            // 只在第一次创建 hook（target 为空时）
            if self.target.is_null() {
                let (trampoline, target) =
                    create_hook_api("ws2_32", "WSASend", hooked_wsasend as *mut c_void)?;
                
                unsafe {
                    ORIGINAL_WSASEND = Some(std::mem::transmute(trampoline));
                    let _ = WSASEND_CALLBACK.set(self.callback.clone());
                }
                
                self.target = target;
            }

            enable_hook(self.target)?;
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

            // 只在第一次创建 hook（target 为空时）
            if self.target.is_null() {
                let (trampoline, target) =
                    create_hook_api("ws2_32", "WSARecv", hooked_wsarecv as *mut c_void)?;
                
                unsafe {
                    ORIGINAL_WSARECV = Some(std::mem::transmute(trampoline));
                    let _ = WSARECV_CALLBACK.set(self.callback.clone());
                }
                
                self.target = target;
            }

            enable_hook(self.target)?;
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
                if let Err(e) = hook.enable() {
                    error!("Failed to enable hook {}: {}", hook.name(), e);
                }
            }
            Ok(())
        }

        /// 禁用所有 hook
        pub fn disable_all(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            for hook in &mut self.hooks {
                if let Err(e) = hook.disable() {
                    error!("Failed to disable hook {}: {}", hook.name(), e);
                }
            }
            Ok(())
        }

        /// 清理所有资源
        pub fn cleanup_all(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            for hook in &mut self.hooks {
                if let Err(e) = hook.cleanup() {
                    error!("Failed to cleanup hook {}: {}", hook.name(), e);
                }
            }
            self.hooks.clear();
            Ok(())
        }

        /// 启用 Send hook
        pub fn enable_send(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.send_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    if let Err(e) = hook.enable() {
                        error!("Failed to enable hook {}: {}", hook.name(), e);
                    }
                }
            }
            Ok(())
        }

        /// 禁用 Send hook
        pub fn disable_send(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.send_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    if let Err(e) = hook.disable() {
                        error!("Failed to disable hook {}: {}", hook.name(), e);
                    }
                }
            }
            Ok(())
        }

        /// 启用 Recv hook
        pub fn enable_recv(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.recv_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    if let Err(e) = hook.enable() {
                        error!("Failed to enable hook {}: {}", hook.name(), e);
                    }
                }
            }
            Ok(())
        }

        /// 禁用 Recv hook
        pub fn disable_recv(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.recv_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    if let Err(e) = hook.disable() {
                        error!("Failed to disable hook {}: {}", hook.name(), e);
                    }
                }
            }
            Ok(())
        }

        /// 启用 SendTo hook
        pub fn enable_sendto(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.sendto_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    if let Err(e) = hook.enable() {
                        error!("Failed to enable hook {}: {}", hook.name(), e);
                    }
                }
            }
            Ok(())
        }

        /// 禁用 SendTo hook
        pub fn disable_sendto(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.sendto_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    if let Err(e) = hook.disable() {
                        error!("Failed to disable hook {}: {}", hook.name(), e);
                    }
                }
            }
            Ok(())
        }

        /// 启用 RecvFrom hook
        pub fn enable_recvfrom(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.recvfrom_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    if let Err(e) = hook.enable() {
                        error!("Failed to enable hook {}: {}", hook.name(), e);
                    }
                }
            }
            Ok(())
        }

        /// 禁用 RecvFrom hook
        pub fn disable_recvfrom(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.recvfrom_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    if let Err(e) = hook.disable() {
                        error!("Failed to disable hook {}: {}", hook.name(), e);
                    }
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
                    if let Err(e) = hook.enable() {
                        error!("Failed to enable hook {}: {}", hook.name(), e);
                    }
                }
            }
            Ok(())
        }

        /// 禁用 WSARecv hook
        pub fn disable_wsarecv(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
            if let Some(idx) = self.wsa_recv_hook_index {
                if let Some(hook) = self.hooks.get_mut(idx) {
                    if let Err(e) = hook.disable() {
                        error!("Failed to disable hook {}: {}", hook.name(), e);
                    }
                }
            }
            Ok(())
        }
    }

    // ========== Hook 函数实现 ==========

    // 原始函数指针存储
    #[cfg(target_os = "windows")]
    type SendFn = unsafe extern "system" fn(SOCKET, *const u8, i32, i32) -> i32;
    #[cfg(target_os = "windows")]
    type SendToFn = unsafe extern "system" fn(SOCKET, *const u8, i32, i32, *const SOCKADDR, i32) -> i32;
    #[cfg(target_os = "windows")]
    type RecvFn = unsafe extern "system" fn(SOCKET, *mut u8, i32, i32) -> i32;
    #[cfg(target_os = "windows")]
    type RecvFromFn = unsafe extern "system" fn(SOCKET, *mut u8, i32, i32, *mut SOCKADDR, *mut i32) -> i32;
    #[cfg(target_os = "windows")]
    type WSASendFn = unsafe extern "system" fn(SOCKET, *const WSABUF, u32, *mut u32, u32, *mut OVERLAPPED, Option<unsafe extern "system" fn(*mut OVERLAPPED, u32, u32, *mut c_void)>) -> i32;
    #[cfg(target_os = "windows")]
    type WSARecvFn = unsafe extern "system" fn(SOCKET, *mut WSABUF, u32, *mut u32, *mut u32, *mut OVERLAPPED, Option<unsafe extern "system" fn(*mut OVERLAPPED, u32, u32, *mut c_void)>) -> i32;

    #[cfg(target_os = "windows")]
    static mut ORIGINAL_SEND: Option<SendFn> = None;
    #[cfg(target_os = "windows")]
    static mut ORIGINAL_SENDTO: Option<SendToFn> = None;
    #[cfg(target_os = "windows")]
    static mut ORIGINAL_RECV: Option<RecvFn> = None;
    #[cfg(target_os = "windows")]
    static mut ORIGINAL_RECVFROM: Option<RecvFromFn> = None;
    #[cfg(target_os = "windows")]
    static mut ORIGINAL_WSASEND: Option<WSASendFn> = None;
    #[cfg(target_os = "windows")]
    static mut ORIGINAL_WSARECV: Option<WSARecvFn> = None;

    // 全局回调存储 - 使用 OnceLock 避免 static mut 的警告
    #[cfg(target_os = "windows")]
    static SEND_CALLBACK: OnceLock<Option<Arc<Mutex<Box<SendCallback>>>>> = OnceLock::new();
    #[cfg(target_os = "windows")]
    static SENDTO_CALLBACK: OnceLock<Option<Arc<Mutex<Box<SendToCallback>>>>> = OnceLock::new();
    #[cfg(target_os = "windows")]
    static RECV_CALLBACK: OnceLock<Option<Arc<Mutex<Box<RecvCallback>>>>> = OnceLock::new();
    #[cfg(target_os = "windows")]
    static RECVFROM_CALLBACK: OnceLock<Option<Arc<Mutex<Box<RecvFromCallback>>>>> = OnceLock::new();
    #[cfg(target_os = "windows")]
    static WSASEND_CALLBACK: OnceLock<Option<Arc<Mutex<Box<WSASendCallback>>>>> = OnceLock::new();
    #[cfg(target_os = "windows")]
    static WSARECV_CALLBACK: OnceLock<Option<Arc<Mutex<Box<WSARecvCallback>>>>> = OnceLock::new();
    
    // 全局规则存储
    #[cfg(test)]
    pub(crate) static TAMPER_RULES: std::sync::Mutex<Option<Arc<Mutex<Vec<TamperRule>>>>> = std::sync::Mutex::new(None);
    
    #[cfg(not(test))]
    static TAMPER_RULES: OnceLock<Arc<Mutex<Vec<TamperRule>>>> = OnceLock::new();
    
    /// 设置全局规则存储
    #[cfg(not(test))]
    pub fn set_global_rules(rules: Arc<Mutex<Vec<TamperRule>>>) {
        let _ = TAMPER_RULES.set(rules);
    }
    
    /// 设置全局规则存储（测试版本，允许重新设置）
    #[cfg(test)]
    pub fn set_global_rules(rules: Arc<Mutex<Vec<TamperRule>>>) {
        let mut guard = TAMPER_RULES.lock().unwrap();
        *guard = Some(rules);
    }
    
    
    /// 应用规则到数据包（内部实现）
    pub(crate) fn apply_tamper_rules(data: &[u8], hook_type: HookType) -> Option<PacketAction> {
        #[cfg(test)]
        {
            if let Ok(guard) = TAMPER_RULES.lock() {
                if let Some(rules_arc) = guard.as_ref() {
                    if let Ok(mut rules) = rules_arc.lock() {
                        return apply_tamper_rules_impl(data, hook_type, &mut rules);
                    }
                }
            }
            return None;
        }
        
        #[cfg(not(test))]
        {
            if let Some(rules_arc) = TAMPER_RULES.get() {
                if let Ok(mut rules) = rules_arc.lock() {
                    return apply_tamper_rules_impl(data, hook_type, &mut rules);
                }
            }
            return None;
        }
    }
    
    /// 应用规则到数据包（实际实现）
    fn apply_tamper_rules_impl(data: &[u8], hook_type: HookType, rules: &mut Vec<TamperRule>) -> Option<PacketAction> {
        for rule in rules.iter_mut() {
            if !rule.active {
                continue;
            }
            if rule.hook != hook_type {
                continue;
            }
            // 查找匹配位置
            if let Some((start, length)) = wildcard_find(&rule.match_pattern, data) {
                // 增加命中计数
                rule.hits += 1;
                
                match rule.action {
                    TamperAction::Block => {
                        return Some(PacketAction::Block);
                    }
                    TamperAction::Replace => {
                        // 解析替换内容（hex 字符串）
                        let replace_pattern = rule.replace.trim();
                        let mut replace_bytes = Vec::new();
                        
                        // 移除空格并转换为小写
                        let normalized: String = replace_pattern.chars()
                            .filter(|c| !c.is_whitespace())
                            .map(|c| c.to_ascii_lowercase())
                            .collect();
                        
                        let normalized_bytes = normalized.as_bytes();
                        let mut idx = 0;
                        
                        // 解析 hex 字节
                        while idx + 1 < normalized_bytes.len() {
                            let hex_str = unsafe { std::str::from_utf8_unchecked(&normalized_bytes[idx..idx + 2]) };
                            if let Ok(byte) = u8::from_str_radix(hex_str, 16) {
                                replace_bytes.push(byte);
                                idx += 2;
                            } else {
                                // 无效的 hex，使用原始字符串的字节
                                replace_bytes = rule.replace.as_bytes().to_vec();
                                break;
                            }
                        }
                        
                        // 只替换匹配的部分，保留其他部分
                        let mut new_data = Vec::with_capacity(data.len() - length + replace_bytes.len());
                        new_data.extend_from_slice(&data[..start]);
                        new_data.extend_from_slice(&replace_bytes);
                        new_data.extend_from_slice(&data[start + length..]);
                        
                        return Some(PacketAction::Replace(new_data));
                    }
                }
            }
        }
        None
    }

    /// Hooked send 函数
    #[no_mangle]
    #[cfg(target_os = "windows")]
    pub unsafe extern "system" fn hooked_send(
        s: SOCKET,
        buf: *const u8,
        len: i32,
        flags: i32,
    ) -> i32 {
            // 先应用 TamperRule（检查所有规则，增加命中计数）
            let rule_action = if !buf.is_null() && len > 0 {
                let data_slice = std::slice::from_raw_parts(buf, len as usize);
                
                // 捕获并发送封包数据
                capture_and_send_packet(data_slice, HookType::Send, s as u64, None, None);
                
                apply_tamper_rules(data_slice, HookType::Send)
        } else {
            None
        };
        
        // 无论规则是否匹配，都调用回调函数（用于事件通知）
        let callback_action = if !buf.is_null() && len > 0 {
            let data_slice = std::slice::from_raw_parts(buf, len as usize);
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
        } else {
            PacketAction::Allow
        };
        
        // 规则动作优先级高于回调函数
        let action = rule_action.unwrap_or(callback_action);

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
    #[cfg(target_os = "windows")]
    pub unsafe extern "system" fn hooked_sendto(
        s: SOCKET,
        buf: *const u8,
        len: i32,
        flags: i32,
        to: *const SOCKADDR,
        tolen: i32,
    ) -> i32 {
        // 先应用 TamperRule（检查所有规则，增加命中计数）
        let rule_action = if !buf.is_null() && len > 0 {
            let data_slice = std::slice::from_raw_parts(buf, len as usize);
            
            // 捕获并发送封包数据
            capture_and_send_packet(data_slice, HookType::SendTo, s as u64, None, None);
            
            apply_tamper_rules(data_slice, HookType::SendTo)
        } else {
            None
        };
        
        // 无论规则是否匹配，都调用回调函数（用于事件通知）
        let callback_action = if !buf.is_null() && len > 0 {
            let data_slice = std::slice::from_raw_parts(buf, len as usize);
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
        } else {
            PacketAction::Allow
        };
        
        // 规则动作优先级高于回调函数
        let action = rule_action.unwrap_or(callback_action);

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
    #[cfg(target_os = "windows")]
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
            
            // 捕获并发送封包数据
            capture_and_send_packet(data_slice, HookType::Recv, s as u64, None, None);
            
            // 先应用 TamperRule（检查所有规则，增加命中计数）
            let rule_action = apply_tamper_rules(data_slice, HookType::Recv);
            
            // 无论规则是否匹配，都调用回调函数（用于事件通知）
            let callback_action = if let Some(callback) = RECV_CALLBACK.get_or_init(|| None) {
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
            };
            
            // 规则动作优先级高于回调函数
            let action = rule_action.unwrap_or(callback_action);

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
    #[cfg(target_os = "windows")]
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
            
            // 捕获并发送封包数据
            capture_and_send_packet(data_slice, HookType::RecvFrom, s as u64, None, None);
            
            // 先应用 TamperRule（检查所有规则，增加命中计数）
            let rule_action = apply_tamper_rules(data_slice, HookType::RecvFrom);
            
            // 无论规则是否匹配，都调用回调函数（用于事件通知）
            let callback_action = if let Some(callback) = RECVFROM_CALLBACK.get_or_init(|| None) {
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
            };
            
            // 规则动作优先级高于回调函数
            let action = rule_action.unwrap_or(callback_action);

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
    #[cfg(target_os = "windows")]
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
        // 先应用 TamperRule（检查所有规则，增加命中计数）
        let rule_action = if !lpBuffers.is_null() {
            let buffers = std::slice::from_raw_parts(lpBuffers, dwBufferCount as usize);
            
            // 收集所有缓冲区的数据
            let mut all_data = Vec::new();
            for buf in buffers.iter() {
                if !buf.buf.is_null() && buf.len > 0 {
                    let data_slice = std::slice::from_raw_parts(buf.buf, buf.len as usize);
                    all_data.extend_from_slice(data_slice);
                }
            }
            
            if !all_data.is_empty() {
                // 捕获并发送封包数据
                capture_and_send_packet(&all_data, HookType::WSASend, s as u64, None, None);
                
                apply_tamper_rules(&all_data, HookType::WSASend)
            } else {
                None
            }
        } else {
            None
        };
        
        // 无论规则是否匹配，都调用回调函数（用于事件通知）
        let callback_action = if !lpBuffers.is_null() {
            let buffers = std::slice::from_raw_parts(lpBuffers, dwBufferCount as usize);
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
        } else {
            PacketAction::Allow
        };
        
        // 规则动作优先级高于回调函数
        let action = rule_action.unwrap_or(callback_action);

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
    #[cfg(target_os = "windows")]
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

            // 捕获并发送封包数据
            capture_and_send_packet(&all_data, HookType::WSARecv, s as u64, None, None);

            // 先应用 TamperRule（检查所有规则，增加命中计数）
            let rule_action = apply_tamper_rules(&all_data, HookType::WSARecv);
            
            // 无论规则是否匹配，都调用回调函数（用于事件通知）
            let callback_action = if let Some(callback) = WSARECV_CALLBACK.get_or_init(|| None) {
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
            };
            
            // 规则动作优先级高于回调函数
            let action = rule_action.unwrap_or(callback_action);

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

#[cfg(test)]
mod tests {
    use super::network_hook::apply_tamper_rules;
    use crate::{HookType, TamperAction, TamperRule};
    use crate::network_hook::{PacketAction, set_global_rules};
    use std::sync::{Arc, Mutex};
    
    fn setup_rules(rules: Vec<TamperRule>) {
        let rules_arc = Arc::new(Mutex::new(rules));
        set_global_rules(rules_arc);
    }
    
    fn get_rules() -> Arc<Mutex<Vec<TamperRule>>> {
        use super::network_hook::TAMPER_RULES;
        let guard = TAMPER_RULES.lock().unwrap();
        guard.as_ref().unwrap().clone()
    }
    
    fn clear_rules() {
        use super::network_hook::TAMPER_RULES;
        let mut guard = TAMPER_RULES.lock().unwrap();
        *guard = None;
    }
    
    #[test]
    fn test_apply_tamper_rules_block() {
        clear_rules(); // 清理之前的状态
        // 测试 Block 动作
        let rule = TamperRule {
            id: "test_block_1".to_string(),
            name: "Block Test".to_string(),
            match_pattern: "bb ?? dd".to_string(),
            replace: "".to_string(),
            action: TamperAction::Block,
            active: true,
            hits: 0,
            hook: HookType::Send,
        };
        
        setup_rules(vec![rule]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        let result = apply_tamper_rules(data, HookType::Send);
        
        assert!(result.is_some(), "规则应该匹配");
        match result.unwrap() {
            PacketAction::Block => {
                // 正确
            }
            _ => panic!("应该返回 Block 动作"),
        }
        
        // 验证命中计数
        let rules_arc = get_rules();
        let rules = rules_arc.lock().unwrap();
        assert_eq!(rules[0].hits, 1, "命中计数应该为 1");
    }
    
    #[test]
    fn test_apply_tamper_rules_replace() {
        clear_rules(); // 清理之前的状态
        // 测试 Replace 动作 - 只替换匹配部分
        let rule = TamperRule {
            id: "test_replace_1".to_string(),
            name: "Replace Test".to_string(),
            match_pattern: "bb ?? dd".to_string(),
            replace: "11 22 33".to_string(), // hex 字符串
            action: TamperAction::Replace,
            active: true,
            hits: 0,
            hook: HookType::Send,
        };
        
        setup_rules(vec![rule]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        let result = apply_tamper_rules(data, HookType::Send);
        
        assert!(result.is_some(), "规则应该匹配");
        match result.unwrap() {
            PacketAction::Replace(new_data) => {
                // 应该只替换匹配的部分：bb ?? dd (位置 1-3) 替换为 11 22 33
                // 原始: [0xaa, 0xbb, 0xcc, 0xdd, 0xee]
                // 匹配: [0xbb, 0xcc, 0xdd] (位置 1-3，长度 3)
                // 替换为: [0x11, 0x22, 0x33]
                // 结果: [0xaa, 0x11, 0x22, 0x33, 0xee]
                let expected = vec![0xaa, 0x11, 0x22, 0x33, 0xee];
                assert_eq!(new_data, expected, "应该只替换匹配的部分");
            }
            _ => panic!("应该返回 Replace 动作"),
        }
        
        // 验证命中计数
        let rules_arc = get_rules();
        let rules = rules_arc.lock().unwrap();
        assert_eq!(rules[0].hits, 1, "命中计数应该为 1");
    }
    
    #[test]
    fn test_apply_tamper_rules_no_match() {
        clear_rules(); // 清理之前的状态
        // 测试不匹配的情况
        let rule = TamperRule {
            id: "test_no_match".to_string(),
            name: "No Match Test".to_string(),
            match_pattern: "ff ff ff".to_string(),
            replace: "".to_string(),
            action: TamperAction::Block,
            active: true,
            hits: 0,
            hook: HookType::Send,
        };
        
        setup_rules(vec![rule]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        let result = apply_tamper_rules(data, HookType::Send);
        
        assert!(result.is_none(), "规则不应该匹配");
        
        // 验证命中计数未增加
        let rules_arc = get_rules();
        let rules = rules_arc.lock().unwrap();
        assert_eq!(rules[0].hits, 0, "命中计数应该仍为 0");
    }
    
    #[test]
    fn test_apply_tamper_rules_inactive() {
        clear_rules(); // 清理之前的状态
        // 测试非激活规则
        let rule = TamperRule {
            id: "test_inactive".to_string(),
            name: "Inactive Test".to_string(),
            match_pattern: "bb ?? dd".to_string(),
            replace: "".to_string(),
            action: TamperAction::Block,
            active: false, // 非激活
            hits: 0,
            hook: HookType::Send,
        };
        
        setup_rules(vec![rule]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        let result = apply_tamper_rules(data, HookType::Send);
        
        assert!(result.is_none(), "非激活规则不应该匹配");
    }
    
    #[test]
    fn test_apply_tamper_rules_wrong_hook_type() {
        clear_rules(); // 清理之前的状态
        // 测试错误的 HookType
        let rule = TamperRule {
            id: "test_wrong_hook".to_string(),
            name: "Wrong Hook Test".to_string(),
            match_pattern: "bb ?? dd".to_string(),
            replace: "".to_string(),
            action: TamperAction::Block,
            active: true,
            hits: 0,
            hook: HookType::Send, // 规则适用于 Send
        };
        
        setup_rules(vec![rule]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        let result = apply_tamper_rules(data, HookType::Recv); // 但数据是 Recv 类型
        
        assert!(result.is_none(), "不同 HookType 不应该匹配");
    }
    
    #[test]
    fn test_apply_tamper_rules_multiple_rules() {
        clear_rules(); // 清理之前的状态
        // 测试多个规则，第一个匹配的规则生效
        let rule1 = TamperRule {
            id: "test_rule_1".to_string(),
            name: "First Rule".to_string(),
            match_pattern: "aa bb".to_string(),
            replace: "".to_string(),
            action: TamperAction::Block,
            active: true,
            hits: 0,
            hook: HookType::Send,
        };
        
        let rule2 = TamperRule {
            id: "test_rule_2".to_string(),
            name: "Second Rule".to_string(),
            match_pattern: "bb ?? dd".to_string(),
            replace: "".to_string(),
            action: TamperAction::Block,
            active: true,
            hits: 0,
            hook: HookType::Send,
        };
        
        setup_rules(vec![rule1, rule2]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        let result = apply_tamper_rules(data, HookType::Send);
        
        assert!(result.is_some(), "应该匹配第一个规则");
        match result.unwrap() {
            PacketAction::Block => {
                // 正确
            }
            _ => panic!("应该返回 Block 动作"),
        }
        
        // 验证只有第一个规则命中计数增加
        let rules_arc = get_rules();
        let rules = rules_arc.lock().unwrap();
        assert_eq!(rules[0].hits, 1, "第一个规则命中计数应该为 1");
        assert_eq!(rules[1].hits, 0, "第二个规则命中计数应该仍为 0");
    }
    
    #[test]
    fn test_apply_tamper_rules_replace_at_start() {
        clear_rules(); // 清理之前的状态
        // 测试替换在数据开头的匹配
        let rule = TamperRule {
            id: "test_replace_start".to_string(),
            name: "Replace Start".to_string(),
            match_pattern: "aa bb".to_string(),
            replace: "ff ee".to_string(),
            action: TamperAction::Replace,
            active: true,
            hits: 0,
            hook: HookType::Send,
        };
        
        setup_rules(vec![rule]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd];
        let result = apply_tamper_rules(data, HookType::Send);
        
        assert!(result.is_some(), "规则应该匹配");
        match result.unwrap() {
            PacketAction::Replace(new_data) => {
                let expected = vec![0xff, 0xee, 0xcc, 0xdd];
                assert_eq!(new_data, expected, "应该替换开头的匹配部分");
            }
            _ => panic!("应该返回 Replace 动作"),
        }
    }
    
    #[test]
    fn test_apply_tamper_rules_replace_at_end() {
        clear_rules(); // 清理之前的状态
        // 测试替换在数据结尾的匹配
        let rule = TamperRule {
            id: "test_replace_end".to_string(),
            name: "Replace End".to_string(),
            match_pattern: "cc dd".to_string(),
            replace: "ff ee".to_string(),
            action: TamperAction::Replace,
            active: true,
            hits: 0,
            hook: HookType::Send,
        };
        
        setup_rules(vec![rule]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd];
        let result = apply_tamper_rules(data, HookType::Send);
        
        assert!(result.is_some(), "规则应该匹配");
        match result.unwrap() {
            PacketAction::Replace(new_data) => {
                let expected = vec![0xaa, 0xbb, 0xff, 0xee];
                assert_eq!(new_data, expected, "应该替换结尾的匹配部分");
            }
            _ => panic!("应该返回 Replace 动作"),
        }
    }
    
    #[test]
    fn test_apply_tamper_rules_replace_different_length() {
        clear_rules(); // 清理之前的状态
        // 测试替换长度不同的情况
        let rule = TamperRule {
            id: "test_replace_length".to_string(),
            name: "Replace Different Length".to_string(),
            match_pattern: "bb cc".to_string(), // 2 字节
            replace: "11 22 33 44".to_string(), // 4 字节
            action: TamperAction::Replace,
            active: true,
            hits: 0,
            hook: HookType::Send,
        };
        
        setup_rules(vec![rule]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd];
        let result = apply_tamper_rules(data, HookType::Send);
        
        assert!(result.is_some(), "规则应该匹配");
        match result.unwrap() {
            PacketAction::Replace(new_data) => {
                // 原始: [0xaa, 0xbb, 0xcc, 0xdd]
                // 匹配: [0xbb, 0xcc] (位置 1-2，长度 2)
                // 替换为: [0x11, 0x22, 0x33, 0x44] (长度 4)
                // 结果: [0xaa, 0x11, 0x22, 0x33, 0x44, 0xdd]
                let expected = vec![0xaa, 0x11, 0x22, 0x33, 0x44, 0xdd];
                assert_eq!(new_data, expected, "应该正确替换不同长度的内容");
                assert_eq!(new_data.len(), 6, "新数据长度应该是 6");
            }
            _ => panic!("应该返回 Replace 动作"),
        }
    }
    
    #[test]
    fn test_apply_tamper_rules_replace_with_wildcard() {
        clear_rules(); // 清理之前的状态
        // 测试包含通配符的替换
        let rule = TamperRule {
            id: "test_replace_wildcard".to_string(),
            name: "Replace With Wildcard".to_string(),
            match_pattern: "aa ?? cc".to_string(), // 匹配 aa ?? cc
            replace: "ff ee".to_string(),
            action: TamperAction::Replace,
            active: true,
            hits: 0,
            hook: HookType::Send,
        };
        
        setup_rules(vec![rule]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd];
        let result = apply_tamper_rules(data, HookType::Send);
        
        assert!(result.is_some(), "规则应该匹配");
        match result.unwrap() {
            PacketAction::Replace(new_data) => {
                // 原始: [0xaa, 0xbb, 0xcc, 0xdd]
                // 匹配: [0xaa, 0xbb, 0xcc] (位置 0-2，长度 3)
                // 替换为: [0xff, 0xee] (长度 2)
                // 结果: [0xff, 0xee, 0xdd]
                let expected = vec![0xff, 0xee, 0xdd];
                assert_eq!(new_data, expected, "应该正确替换包含通配符的匹配");
            }
            _ => panic!("应该返回 Replace 动作"),
        }
    }
    
    #[test]
    fn test_apply_tamper_rules_hits_increment() {
        clear_rules(); // 清理之前的状态
        // 测试命中计数多次增加
        let rule = TamperRule {
            id: "test_hits".to_string(),
            name: "Hits Test".to_string(),
            match_pattern: "bb ?? dd".to_string(),
            replace: "".to_string(),
            action: TamperAction::Block,
            active: true,
            hits: 0,
            hook: HookType::Send,
        };
        
        setup_rules(vec![rule]);
        
        let data = &[0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        
        // 第一次匹配
        let result1 = apply_tamper_rules(data, HookType::Send);
        assert!(result1.is_some());
        
        // 第二次匹配
        let result2 = apply_tamper_rules(data, HookType::Send);
        assert!(result2.is_some());
        
        // 验证命中计数
        let rules_arc = get_rules();
        let rules = rules_arc.lock().unwrap();
        assert_eq!(rules[0].hits, 2, "命中计数应该为 2");
    }
}
