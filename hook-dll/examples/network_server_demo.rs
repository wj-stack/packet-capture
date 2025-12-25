#[cfg(target_os = "windows")]
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    use windows_sys::Win32::Networking::WinSock::*;

    println!("网络服务器演示 - 用于测试 Hook");
    println!("==================================");

    unsafe {
        // 初始化 Winsock
        let mut wsa_data = std::mem::zeroed::<WSADATA>();
        let result = WSAStartup(0x0202, &mut wsa_data);
        if result != 0 {
            eprintln!("WSAStartup 失败: {}", result);
            return Err(format!("WSAStartup failed: {}", result).into());
        }

        // 创建服务器 socket
        let server_socket = socket(AF_INET as i32, SOCK_STREAM as i32, IPPROTO_TCP as i32);
        if server_socket == INVALID_SOCKET {
            let error = WSAGetLastError();
            WSACleanup();
            return Err(format!("创建 socket 失败，错误代码: {}", error).into());
        }

        // 设置 socket 选项：允许地址重用
        let reuse = 1i32;
        setsockopt(
            server_socket,
            SOL_SOCKET as i32,
            SO_REUSEADDR as i32,
            &reuse as *const _ as *const u8,
            std::mem::size_of::<i32>() as i32,
        );

        // 设置服务器地址
        let mut server_addr: SOCKADDR_IN = std::mem::zeroed();
        server_addr.sin_family = AF_INET as u16;
        server_addr.sin_port = (8080u16).to_be();
        server_addr.sin_addr.S_un.S_addr = INADDR_ANY;

        // 绑定地址
        let bind_result = bind(
            server_socket,
            &server_addr as *const _ as *const SOCKADDR,
            std::mem::size_of::<SOCKADDR_IN>() as i32,
        );

        if bind_result == SOCKET_ERROR {
            let error = WSAGetLastError();
            closesocket(server_socket);
            WSACleanup();
            return Err(format!("绑定地址失败，错误代码: {}", error).into());
        }

        // 开始监听
        let listen_result = listen(server_socket, 5);
        if listen_result == SOCKET_ERROR {
            let error = WSAGetLastError();
            closesocket(server_socket);
            WSACleanup();
            return Err(format!("监听失败，错误代码: {}", error).into());
        }

        println!("服务器已启动，监听地址: 127.0.0.1:8080");
        println!("等待客户端连接...");
        println!("按 Ctrl+C 退出\n");

        // 接受客户端连接
        loop {
            let mut client_addr: SOCKADDR_IN = std::mem::zeroed();
            let mut client_addr_len = std::mem::size_of::<SOCKADDR_IN>() as i32;

            let client_socket = accept(
                server_socket,
                &mut client_addr as *mut _ as *mut SOCKADDR,
                &mut client_addr_len,
            );

            if client_socket == INVALID_SOCKET {
                let error = WSAGetLastError();
                eprintln!("接受连接失败，错误代码: {}", error);
                continue;
            }

            // 获取客户端 IP 地址
            let client_ip = u32::from_be(client_addr.sin_addr.S_un.S_addr);
            let ip_bytes = client_ip.to_be_bytes();
            println!(
                "\n客户端已连接: {}.{}.{}.{}:{}",
                ip_bytes[0],
                ip_bytes[1],
                ip_bytes[2],
                ip_bytes[3],
                u16::from_be(client_addr.sin_port)
            );

            // 处理客户端连接
            handle_client(client_socket)?;
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn handle_client(client_socket: windows_sys::Win32::Networking::WinSock::SOCKET) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use windows_sys::Win32::Networking::WinSock::*;

    let mut buffer = [0u8; 1024];

    loop {
        // 接收数据
        let recv_result = recv(
            client_socket,
            buffer.as_mut_ptr(),
            buffer.len() as i32,
            0,
        );

        if recv_result == SOCKET_ERROR {
            let error = WSAGetLastError();
            if error == WSAECONNRESET || error == WSAECONNABORTED {
                println!("客户端断开连接");
                break;
            }
            eprintln!("接收数据失败，错误代码: {}", error);
            break;
        }

        if recv_result == 0 {
            println!("客户端正常关闭连接");
            break;
        }

        // 打印接收到的数据
        let received_data = &buffer[..recv_result as usize];
        println!("收到 {} 字节数据:", recv_result);
        println!("  内容: {}", String::from_utf8_lossy(received_data));
        println!("  十六进制: {:?}", received_data);

        // 发送响应
        let response = format!("服务器收到: {}\n", String::from_utf8_lossy(received_data));
        let send_result = send(
            client_socket,
            response.as_ptr(),
            response.len() as i32,
            0,
        );

        if send_result == SOCKET_ERROR {
            let error = WSAGetLastError();
            eprintln!("发送数据失败，错误代码: {}", error);
            break;
        }

        println!("已发送 {} 字节响应\n", send_result);
    }

    closesocket(client_socket);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("此示例仅支持 Windows");
}

