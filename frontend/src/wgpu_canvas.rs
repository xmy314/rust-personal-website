use crate::{wgpu_context::WgpuState, EVENT_LOOP};

use web_sys;
use yew::{platform::spawn_local, prelude::*};

#[cfg(target_family = "wasm")] // cfg here to trick rust analyzer
use winit::platform::web::WindowAttributesExtWebSys;

use winit::window::WindowAttributes;

#[derive(PartialEq, Properties)]
pub struct WgpuCanvasProps {}

pub enum WgpuCanvasMsg<'a> {
    Initializing,
    Initialized(WgpuState<'a>),
    Redraw,
}
pub struct WgpuCanvas<'a> {
    canvas: NodeRef,
    context: Option<WgpuState<'a>>,
    callback: Callback<WgpuState<'a>>,
    initialize_sent: bool,
}

impl Component for WgpuCanvas<'static> {
    type Message = WgpuCanvasMsg<'static>;

    type Properties = WgpuCanvasProps;

    fn create(ctx: &Context<Self>) -> Self {
        let canvas = NodeRef::default();
        let context_cb: Callback<WgpuState> = ctx.link().callback(WgpuCanvasMsg::Initialized);

        WgpuCanvas {
            canvas: canvas,
            context: None,
            callback: context_cb,
            initialize_sent: false,
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
                ctx.link().send_message(WgpuCanvasMsg::Redraw);
                true
            }
            WgpuCanvasMsg::Redraw => {
                if let Some(context) = &mut self.context {
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.context.is_none() && !self.initialize_sent {
            ctx.link().send_message(WgpuCanvasMsg::Initializing);
        }

        html! (
            <div>
              <canvas ref = {self.canvas.clone()}/>
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
        let window_attr = WindowAttributes::default()
            .with_inner_size(winit::dpi::LogicalSize::new(width, height));
        #[cfg(target_family = "wasm")]
        let window_attr = WindowAttributes::default().with_canvas(Some(canvas));
        #[allow(deprecated)]
        let window = EVENT_LOOP.with(|event_loop| event_loop.create_window(window_attr).unwrap());

        let cb = ctx.callback.clone();
        spawn_local(async move {
            let wgpu_state = WgpuState::new(window, height, width).await;
            cb.emit(wgpu_state);
        });
    }
}
