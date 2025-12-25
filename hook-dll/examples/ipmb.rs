use ipmb::label;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main () -> Result<(), Box<dyn Error>> {
    // Join your bus 
    let options = ipmb::Options::new("com.solar", label!("moon"), "");
    let (sender, _receiver) = ipmb::join::<String, String>(options, None)?;

    loop {
        // Create a message
        let selector = ipmb::Selector::unicast("earth");
        let  message = ipmb::Message::new(selector, "hello world".to_string());

        // Send the message
        sender.send(message)?;
        
        thread::sleep(Duration::from_secs(1));
    }
}
