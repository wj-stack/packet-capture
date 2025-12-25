use std::{error::Error, sync::Mutex};

use hook_dll_lib::HookCommand;
use ipmb::{Message, Options, Selector, label};
use std::sync::OnceLock;
fn main () -> Result<(), Box<dyn Error>> {
    hook_dll_lib::test_main()
}

