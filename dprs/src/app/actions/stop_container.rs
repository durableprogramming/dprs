use crate::app::state_machine::AppState;
use std::process::Command;

pub fn stop_container(app_state: &mut AppState) -> Result<(), String> {
    let selected = app_state.list_state.selected()
        .ok_or("No container selected")?;
    
    let container = app_state.containers.get(selected)
        .ok_or("Invalid container index")?;
    
    Command::new("docker")
        .args(["stop", &container.name])
        .output()
        .map_err(|e| format!("Failed to stop container: {}", e))?;
    
    // Reload containers to reflect the changes
    app_state.load_containers();
    
    Ok(())
}
