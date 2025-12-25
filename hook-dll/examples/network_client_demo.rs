fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {

    println!("网络 Client 演示");
    println!("================");
    println!("正在连接到服务器 127.0.0.1:8080...");
    
    // 执行网络通信（使用 Windows Winsock API）
    use windows_sys::Win32::Networking::WinSock::*;
    
    // 按下回车键开始通信
    println!("按下回车键开始通信");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
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
                    
                    // 发送多条测试消息
                    let test_messages = vec![
                        "Hello, Server!",
                        "这是第二条消息",
                        "Testing hook functionality",
                        "最后一条测试消息",
                    ];
                    
                    for (i, msg) in test_messages.iter().enumerate() {
                        println!("[测试 {}] 发送消息: {}", i + 1, msg);
                        let data = msg.as_bytes();
                        let send_result = send(socket, data.as_ptr(), data.len() as i32, 0);
                        
                        if send_result == SOCKET_ERROR {
                            let error = WSAGetLastError();
                            eprintln!("发送失败，错误代码: {}", error);
                            break;
                        }
                        
                        println!("  → 已发送 {} 字节\n", send_result);
                        
                        // 接收响应
                        let mut buffer = [0u8; 1024];
                        let recv_result = recv(socket, buffer.as_mut_ptr(), buffer.len() as i32, 0);
                        
                        if recv_result > 0 {
                            println!("  ← 收到响应: {}", String::from_utf8_lossy(&buffer[..recv_result as usize]));
                        } else if recv_result == SOCKET_ERROR {
                            let error = WSAGetLastError();
                            eprintln!("接收失败，错误代码: {}", error);
                        }
                        
                        println!();
                        
                        // 短暂延迟
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                    
                    println!("所有测试消息已发送完成！");
                } else {
                    let error = WSAGetLastError();
                    eprintln!("连接失败，错误代码: {}", error);
                    eprintln!("提示: 请先运行 network_server_demo 启动服务器");
                }
                
                closesocket(socket);
            } else {
                let error = WSAGetLastError();
                eprintln!("创建 socket 失败，错误代码: {}", error);
            }
            WSACleanup();
        }
    }
    println!("\n演示完成!");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("此示例仅支持 Windows");
}

