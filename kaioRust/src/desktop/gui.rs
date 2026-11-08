use crate::desktop::app::KaiokenDesktopController;
use anyhow::Result;
use serde_json::{json, Value};
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tao::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{CursorIcon, ResizeDirection, WindowBuilder},
};
use wry::{
    http::{header::CONTENT_TYPE, Response},
    WebViewBuilder,
};

const DESIGN_CSS: &str = include_str!("ui/design.css");
const APP_HTML: &str = include_str!("ui/app.html");
const APP_JS: &str = include_str!("ui/app.js");

const POLL_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy)]
enum UserEvent {
    StateChanged,
    CloseWindow,
    Minimize,
    Maximize,
    DragWindow,
    MouseDown(i32, i32),
    MouseMove(i32, i32),
}

#[derive(Debug, Clone, Copy)]
enum HitTestResult {
    Client,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    NoWhere,
}

impl HitTestResult {
    fn drag_resize_window(self, window: &tao::window::Window) {
        let direction = match self {
            HitTestResult::Left => ResizeDirection::West,
            HitTestResult::Right => ResizeDirection::East,
            HitTestResult::Top => ResizeDirection::North,
            HitTestResult::Bottom => ResizeDirection::South,
            HitTestResult::TopLeft => ResizeDirection::NorthWest,
            HitTestResult::TopRight => ResizeDirection::NorthEast,
            HitTestResult::BottomLeft => ResizeDirection::SouthWest,
            HitTestResult::BottomRight => ResizeDirection::SouthEast,
            HitTestResult::Client | HitTestResult::NoWhere => return,
        };
        let _ = window.drag_resize_window(direction);
    }

    fn change_cursor(self, window: &tao::window::Window) {
        window.set_cursor_icon(match self {
            HitTestResult::Left => CursorIcon::WResize,
            HitTestResult::Right => CursorIcon::EResize,
            HitTestResult::Top => CursorIcon::NResize,
            HitTestResult::Bottom => CursorIcon::SResize,
            HitTestResult::TopLeft => CursorIcon::NwResize,
            HitTestResult::TopRight => CursorIcon::NeResize,
            HitTestResult::BottomLeft => CursorIcon::SwResize,
            HitTestResult::BottomRight => CursorIcon::SeResize,
            HitTestResult::Client | HitTestResult::NoWhere => CursorIcon::Default,
        });
    }
}

fn hit_test(window_size: PhysicalSize<u32>, x: i32, y: i32, scale: f64) -> HitTestResult {
    const BORDERLESS_RESIZE_INSET: f64 = 5.0;
    const CLIENT: isize = 0b0000;
    const LEFT: isize = 0b0001;
    const RIGHT: isize = 0b0010;
    const TOP: isize = 0b0100;
    const BOTTOM: isize = 0b1000;
    const TOPLEFT: isize = TOP | LEFT;
    const TOPRIGHT: isize = TOP | RIGHT;
    const BOTTOMLEFT: isize = BOTTOM | LEFT;
    const BOTTOMRIGHT: isize = BOTTOM | RIGHT;

    let top = 0;
    let left = 0;
    let bottom = top + window_size.height as i32;
    let right = left + window_size.width as i32;
    let inset = (BORDERLESS_RESIZE_INSET * scale) as i32;

    let result = (LEFT * (if x < left + inset { 1 } else { 0 }))
        | (RIGHT * (if x >= right - inset { 1 } else { 0 }))
        | (TOP * (if y < top + inset { 1 } else { 0 }))
        | (BOTTOM * (if y >= bottom - inset { 1 } else { 0 }));

    match result {
        CLIENT => HitTestResult::Client,
        LEFT => HitTestResult::Left,
        RIGHT => HitTestResult::Right,
        TOP => HitTestResult::Top,
        BOTTOM => HitTestResult::Bottom,
        TOPLEFT => HitTestResult::TopLeft,
        TOPRIGHT => HitTestResult::TopRight,
        BOTTOMLEFT => HitTestResult::BottomLeft,
        BOTTOMRIGHT => HitTestResult::BottomRight,
        _ => HitTestResult::NoWhere,
    }
}

