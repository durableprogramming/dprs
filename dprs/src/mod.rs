mod app;
mod display;

use app::state_machine::{AppState, AppEvent};
use app::actions::{copy_ip_address, open_browser, stop_container};
use display::{render_container_list, render_hotkey_bar, ToastManager};
