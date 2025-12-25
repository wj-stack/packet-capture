fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    use hook_dll_lib::network_hook::{NetworkHookManager, SendHook, RecvHook, PacketAction};
    use min_hook_rs::*;
    use windows_sys::Win32::Networking::WinSock::*;

    println!("网络 Hook 回调演示");
    println!("==================");

    // 检查是否支持
    if !is_supported() {
        eprintln!("错误: 仅支持 x64 Windows!");
        return Ok(());
    }

    // 初始化 MinHook
    println!("\n初始化 MinHook...");
    initialize()?;

    // 创建 Hook 管理器
    let mut manager = NetworkHookManager::new();

    // 创建 Send Hook 并设置回调
    let mut send_hook = SendHook::new()?;
    send_hook.set_callback(|socket, data, _flags| {
        println!("[回调] Send 回调被调用: socket={}, 数据长度={}", socket, data.len());
        
        // 示例1: 如果数据包含 "block" 关键字，则阻止发送
        if let Ok(text) = std::str::from_utf8(data) {
            if text.contains("block") {
                println!("[回调] 检测到 'block' 关键字，阻止发送");
                return PacketAction::Block;
            }
            
            // 示例2: 如果数据包含 "replace" 关键字，则替换数据
            if text.contains("replace") {
                println!("[回调] 检测到 'replace' 关键字，替换数据");
                let new_data = b"Data has been replaced!\0";
                return PacketAction::Replace(new_data.to_vec());
            }
        }
        
        // 默认允许通过
        PacketAction::Allow
    });

    // 创建 Recv Hook 并设置回调
    let mut recv_hook = RecvHook::new()?;
    recv_hook.set_callback(|socket, data| {
        println!("[回调] Recv 回调被调用: socket={}, 数据长度={}", socket, data.len());
        
        // 示例: 如果接收到的数据包含特定内容，则替换
        if let Ok(text) = std::str::from_utf8(data) {
            if text.contains("original") {
                println!("[回调] 检测到 'original' 关键字，替换接收数据");
                let new_data = b"Received data has been modified!\0";
                return PacketAction::Replace(new_data.to_vec());
            }
        }
        
        // 默认允许通过
        PacketAction::Allow
    });

    manager.add_hook(Box::new(send_hook));
    manager.add_hook(Box::new(recv_hook));

    // 启用所有 Hook
    println!("\n启用所有 Hook...");
    manager.enable_all()?;
    println!("所有 Hook 已启用，回调已设置");

    println!("\n测试说明:");
    println!("1. 发送包含 'block' 的消息会被阻止");
    println!("2. 发送包含 'replace' 的消息会被替换");
    println!("3. 接收包含 '原始' 的消息会被替换");
    println!("\n正在连接到服务器 127.0.0.1:8080...");

    unsafe {
        // 初始化 Winsock
        let mut wsa_data = std::mem::zeroed::<WSADATA>();
        let result = WSAStartup(0x0202, &mut wsa_data);
        if result != 0 {
            eprintln!("WSAStartup failed: {}", result);
        } else {
            // 创建 socket
            let socket = socket(AF_INET as i32, SOCK_STREAM as i32, IPPROTO_TCP as i32);
            if socket != INVALID_SOCKET {
                // 设置服务器地址
                let mut addr: SOCKADDR_IN = std::mem::zeroed();
                addr.sin_family = AF_INET as u16;
                addr.sin_port = (8080u16).to_be();
                let ip_str = b"127.0.0.1\0";
                addr.sin_addr.S_un.S_addr = inet_addr(ip_str.as_ptr());
                
                // 尝试连接
                let connect_result = connect(socket, &addr as *const _ as *const SOCKADDR, std::mem::size_of::<SOCKADDR_IN>() as i32);
                if connect_result == 0 {
                    println!("✓ 连接成功！\n");
                    
                    // 测试1: 正常消息（应该通过）
                    println!("[测试1] 发送正常消息...");
                    let msg1 = b"Hello, Server!\0";
                    let send_result = send(socket, msg1.as_ptr(), msg1.len() as i32 - 1, 0);
                    println!("发送结果: {} 字节\n", send_result);
                    
                    // 测试2: 包含 'block' 的消息（应该被阻止）
                    println!("[测试2] 发送包含 'block' 的消息（应该被阻止）...");
                    let msg2 = b"This message should be blocked\0";
                    let send_result = send(socket, msg2.as_ptr(), msg2.len() as i32 - 1, 0);
                    println!("发送结果: {} 字节（如果等于消息长度，说明被阻止）\n", send_result);
                    
                    // 测试3: 包含 'replace' 的消息（应该被替换）
                    println!("[测试3] 发送包含 'replace' 的消息（应该被替换）...");
                    let msg3 = b"This message should be replaced\0";
                    let send_result = send(socket, msg3.as_ptr(), msg3.len() as i32 - 1, 0);
                    println!("发送结果: {} 字节\n", send_result);
                    
                    // 接收响应
                    let mut buffer = [0u8; 1024];
                    let recv_result = recv(socket, buffer.as_mut_ptr(), buffer.len() as i32, 0);
                    if recv_result > 0 {
                        println!("收到响应: {}", String::from_utf8_lossy(&buffer[..recv_result as usize]));
                    }
                } else {
                    let error = WSAGetLastError();
                    eprintln!("连接失败，错误代码: {}", error);
                    eprintln!("提示: 请先运行 network_server_demo 启动服务器");
                }
                
                closesocket(socket);
            }
            WSACleanup();
        }
    }

    println!("\n按 Enter 键禁用 Hook...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // 禁用所有 Hook
    println!("\n禁用所有 Hook...");
    manager.disable_all()?;
    println!("所有 Hook 已禁用");

    // 清理资源
    println!("\n清理资源...");
    manager.cleanup_all()?;
    uninitialize()?;
    println!("清理完成");

    println!("\n演示完成!");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("此示例仅支持 Windows");
}

