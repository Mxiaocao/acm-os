mod ipc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _infrastructure = acm_os_infrastructure::Infrastructure::default();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ipc::foundation_status])
        .run(tauri::generate_context!())
        .expect("error while running ACM-OS");
}
