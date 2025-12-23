// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

// 封包数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    packet_counter: u64,
}

// 全局状态
static CAPTURE_STATE: Mutex<CaptureState> = Mutex::new(CaptureState {
    is_capturing: false,
    packet_counter: 0,
});

// 生成模拟封包
fn generate_mock_packet(counter: u64) -> Packet {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let protocols = vec!["TCP", "UDP"];
    let directions = vec!["send", "receive"];

    let protocol_idx = (counter % protocols.len() as u64) as usize;
    let direction_idx = (counter % directions.len() as u64) as usize;

    let protocol = protocols[protocol_idx].to_string();
    let direction = directions[direction_idx].to_string();

    let size = 100 + ((counter * 17) % 900) as u32; // 100-1000 bytes
    let src_port = 50000 + (counter % 10000) as u16;
    let dst_port = 8080;

    Packet {
        id: counter,
        timestamp,
        process_id: 1234 + (counter % 10) as u32,
        process_name: format!("process_{}", counter % 5),
        protocol,
        direction,
        src_addr: format!("192.168.1.{}:{}", 100 + (counter % 155), src_port),
        dst_addr: format!("203.0.113.{}:{}", counter % 255, dst_port),
        size,
        socket: Some(counter % 1000),
        packet_function: Some(format!("Send{}", counter % 4)),
        packet_data: Some(format!("{:02x}", counter % 256)),
    }
}

// 开始抓包命令
#[tauri::command]
async fn start_capture(app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut state = CAPTURE_STATE.lock().map_err(|e| format!("锁定状态失败: {}", e))?;
    
    if state.is_capturing {
        return Err("抓包已在运行中".to_string());
    }

    state.is_capturing = true;
    let packet_counter = state.packet_counter;
    drop(state);

    // 启动模拟抓包线程
    let app_handle_clone = app_handle.clone();
    thread::spawn(move || {
        let mut counter = packet_counter;
        loop {
            // 检查是否还在抓包
            let should_continue = {
                let state = CAPTURE_STATE.lock().unwrap();
                state.is_capturing
            };

            if !should_continue {
                break;
            }

            // 生成并发送封包
            let packet = generate_mock_packet(counter);

            counter += 1;

            // 更新计数器
            {
                let mut state = CAPTURE_STATE.lock().unwrap();
                state.packet_counter = counter;
            }

            // 发送事件到前端
            let _ = app_handle_clone.emit("packet-captured", &packet);

            // 模拟网络延迟，每 100-500ms 发送一个封包
            let delay = 100 + ((counter * 7) % 400);
            thread::sleep(Duration::from_millis(delay));
        }
    });

    Ok(())
}

// 停止抓包命令
#[tauri::command]
async fn stop_capture() -> Result<(), String> {
    let mut state = CAPTURE_STATE.lock().map_err(|e| format!("锁定状态失败: {}", e))?;
    state.is_capturing = false;
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

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_capture,
            stop_capture,
            get_capture_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
