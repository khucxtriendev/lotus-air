use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WindowsConnection {
    pub connected: bool,
    pub name: Option<String>,
}

#[tauri::command]
pub fn get_windows_connection() -> Result<WindowsConnection, String> {
    if let Some(dev) = lotusair_driver::find_connected_airpods() {
        Ok(WindowsConnection {
            connected: dev.connected,
            name: Some(dev.name),
        })
    } else {
        Ok(WindowsConnection {
            connected: false,
            name: None,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn probe_windows_airpods() {
        println!("Windows Bluetooth: {:?}", super::get_windows_connection());
    }
}
