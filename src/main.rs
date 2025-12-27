#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod backend;
mod state;
mod ui;
mod workers;
use app::KovaaksApp;
use backend::{initialize_installation_path, is_kovaaks_running, load_settings, DiscordRPC};
use eframe::egui;
use state::{create_tray_channel, create_ui_channel, AppState, TrayMessage};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};
fn main() {

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut settings = load_settings().unwrap_or_default();
    let _ = initialize_installation_path(&mut settings);

    let (ui_tx, ui_rx) = create_ui_channel();
    let (tray_tx, tray_rx) = create_tray_channel();
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel::<()>();

    let app_state = match AppState::new(settings.clone(), ui_tx) {
        Ok(state) => Arc::new(state),
        Err(_) => return,
    };

    let menu = Menu::new();
    let show_item = MenuItem::new("Show Window", true, None);
    let separator1 = PredefinedMenuItem::separator();
    let start_item = MenuItem::new("Start RPC", true, None);
    let stop_item = MenuItem::new("Stop RPC", true, None);
    let separator2 = PredefinedMenuItem::separator();
    let quit_item = MenuItem::new("Exit", true, None);
    menu.append_items(&[
        &show_item,
        &separator1,
        &start_item,
        &stop_item,
        &separator2,
        &quit_item,
    ])
    .expect("Failed to build tray menu");

    let icon = load_tray_icon();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Kovaaks Discord RPC")
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon");

    let tray_tx_clone = tray_tx.clone();
    let show_id = show_item.id().clone();
    let start_id = start_item.id().clone();
    let stop_id = stop_item.id().clone();
    let quit_id = quit_item.id().clone();
    let tray_thread_handle = std::thread::spawn(move || {
        let menu_channel = MenuEvent::receiver();
        let tray_channel = TrayIconEvent::receiver();
        loop {
            if shutdown_rx.try_recv().is_ok() {
                break;
            }

            if let Ok(event) = tray_channel.try_recv() {
                match event {
                    TrayIconEvent::DoubleClick { .. } => {
                        #[cfg(windows)]
                        {
                            use windows::core::PCWSTR;
                            use windows::Win32::UI::WindowsAndMessaging::{
                                FindWindowW, ShowWindow, SetForegroundWindow, SW_SHOW,
                            };
                            let title: Vec<u16> = "Kovaaks Discord RPC\0".encode_utf16().collect();
                            if let Ok(hwnd) = unsafe { FindWindowW(None, PCWSTR(title.as_ptr())) } {
                                if !hwnd.is_invalid() {
                                    unsafe {
                                        let _ = ShowWindow(hwnd, SW_SHOW);
                                        let _ = SetForegroundWindow(hwnd);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            match menu_channel.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(event) => {
                    if event.id == show_id {
                        #[cfg(windows)]
                        {
                            use windows::core::PCWSTR;
                            use windows::Win32::UI::WindowsAndMessaging::{
                                FindWindowW, ShowWindow, SetForegroundWindow, SW_SHOW,
                            };
                            let title: Vec<u16> = "Kovaaks Discord RPC\0".encode_utf16().collect();
                            if let Ok(hwnd) = unsafe { FindWindowW(None, PCWSTR(title.as_ptr())) } {
                                if !hwnd.is_invalid() {
                                    unsafe {
                                        let _ = ShowWindow(hwnd, SW_SHOW);
                                        let _ = SetForegroundWindow(hwnd);
                                    }
                                }
                            }
                        }
                    } else if event.id == start_id {
                        let _ = tray_tx_clone.send(TrayMessage::StartRpc);
                    } else if event.id == stop_id {
                        let _ = tray_tx_clone.send(TrayMessage::StopRpc);
                    } else if event.id == quit_id {
                        std::process::exit(0);
                        break;
                    }
                }
                Err(_) => continue,
            }
        }
    });

    if !settings.open_manually && is_kovaaks_running() {

        let state = app_state.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2));
            if let Ok(mut rpc) = DiscordRPC::new() {
                if rpc.connect().is_ok() {
                    *state.rpc.lock() = Some(rpc);
                    state.rpc_running.store(true, Ordering::Relaxed);
                    *state.start_time.lock() = Some(chrono::Utc::now().timestamp());
                    *state.session_start_time.lock() = std::time::SystemTime::now();
                    workers::start_monitoring_thread(state);
                }
            }
        });
    }

    let start_visible = !settings.start_in_tray;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 450.0])
            .with_title("Kovaaks Discord RPC")
            .with_visible(start_visible),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Kovaaks Discord RPC",
        native_options,
        Box::new(move |cc| Ok(Box::new(KovaaksApp::new(cc, app_state, tray_rx, ui_rx, tray_icon, shutdown_tx, tray_thread_handle)))),
    );
}
fn load_tray_icon() -> tray_icon::Icon {
    let icon_data = include_bytes!("../assets/icon.png");

    if let Ok(img) = image::load_from_memory(icon_data) {
        let resized = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let (width, height) = rgba.dimensions();
        if let Ok(icon) = tray_icon::Icon::from_rgba(rgba.into_raw(), width, height) {
            return icon;
        }
    }

    let size = 16u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for _ in 0..(size * size) {
        rgba.extend_from_slice(&[100, 149, 237, 255]);
    }
    tray_icon::Icon::from_rgba(rgba, size, size).expect("Failed to create default icon")
}