pub fn launch_desktop_gui(controller: Arc<Mutex<KaiokenDesktopController>>) -> Result<()> {
    let mut builder = EventLoopBuilder::<UserEvent>::with_user_event();
    let event_loop = builder.build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Kaioken")
        .with_decorations(false)
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(1360.0, 880.0))
        .with_min_inner_size(LogicalSize::new(1080.0, 680.0))
        .build(&event_loop)?;

    let initial_state = {
        let ctrl = controller
            .lock()
            .map_err(|_| anyhow::anyhow!("desktop controller lock poisoned"))?;
        ctrl.window.snapshot_json()
    };
    let html = Arc::new(build_html(&initial_state)?);

    // A real origin (rather than about:blank from with_html) is required for
    // localStorage theme persistence. The protocol serves the single embedded
    // document; every asset is already inlined in it.
    let html_for_protocol = html.clone();
    let controller_for_ipc = controller.clone();
    let proxy_for_ipc = proxy.clone();

    let webview = WebViewBuilder::new(&window)
        .with_custom_protocol("wry".into(), move |request| {
            if request.uri().path() == "/" || request.uri().path() == "/index.html" {
                Response::builder()
                    .header(CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Cow::Owned(html_for_protocol.as_bytes().to_vec()))
                    .unwrap()
            } else {
                Response::builder()
                    .status(404)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Cow::Borrowed(b"not found".as_slice()))
                    .unwrap()
            }
        })
        .with_url("wry://localhost/")
        .with_ipc_handler(move |request: wry::http::Request<String>| {
            handle_ipc(request.body(), &controller_for_ipc, &proxy_for_ipc);
        })
        .build()?;

    let mut last_revision = controller
        .lock()
        .map(|ctrl| ctrl.revision)
        .unwrap_or_default();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + POLL_INTERVAL);

        match event {
            Event::NewEvents(StartCause::Init) => {
                let (mode, endpoint) = controller
                    .lock()
                    .map(|ctrl| {
                        (
                            ctrl.window.settings.daemon_mode.clone(),
                            ctrl.window.settings.endpoint.clone(),
                        )
                    })
                    .unwrap_or_else(|_| ("unknown".to_string(), "unknown".to_string()));
                println!(
                    "🚀 Kaioken Desktop · tao + WebView2 · {} ({})",
                    mode, endpoint
                );
                let _ = webview.evaluate_script("window.KAI && window.KAI.boot()");
            }
            Event::UserEvent(UserEvent::StateChanged) => {
                push_state(&webview, &controller);
                last_revision = controller
                    .lock()
                    .map(|ctrl| ctrl.revision)
                    .unwrap_or(last_revision);
            }
            Event::UserEvent(UserEvent::CloseWindow) => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::Minimize) => {
                window.set_minimized(true);
            }
            Event::UserEvent(UserEvent::Maximize) => {
                window.set_maximized(!window.is_maximized());
            }
            Event::UserEvent(UserEvent::DragWindow) => {
                let _ = window.drag_window();
            }
            Event::UserEvent(UserEvent::MouseDown(x, y)) => {
                let hit = hit_test(window.inner_size(), x, y, window.scale_factor());
                hit.drag_resize_window(&window);
            }
            Event::UserEvent(UserEvent::MouseMove(x, y)) => {
                hit_test(window.inner_size(), x, y, window.scale_factor()).change_cursor(&window);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                let revision = match controller.lock() {
                    Ok(mut ctrl) => {
                        ctrl.poll_events();
                        ctrl.revision
                    }
                    Err(_) => last_revision,
                };
                if revision != last_revision {
                    last_revision = revision;
                    push_state(&webview, &controller);
                }
            }
            _ => {}
        }
    });
}

fn push_state(webview: &wry::WebView, controller: &Arc<Mutex<KaiokenDesktopController>>) {
    let state = match controller.lock() {
        Ok(ctrl) => ctrl.window.snapshot_json(),
        Err(_) => return,
    };
    let json = match serde_json::to_string(&state) {
        Ok(json) => json.replace("</", "<\\/"),
        Err(_) => return,
    };
    let script = format!(
        "if (window.KAI && typeof window.KAI.mergeState === 'function') {{ window.KAI.mergeState({json}); }}"
    );
    let _ = webview.evaluate_script(&script);
}

