use yew::prelude::*;
use gloo_net::websocket::{futures::WebSocket, Message};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, MouseEvent, HtmlInputElement, HtmlElement};
use serde::{Serialize, Deserialize};
use futures_util::sink::SinkExt;
use futures_util::stream::{StreamExt, SplitSink};
use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
enum WhiteboardEvent {
    DrawFreehand { x: f64, y: f64, dragging: bool },
    DrawLine { from: (f64, f64), to: (f64, f64), color: String, width: f64 },
    DrawRect { from: (f64, f64), to: (f64, f64), color: String, width: f64 },
    DrawCircle { center: (f64, f64), radius: f64, color: String, width: f64 },
    AddText { pos: (f64, f64), text: String, color: String, size: f64 },
    Pan { dx: f64, dy: f64 },
    Zoom { factor: f64 },
}

#[derive(Clone, PartialEq, Debug)]
enum Tool {
    Freehand,
    Line,
    Rect,
    Circle,
    Text,
    Pan,
    Zoom,
}

impl Tool {
    fn icon(&self) -> &'static str {
        match self {
            Tool::Freehand => "✏️",
            Tool::Line => "📏",
            Tool::Rect => "⬜",
            Tool::Circle => "⭕",
            Tool::Text => "🅰️",
            Tool::Pan => "👆",
            Tool::Zoom => "🔍",
        }
    }

    fn cursor(&self) -> &'static str {
        match self {
            Tool::Freehand => "url('data:image/svg+xml;utf8,<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\"><circle cx=\"12\" cy=\"12\" r=\"2\" fill=\"%23000\"/></svg>') 12 12, auto",
            Tool::Line => "crosshair",
            Tool::Rect => "crosshair",
            Tool::Circle => "crosshair",
            Tool::Text => "text",
            Tool::Pan => "grab",
            Tool::Zoom => "zoom-in",
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let canvas_ref = use_node_ref();
    let drawing = use_state(|| false);
    let ws = use_mut_ref(|| None::<Rc<RefCell<SplitSink<WebSocket, Message>>>>);
    let last_pos = use_mut_ref(|| (0.0, 0.0));
    let tool = use_state(|| Tool::Freehand);
    let color = use_state(|| "#2563eb".to_string());
    let width = use_state(|| 3.0);
    let start_pos = use_mut_ref(|| None::<(f64, f64)>);
    let text_input = use_state(|| None::<(f64, f64)>);
    let pan = use_state(|| (0.0, 0.0));
    let zoom = use_state(|| 1.0);
    let pan_start = use_mut_ref(|| None::<(f64, f64)>);
    let connection_status = use_state(|| "connecting".to_string());

    // Connect to backend WebSocket
    {
        let ws = ws.clone();
        let canvas_ref = canvas_ref.clone();
        let pan = pan.clone();
        let zoom = zoom.clone();
        let connection_status = connection_status.clone();
        use_effect_with((), move |_| {
            let pan = pan.clone();
            let zoom = zoom.clone();
            let connection_status = connection_status.clone();
            spawn_local(async move {
                let ws_url = "ws://127.0.0.1:3000/ws";
                match WebSocket::open(ws_url) {
                    Ok(socket) => {
                        connection_status.set("connected".to_string());
                        let (write, read) = socket.split();
                        
                        // Spawn a task to handle incoming messages
                        let canvas_ref = canvas_ref.clone();
                        let pan = pan.clone();
                        let zoom = zoom.clone();
                        let connection_status = connection_status.clone();
                        spawn_local(async move {
                            let mut read = read;
                            while let Some(msg) = read.next().await {
                                match msg {
                                    Ok(Message::Text(txt)) => {
                                        if let Ok(event) = serde_json::from_str::<WhiteboardEvent>(&txt) {
                                            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                                                let ctx = canvas
                                                    .get_context("2d")
                                                    .unwrap()
                                                    .unwrap()
                                                    .dyn_into::<CanvasRenderingContext2d>()
                                                    .unwrap();
                                                
                                                ctx.save();
                                                ctx.translate(pan.0, pan.1).unwrap();
                                                ctx.scale(*zoom, *zoom).unwrap();
                                                
                                                match event {
                                                    WhiteboardEvent::DrawFreehand { x, y, .. } => {
                                                        ctx.set_fill_style_str("#2563eb");
                                                        ctx.begin_path();
                                                        ctx.arc(x, y, 2.0, 0.0, std::f64::consts::PI * 2.0).unwrap();
                                                        ctx.fill();
                                                    }
                                                    WhiteboardEvent::DrawLine { from, to, color, width } => {
                                                        ctx.begin_path();
                                                        ctx.set_stroke_style_str(&color);
                                                        ctx.set_line_width(width);
                                                        ctx.set_line_cap("round");
                                                        ctx.move_to(from.0, from.1);
                                                        ctx.line_to(to.0, to.1);
                                                        ctx.stroke();
                                                    }
                                                    WhiteboardEvent::DrawRect { from, to, color, width } => {
                                                        ctx.begin_path();
                                                        ctx.set_stroke_style_str(&color);
                                                        ctx.set_line_width(width);
                                                        ctx.set_line_cap("round");
                                                        ctx.stroke_rect(from.0, from.1, to.0 - from.0, to.1 - from.1);
                                                    }
                                                    WhiteboardEvent::DrawCircle { center, radius, color, width } => {
                                                        ctx.begin_path();
                                                        ctx.set_stroke_style_str(&color);
                                                        ctx.set_line_width(width);
                                                        ctx.set_line_cap("round");
                                                        ctx.arc(center.0, center.1, radius, 0.0, std::f64::consts::PI * 2.0).unwrap();
                                                        ctx.stroke();
                                                    }
                                                    WhiteboardEvent::AddText { pos, text, color, size } => {
                                                        ctx.set_fill_style_str(&color);
                                                        ctx.set_font(&format!("{}px 'Inter', -apple-system, system-ui, sans-serif", size));
                                                        ctx.fill_text(&text, pos.0, pos.1).unwrap();
                                                    }
                                                    _ => {}
                                                }
                                                ctx.restore();
                                            }
                                        }
                                    }
                                    Ok(Message::Bytes(_)) => {
                                        // Handle binary messages if needed
                                    }
                                    Err(_) => {
                                        connection_status.set("disconnected".to_string());
                                        break;
                                    }
                                }
                            }
                        });
                        *ws.borrow_mut() = Some(Rc::new(RefCell::new(write)));
                    }
                    Err(_) => {
                        connection_status.set("failed".to_string());
                    }
                }
            });
            || ()
        });
    }

    // Color palette
    let colors = vec![
        "#2563eb", "#dc2626", "#059669", "#d97706", "#7c3aed", 
        "#db2777", "#0891b2", "#65a30d", "#be185d", "#0f172a"
    ];

    // Toolbar UI
    let toolbar = html! {
        <div class="toolbar">
            <div class="toolbar-section">
                <h3 class="toolbar-title">{"🎨 Whiteboard"}</h3>
                <div class="connection-status" data-status={connection_status.to_string()}>
                    {match connection_status.as_str() {
                        "connected" => "🟢 Connected",
                        "connecting" => "🟡 Connecting...",
                        "disconnected" => "🔴 Disconnected",
                        _ => "🔴 Failed to connect"
                    }}
                </div>
            </div>
            
            <div class="toolbar-section">
                <label class="toolbar-label">{"Tools"}</label>
                <div class="tool-buttons">
                    {for [Tool::Freehand, Tool::Line, Tool::Rect, Tool::Circle, Tool::Text, Tool::Pan].iter().map(|t| {
                        let tool_clone = tool.clone();
                        let current_tool = t.clone();
                        let current_tool_for_onclick = current_tool.clone();
                        let is_active = *tool == current_tool;
                        html! {
                            <button 
                                class={classes!("tool-btn", is_active.then(|| "active"))}
                                onclick={move |_| tool_clone.set(current_tool_for_onclick.clone())}
                                title={format!("{:?}", current_tool)}
                            >
                                {current_tool.icon()}
                                <span>{format!("{:?}", current_tool)}</span>
                            </button>
                        }
                    })}
                </div>
            </div>

            <div class="toolbar-section">
                <label class="toolbar-label">{"Colors"}</label>
                <div class="color-palette">
                    {for colors.iter().map(|c| {
                        let color_clone = color.clone();
                        let current_color = c.to_string();
                        let is_active = *color == current_color;
                        html! {
                            <button 
                                class={classes!("color-btn", is_active.then(|| "active"))}
                                style={format!("background-color: {}", current_color)}
                                onclick={move |_| color_clone.set(current_color.clone())}
                            />
                        }
                    })}
                </div>
            </div>

            <div class="toolbar-section">
                <label class="toolbar-label">{"Brush Size"}</label>
                <div class="brush-controls">
                    <input 
                        type="range" 
                        min="1" 
                        max="20" 
                        step="1"
                        value={width.to_string()}
                        class="brush-slider"
                        oninput={{
                            let width = width.clone();
                            Callback::from(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                width.set(input.value().parse::<f64>().unwrap_or(3.0));
                            })
                        }}
                    />
                    <span class="brush-value">{format!("{}px", *width as i32)}</span>
                </div>
            </div>

            <div class="toolbar-section">
                <label class="toolbar-label">{"Zoom"}</label>
                <div class="zoom-controls">
                    <button 
                        class="zoom-btn"
                        onclick={{
                            let zoom = zoom.clone();
                            Callback::from(move |_| {
                                zoom.set((*zoom * 1.2).min(5.0));
                            })
                        }}
                    >{"+"}</button>
                    <span class="zoom-value">{format!("{}%", (*zoom * 100.0) as i32)}</span>
                    <button 
                        class="zoom-btn"
                        onclick={{
                            let zoom = zoom.clone();
                            Callback::from(move |_| {
                                zoom.set((*zoom / 1.2).max(0.2));
                            })
                        }}
                    >{"-"}</button>
                </div>
            </div>
        </div>
    };

    // Mouse event handlers
    let onmousedown = {
        let drawing = drawing.clone();
        let last_pos = last_pos.clone();
        let tool = tool.clone();
        let start_pos = start_pos.clone();
        let text_input = text_input.clone();
        let pan_start = pan_start.clone();
        let pan = pan.clone();
        let zoom = zoom.clone();
        Callback::from(move |e: MouseEvent| {
            let canvas_x = (e.offset_x() as f64 - pan.0) / *zoom;
            let canvas_y = (e.offset_y() as f64 - pan.1) / *zoom;
            
            match *tool {
                Tool::Freehand => {
                    drawing.set(true);
                    last_pos.borrow_mut().0 = canvas_x;
                    last_pos.borrow_mut().1 = canvas_y;
                }
                Tool::Line | Tool::Rect | Tool::Circle => {
                    start_pos.borrow_mut().replace((canvas_x, canvas_y));
                    drawing.set(true);
                }
                Tool::Text => {
                    text_input.set(Some((canvas_x, canvas_y)));
                }
                Tool::Pan => {
                    pan_start.borrow_mut().replace((e.offset_x() as f64, e.offset_y() as f64));
                    // Change cursor to grabbing
                    if let Some(canvas) = e.target().and_then(|t| t.dyn_into::<HtmlCanvasElement>().ok()) {
                        if let Ok(html_element) = canvas.dyn_into::<HtmlElement>() {
                            let _ = html_element.style().set_property("cursor", "grabbing");
                        }
                    }
                    
                }
                _ => {}
            }
        })
    };

    let onmouseup = {
        let drawing = drawing.clone();
        let tool = tool.clone();
        let start_pos = start_pos.clone();
        let ws = ws.clone();
        let color = color.clone();
        let width = width.clone();
        let pan_start = pan_start.clone();
        let pan = pan.clone();
        let zoom = zoom.clone();
        Callback::from(move |e: MouseEvent| {
            match *tool {
                Tool::Freehand => drawing.set(false),
                Tool::Line | Tool::Rect | Tool::Circle => {
                    if *drawing {
                        if let Some((sx, sy)) = *start_pos.borrow() {
                            let canvas_x = (e.offset_x() as f64 - pan.0) / *zoom;
                            let canvas_y = (e.offset_y() as f64 - pan.1) / *zoom;
                            
                            let event = match *tool {
                                Tool::Line => WhiteboardEvent::DrawLine {
                                    from: (sx, sy),
                                    to: (canvas_x, canvas_y),
                                    color: color.to_string(),
                                    width: *width,
                                },
                                Tool::Rect => WhiteboardEvent::DrawRect {
                                    from: (sx, sy),
                                    to: (canvas_x, canvas_y),
                                    color: color.to_string(),
                                    width: *width,
                                },
                                Tool::Circle => {
                                    let dx = canvas_x - sx;
                                    let dy = canvas_y - sy;
                                    let radius = (dx * dx + dy * dy).sqrt();
                                    WhiteboardEvent::DrawCircle {
                                        center: (sx, sy),
                                        radius,
                                        color: color.to_string(),
                                        width: *width,
                                    }
                                }
                                _ => unreachable!(),
                            };
                            if let Some(writer_rc) = &*ws.borrow() {
                                let writer = writer_rc.clone();
                                let msg = Message::Text(serde_json::to_string(&event).unwrap());
                                spawn_local(async move {
                                    let mut writer = writer.borrow_mut();
                                    let _ = writer.send(msg).await;
                                });
                            }
                        }
                        drawing.set(false);
                        start_pos.borrow_mut().take();
                    }
                }
                Tool::Pan => {
                    pan_start.borrow_mut().take();
                    // Reset cursor to grab
                    if let Some(canvas) = e.target().and_then(|t| t.dyn_into::<HtmlCanvasElement>().ok()) {
                        if let Ok(html_element) = canvas.dyn_into::<HtmlElement>() {
                            let _ = html_element.style().set_property("cursor", "grab");
                        }
                    }
                }
                _ => {},
            }
        })
    };

    let onmousemove = {
        let drawing = drawing.clone();
        let ws = ws.clone();
        let last_pos = last_pos.clone();
        let canvas_ref = canvas_ref.clone();
        let tool = tool.clone();
        let color = color.clone();
        let pan = pan.clone();
        let pan_start = pan_start.clone();
        let zoom = zoom.clone();
        Callback::from(move |e: MouseEvent| {
            match *tool {
                Tool::Freehand => {
                    if *drawing {
                        let canvas_x = (e.offset_x() as f64 - pan.0) / *zoom;
                        let canvas_y = (e.offset_y() as f64 - pan.1) / *zoom;
                        
                        // Draw locally with smooth line
                        if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                            let ctx = canvas
                                .get_context("2d")
                                .unwrap()
                                .unwrap()
                                .dyn_into::<CanvasRenderingContext2d>()
                                .unwrap();
                            ctx.save();
                            ctx.translate(pan.0, pan.1).unwrap();
                            ctx.scale(*zoom, *zoom).unwrap();
                            ctx.set_stroke_style_str(&color);
                            ctx.set_line_width(3.0);
                            ctx.set_line_cap("round");
                            ctx.begin_path();
                            ctx.move_to(last_pos.borrow().0, last_pos.borrow().1);
                            ctx.line_to(canvas_x, canvas_y);
                            ctx.stroke();
                            ctx.restore();
                        }
                        
                        // Send to backend
                        let event = WhiteboardEvent::DrawFreehand { x: canvas_x, y: canvas_y, dragging: true };
                        if let Some(writer_rc) = &*ws.borrow() {
                            let writer = writer_rc.clone();
                            let msg = Message::Text(serde_json::to_string(&event).unwrap());
                            spawn_local(async move {
                                let mut writer = writer.borrow_mut();
                                let _ = writer.send(msg).await;
                            });
                        }
                        last_pos.borrow_mut().0 = canvas_x;
                        last_pos.borrow_mut().1 = canvas_y;
                    }
                }
                Tool::Pan => {
                    if let Some((start_x, start_y)) = *pan_start.borrow() {
                        let dx = e.offset_x() as f64 - start_x;
                        let dy = e.offset_y() as f64 - start_y;
                        pan.set((pan.0 + dx, pan.1 + dy));
                        pan_start.borrow_mut().replace((e.offset_x() as f64, e.offset_y() as f64));
                    }
                }
                _ => {},
            }
        })
    };

    let onwheel = {
        let zoom = zoom.clone();
        Callback::from(move |e: yew::events::WheelEvent| {
            e.prevent_default();
            let delta = e.delta_y();
            let factor = if delta < 0.0 { 1.1 } else { 0.9 };
            zoom.set(((*zoom) * factor).clamp(0.2, 5.0));
        })
    };

    // Text input overlay
    let text_input_overlay = {
        let text_input = text_input.clone();
        let ws = ws.clone();
        let color = color.clone();
        let pan = pan.clone();
        let zoom = zoom.clone();
        
        if let Some((x, y)) = *text_input {
            let screen_x = x * *zoom + pan.0;
            let screen_y = y * *zoom + pan.1;
            
            let onkeydown = {
                let text_input = text_input.clone();
                let ws = ws.clone();
                let color = color.clone();
                Callback::from(move |e: web_sys::KeyboardEvent| {
                    if e.key() == "Enter" {
                        if let Some((canvas_x, canvas_y)) = *text_input {
                            if let Some(input) = web_sys::window().unwrap().document().unwrap().get_element_by_id("text_input") {
                                let input_element = input.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
                                let value = input_element.value();
                                if !value.is_empty() {
                                    let event = WhiteboardEvent::AddText {
                                        pos: (canvas_x, canvas_y),
                                        text: value.clone(),
                                        color: color.to_string(),
                                        size: 18.0,
                                    };
                                    if let Some(writer_rc) = &*ws.borrow() {
                                        let writer = writer_rc.clone();
                                        let msg = Message::Text(serde_json::to_string(&event).unwrap());
                                        spawn_local(async move {
                                            let mut writer = writer.borrow_mut();
                                            let _ = writer.send(msg).await;
                                        });
                                    }
                                }
                            }
                            text_input.set(None);
                        }
                    } else if e.key() == "Escape" {
                        text_input.set(None);
                    }
                })
            };
            
            let onblur = {
                let text_input = text_input.clone();
                Callback::from(move |_: web_sys::FocusEvent| {
                    text_input.set(None);
                })
            };
            
            html! {
                <input 
                    id="text_input" 
                    type="text" 
                    class="text-input-overlay"
                    style={format!("left: {}px; top: {}px;", screen_x, screen_y - 5.0)}
                    onkeydown={onkeydown}
                    onblur={onblur}
                    placeholder="Type here..."
                />
            }
        } else {
            html! {}
        }
    };

    // Apply cursor style to canvas
    let canvas_cursor = tool.cursor();

    html! {
        <div class="app">
            <style>
                {include_str!("../styles.css")}
            </style>
            {toolbar}
            <div class="canvas-container">
                {text_input_overlay}
                <canvas
                    ref={canvas_ref}
                    width={1200}
                    height={800}
                    class="whiteboard-canvas"
                    style={format!("cursor: {}", canvas_cursor)}
                    onmousedown={onmousedown}
                    onmouseup={onmouseup}
                    onmousemove={onmousemove}
                    onwheel={onwheel}
                />
            </div>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}