use crate::{wgpu_context::WgpuContext, EVENT_LOOP};

use gloo::timers::callback::Interval;
use yew::{platform::spawn_local, prelude::*};

#[cfg(target_family = "wasm")] // cfg here to trick rust analyzer
use winit::platform::web::WindowAttributesExtWebSys;

use winit::window::WindowAttributes;

#[derive(Debug, Default)]
pub struct ControlState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}
impl ControlState {
    pub fn get_2d_vec(&self) -> [i8; 2] {
        let up = if self.up {
            1
        } else if self.down {
            -1
        } else {
            0
        };
        let right = if self.right {
            1
        } else if self.left {
            -1
        } else {
            0
        };
        [up, right]
    }
}

pub enum ControlMsg {
    SUp,
    SDown,
    SLeft,
    SRight,
    EUp,
    EDown,
    ELeft,
    ERight,
}

#[derive(PartialEq, Properties)]
pub struct WgpuCanvasProps {}

pub enum WgpuCanvasMsg<'a> {
    Initializing,
    Initialized(WgpuContext<'a>),
    Control(ControlMsg),
    Update,
}
pub struct WgpuCanvas<'a> {
    canvas: NodeRef,
    context: Option<WgpuContext<'a>>,
    callback: Callback<WgpuContext<'a>>,
    initialize_sent: bool,
    control_state: ControlState,
    update_timeout: Option<Interval>,
}

impl Component for WgpuCanvas<'static> {
    type Message = WgpuCanvasMsg<'static>;

    type Properties = WgpuCanvasProps;

    fn create(ctx: &Context<Self>) -> Self {
        let canvas = NodeRef::default();
        let context_cb: Callback<WgpuContext> = ctx.link().callback(WgpuCanvasMsg::Initialized);

        ctx.link().callback(|_| WgpuCanvasMsg::Update).emit(());

        WgpuCanvas {
            canvas,
            context: None,
            callback: context_cb,
            initialize_sent: false,
            control_state: ControlState::default(),
            update_timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            WgpuCanvasMsg::Initializing => {
                log::info!("Initializing");
                self.initialize_sent = true;
                let canvas = self.canvas.cast::<web_sys::HtmlCanvasElement>().unwrap();
                canvas.set_height(500);
                canvas.set_width(500);
                WgpuCanvas::create_wgpu_context(
                    self.canvas.cast::<web_sys::HtmlCanvasElement>().unwrap(),
                    self,
                );
                true
            }
            WgpuCanvasMsg::Initialized(wgpu_state) => {
                log::info!("Initialized");
                self.context = Some(wgpu_state);

                let handle = {
                    let link = ctx.link().clone();
                    Interval::new(30, move || link.send_message(WgpuCanvasMsg::Update))
                };

                self.update_timeout = Some(handle);

                true
            }
            WgpuCanvasMsg::Update => {
                if let Some(context) = &mut self.context {
                    context.update(self.control_state.get_2d_vec());
                    context.window().request_redraw();
                    match context.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            context.resize(context.size());
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                            log::error!("OutOfMemory");
                            panic!("OutOfMemory");
                        }
                        // This happens when the a frame takes too long to present
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout");
                        }
                    }

                    true
                } else {
                    false
                }
            }
            WgpuCanvasMsg::Control(control_input) => {
                match control_input {
                    ControlMsg::SUp => self.control_state.up = true,
                    ControlMsg::SDown => self.control_state.down = true,
                    ControlMsg::SLeft => self.control_state.left = true,
                    ControlMsg::SRight => self.control_state.right = true,
                    ControlMsg::EUp => self.control_state.up = false,
                    ControlMsg::EDown => self.control_state.down = false,
                    ControlMsg::ELeft => self.control_state.left = false,
                    ControlMsg::ERight => self.control_state.right = false,
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.context.is_none() && !self.initialize_sent {
            ctx.link().send_message(WgpuCanvasMsg::Initializing);
        }

        let on_key_up = ctx
            .link()
            .batch_callback(|e: KeyboardEvent| match e.key().as_str() {
                "ArrowUp" | "w" | "W" => Some(WgpuCanvasMsg::Control(ControlMsg::EUp)),
                "ArrowDown" | "s" | "S" => Some(WgpuCanvasMsg::Control(ControlMsg::EDown)),
                "ArrowLeft" | "a" | "A" => Some(WgpuCanvasMsg::Control(ControlMsg::ELeft)),
                "ArrowRight" | "d" | "D" => Some(WgpuCanvasMsg::Control(ControlMsg::ERight)),
                _ => None,
            });
        let on_key_down = ctx
            .link()
            .batch_callback(|e: KeyboardEvent| match e.key().as_str() {
                "ArrowUp" | "w" | "W" => Some(WgpuCanvasMsg::Control(ControlMsg::SUp)),
                "ArrowDown" | "s" | "S" => Some(WgpuCanvasMsg::Control(ControlMsg::SDown)),
                "ArrowLeft" | "a" | "A" => Some(WgpuCanvasMsg::Control(ControlMsg::SLeft)),
                "ArrowRight" | "d" | "D" => Some(WgpuCanvasMsg::Control(ControlMsg::SRight)),
                _ => None,
            });

        html! (
            <div>
              <canvas onkeydown={on_key_down} onkeyup={on_key_up} ref = {self.canvas.clone()}/>
            </div>
        )
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {}
}

impl WgpuCanvas<'static> {
    pub fn create_wgpu_context(canvas: web_sys::HtmlCanvasElement, ctx: &mut WgpuCanvas<'static>) {
        log::info!("context creation started");
        let height = canvas.height();
        let width = canvas.width();
        #[cfg(not(target_family = "wasm"))]
        let window_attr = WindowAttributes::default()
            .with_inner_size(winit::dpi::LogicalSize::new(width, height));
        #[cfg(target_family = "wasm")]
        let window_attr = WindowAttributes::default().with_canvas(Some(canvas));
        #[allow(deprecated)]
        let window = EVENT_LOOP.with(|event_loop| event_loop.create_window(window_attr).unwrap());

        let cb = ctx.callback.clone();
        spawn_local(async move {
            let wgpu_state = WgpuContext::new(window, height, width).await;
            cb.emit(wgpu_state);
        });
    }
}