fn handle_ipc(
    body: &str,
    controller: &Arc<Mutex<KaiokenDesktopController>>,
    proxy: &EventLoopProxy<UserEvent>,
) {
    let Ok(message) = serde_json::from_str::<Value>(body) else {
        return;
    };
    let Some(action) = message.get("action").and_then(Value::as_str) else {
        return;
    };

    // Window chrome never needs the controller lock.
    match action {
        "window_minimize" | "window-minimize" => {
            let _ = proxy.send_event(UserEvent::Minimize);
            return;
        }
        "window_maximize" | "window-maximize" => {
            let _ = proxy.send_event(UserEvent::Maximize);
            return;
        }
        "window_close" | "window-close" => {
            let _ = proxy.send_event(UserEvent::CloseWindow);
            return;
        }
        "window_drag" | "window-drag" => {
            let _ = proxy.send_event(UserEvent::DragWindow);
            return;
        }
        "mouse_down" => {
            let x = message.get("x").and_then(Value::as_i64).unwrap_or(0) as i32;
            let y = message.get("y").and_then(Value::as_i64).unwrap_or(0) as i32;
            let _ = proxy.send_event(UserEvent::MouseDown(x, y));
            return;
        }
        "mouse_move" => {
            let x = message.get("x").and_then(Value::as_i64).unwrap_or(0) as i32;
            let y = message.get("y").and_then(Value::as_i64).unwrap_or(0) as i32;
            let _ = proxy.send_event(UserEvent::MouseMove(x, y));
            return;
        }
        _ => {}
    }

    let Ok(mut ctrl) = controller.lock() else {
        return;
    };

    let mut changed = true;
    match action {
        "switch_tab" => {
            if let Some(tab) = message.get("tab").and_then(Value::as_str) {
                changed = ctrl.switch_tab(tab);
            } else {
                changed = false;
            }
        }
        "approve_tool" => {
            let id = message.get("proposal_id").and_then(Value::as_str);
            changed = ctrl.approve_pending(id);
        }
        "deny_tool" => {
            let id = message.get("proposal_id").and_then(Value::as_str);
            changed = ctrl.deny_pending(id);
        }
        "send_chat" => {
            if let Some(prompt) = message.get("prompt").and_then(Value::as_str) {
                ctrl.send_chat(prompt);
            } else {
                changed = false;
            }
        }
        "new_session" => {
            ctrl.window.new_chat_session();
            ctrl.touch();
        }
        "compact_session" => {
            ctrl.window.compact_session();
            ctrl.touch();
        }
        "start_research" => {
            if let Some(query) = message.get("query").and_then(Value::as_str) {
                let power = message
                    .get("power_level")
                    .and_then(Value::as_u64)
                    .map(|value| value as u8);
                ctrl.start_research(query, power);
            } else {
                changed = false;
            }
        }
        "set_power_level" => {
            if let Some(power) = message.get("power_level").and_then(Value::as_u64) {
                ctrl.set_power_level(power as u8);
            } else {
                changed = false;
            }
        }
        "research_tick" => {
            ctrl.research_tick();
        }
        "toggle_theme" => {
            if let Some(theme) = message.get("theme").and_then(Value::as_str) {
                let patch = json!({ "theme": theme });
                ctrl.update_settings(&patch);
            } else {
                ctrl.window.toggle_theme();
                ctrl.touch();
            }
        }
        "run_command_action" => {
            if let Some(command) = message.get("command").and_then(Value::as_str) {
                if command == "open_url" {
                    if let Some(url) = message.get("url").and_then(Value::as_str) {
                        let _ = ctrl.execute_browser_request(url, "GET");
                    }
                } else {
                    ctrl.run_command_action(command);
                }
            } else {
                changed = false;
            }
        }
        "update_settings" => {
            if let Some(settings) = message.get("settings") {
                ctrl.update_settings(settings);
            } else {
                ctrl.update_settings(&message);
            }
        }
        "read_file" | "select_file" => {
            if let Some(path) = message.get("path").and_then(Value::as_str) {
                let _ = ctrl.open_file_in_editor(path);
            } else {
                changed = false;
            }
        }
        "save_file" => {
            if let (Some(path), Some(content)) = (
                message.get("path").and_then(Value::as_str),
                message.get("content").and_then(Value::as_str),
            ) {
                let _ = ctrl.save_file_from_editor(path, content);
            } else {
                changed = false;
            }
        }
        "terminal_input" => {
            if let Some(command) = message.get("command").and_then(Value::as_str) {
                let _ = ctrl.execute_terminal_command(command);
            } else {
                changed = false;
            }
        }
        "browser_request" => {
            if let Some(url) = message.get("url").and_then(Value::as_str) {
                let method = message.get("method").and_then(Value::as_str).unwrap_or("GET");
                let _ = ctrl.execute_browser_request(url, method);
            } else {
                changed = false;
            }
        }
        "resolve_failure" => {
            if let Some(id) = message.get("id").and_then(Value::as_str) {
                let fix = message.get("fix_summary").and_then(Value::as_str).unwrap_or("Resolved in UI");
                let _ = ctrl.resolve_failure_by_id(id, fix);
            } else {
                changed = false;
            }
        }
        "toggle_git_hook" => {
            let install = message.get("install").and_then(Value::as_bool).unwrap_or(true);
            let _ = ctrl.toggle_git_hook(install);
        }
        "test_api_key" => {
            if let (Some(provider), Some(key)) = (
                message.get("provider").and_then(Value::as_str),
                message.get("key").and_then(Value::as_str),
            ) {
                let _ = ctrl.test_api_key(provider, key);
            } else {
                changed = false;
            }
        }
        "save_config_key" => {
            if let (Some(provider), Some(key)) = (
                message.get("provider").and_then(Value::as_str),
                message.get("key").and_then(Value::as_str),
            ) {
                let _ = ctrl.save_config_key(provider, key);
            } else {
                changed = false;
            }
        }
        _ => {
            changed = false;
        }
    }

    if changed {
        ctrl.touch();
        drop(ctrl);
        let _ = proxy.send_event(UserEvent::StateChanged);
    }
}

fn build_html(initial_state: &Value) -> Result<String> {
    let state = serde_json::to_string(initial_state)?.replace("</", "<\\/");
    Ok(APP_HTML
        .replace("/*__KAIOKEN_DESIGN_CSS__*/", DESIGN_CSS)
        .replace("/*__KAIOKEN_INITIAL_STATE__*/", &state)
        .replace("/*__KAIOKEN_APP_JS__*/", APP_JS))
}
