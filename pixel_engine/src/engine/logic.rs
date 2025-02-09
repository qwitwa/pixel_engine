use super::decals::Decal;
use super::inputs::{self, Input, KeySet, Mouse, MouseBtn, MouseWheel};
use super::Sprite;

use pixel_engine_draw::traits::SmartDrawingTrait;
use px_draw::graphics::DrawingSprite;

use pixel_engine_draw::vector2::Vu2d;
use px_backend::winit::{
    self,
    event::{Event, WindowEvent},
};

/// A Wrapper around an Engine
#[derive(Debug)]
pub struct EngineWrapper(Option<Engine>);

impl std::ops::Deref for EngineWrapper {
    type Target = Engine;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("Pannic while deref EngineWrapper")
    }
}

impl std::ops::DerefMut for EngineWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
            .as_mut()
            .expect("Panic while deref Mut EngineWrapper")
    }
}

impl EngineWrapper {
    /// Create the Engine and the Wrapper
    pub async fn new(title: String, size: (u32, u32, u32)) -> Self {
        Self(Some(Engine::new(title, size).await))
    }
    /// Create the Engine and the Wrapper
    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub fn new_sync(title: String, size: (u32, u32, u32)) -> Self {
        Self(Some(Engine::new_sync(title, size)))
    }
    /// The core of your program,
    ///
    /// Takes a function F that will be run every frame, It will do the event handling  and similar
    /// things between frames.
    #[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
    pub fn run<F>(mut self, mut main_func: F) -> !
    where
        F: (FnMut(&mut Engine) -> Result<bool, Box<dyn std::error::Error>>) + 'static,
    {
        let mut engine = self.0.take().unwrap();
        let mut force_exit = false;
        let event_loop = engine.event_loop.take().unwrap();
        let mut redraw = true;
        let mut redraw_last_frame = false;
        event_loop.run(move |e, _, control_flow| {
            if redraw_last_frame {
                for key in &engine.k_pressed {
                    engine.k_held.insert(*key);
                }
                engine.k_pressed.clear();
                engine.k_released.clear();
                for i in 0..3 {
                    if engine.mouse.buttons[i].released {
                        engine.mouse.buttons[i].released = false;
                    }
                    if engine.mouse.buttons[i].pressed {
                        engine.mouse.buttons[i].pressed = false;
                        engine.mouse.buttons[i].held = true;
                    }
                }
                engine.mouse.wheel = MouseWheel::None;
                redraw_last_frame = false;
            }
            match e {
                Event::WindowEvent {
                    event: e,
                    window_id,
                } if window_id == engine.window.id() => match e {
                    WindowEvent::KeyboardInput { input: inp, .. } => {
                        if let Some(k) = inp.virtual_keycode {
                            if inp.state == winit::event::ElementState::Released {
                                engine.k_pressed.remove(&inputs::Key::from(inp));
                                engine.k_held.remove(&inputs::Key::from(inp));
                                engine.k_released.insert(inputs::Key::from(inp));
                            } else if !engine.k_held.has(k) {
                                engine.k_pressed.insert(inputs::Key::from(inp));
                            }
                        }
                    }
                    WindowEvent::CloseRequested => {
                        force_exit = true;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let (x, y): (f64, f64) = position.into();
                        //events.push(Events::MouseMove(x, y));
                        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                        {
                            engine.mouse.pos = (
                                (x / f64::from(engine.size.2)).trunc().abs() as u32,
                                (y / f64::from(engine.size.2)).trunc().abs() as u32,
                            );
                        };
                    }
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            engine.mouse.wheel = if x.abs() > y.abs() {
                                if x > 0.0 {
                                    MouseWheel::Right
                                } else if x < 0.0 {
                                    MouseWheel::Left
                                } else {
                                    MouseWheel::None
                                }
                            } else if y > 0.0 {
                                MouseWheel::Down
                            } else if y < 0.0 {
                                MouseWheel::Up
                            } else {
                                MouseWheel::None
                            };
                        }
                        winit::event::MouseScrollDelta::PixelDelta(lp) => {
                            let (x, y): (f64, f64) = lp.into();
                            engine.mouse.wheel = if x.abs() > y.abs() {
                                if x > 0.0 {
                                    MouseWheel::Right
                                } else if x < 0.0 {
                                    MouseWheel::Left
                                } else {
                                    MouseWheel::None
                                }
                            } else if y > 0.0 {
                                MouseWheel::Down
                            } else if y < 0.0 {
                                MouseWheel::Up
                            } else {
                                MouseWheel::None
                            };
                        }
                    },
                    WindowEvent::MouseInput { button, state, .. } => {
                        if std::mem::discriminant(&button)
                            != std::mem::discriminant(&winit::event::MouseButton::Other(0))
                        {
                            let btn = match button {
                                winit::event::MouseButton::Left => MouseBtn::Left,
                                winit::event::MouseButton::Right => MouseBtn::Right,
                                winit::event::MouseButton::Middle => MouseBtn::Middle,
                                winit::event::MouseButton::Other(_) => {
                                    unreachable!("MouseButton::Other()")
                                }
                            };
                            if state == winit::event::ElementState::Pressed {
                                engine.mouse.buttons[match btn {
                                    MouseBtn::Left => 0,
                                    MouseBtn::Right => 1,
                                    MouseBtn::Middle => 2,
                                }]
                                .pressed = true;
                            } else {
                                engine.mouse.buttons[match btn {
                                    MouseBtn::Left => 0,
                                    MouseBtn::Right => 1,
                                    MouseBtn::Middle => 2,
                                }]
                                .released = true;
                                engine.mouse.buttons[match btn {
                                    MouseBtn::Left => 0,
                                    MouseBtn::Right => 1,
                                    MouseBtn::Middle => 2,
                                }]
                                .held = false;
                            }
                        }
                    }
                    _ => {}
                },
                Event::RedrawRequested(_) => {
                    redraw = true;
                }
                Event::MainEventsCleared => {
                    engine.window.request_redraw();
                }
                _ => {}
            }
            if redraw {
                engine.elapsed = (instant::Instant::now()
                    .checked_duration_since(engine.timer)
                    .expect("Error with timer"))
                .as_secs_f64();
                // End
                engine.timer = instant::Instant::now();
                engine.frame_timer += engine.elapsed;
                engine.frame_count += 1;
                if engine.frame_timer > 1.0 {
                    engine.frame_timer -= 1.0;
                    engine
                        .window
                        .set_title(&format!("{} - {}fps", engine.title, engine.frame_count));
                    engine.frame_count = 0;
                }
                let r = (main_func)(&mut engine);
                if r.is_err() || r.as_ref().ok() == Some(&false) || force_exit {
                    if let Err(e) = r {
                        if cfg!(debug_assertions) {
                            println!("Game Stopped:\n{:?}", e);
                        } else {
                            println!("Game Stopped:\n{}", e);
                        }
                    }
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                let (raw, readlock) = engine.screen.get_ref().get_read_lock();
                engine.handler.render(raw);
                redraw = false;
                redraw_last_frame = true;
                drop(readlock);
            }
        });
    }
}

/**
 *  Bone of the Engine, join everything;
 **/
pub struct Engine {
    /* FRONTEND */
    /// Main title of the window, Window's full title will be "Title - fps"
    pub title: String,
    /// Size of the window, with (x-size,y-size,pixel-size)
    pub size: (u32, u32, u32),

    /* TIME */
    /// Time between current frame and last frame, usefull for movement's calculations
    pub elapsed: f64,
    timer: instant::Instant,
    frame_count: u64,
    frame_timer: f64,

    /* BACKEND */
    pub(crate) screen: DrawingSprite<Sprite>,
    pub(crate) handler: px_backend::Context,
    pub(crate) textsheet_decal: Decal,
    k_pressed: std::collections::HashSet<inputs::Key>,
    k_held: std::collections::HashSet<inputs::Key>,
    k_released: std::collections::HashSet<inputs::Key>,
    mouse: Mouse,
    event_loop: Option<winit::event_loop::EventLoop<()>>,
    window: winit::window::Window,
}
impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("title", &self.title)
            .field("size", &self.size)
            .field("elapsed", &self.elapsed)
            .field("screen", &self.screen)
            .finish()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let decal = self.textsheet_decal.clone_decal();
        self.destroy_decal(&decal);
    }
}

impl Engine {
    /// Create a new [`Engine`]
    async fn new(title: String, size: (u32, u32, u32)) -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_inner_size(
                #[allow(clippy::cast_precision_loss)]
                {
                    winit::dpi::PhysicalSize::new(
                        (size.0 * size.2) as f32,
                        (size.1 * size.2) as f32,
                    )
                },
            )
            .with_title(&title)
            .with_resizable(false)
            .build(&event_loop)
            .expect("Error when constructing window");
        window.set_visible(false);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            let canvas = window.canvas();
            let env = option_env!("PIXEL_ENGINE_CANVAS");
            match env {
                Some(id) => {
                    let window_sys = web_sys::window().unwrap();
                    let document = window_sys.document().unwrap();
                    let parent_canvas = document
                        .query_selector(&format!("#{}", id))
                        .expect("The given ID does not exist")
                        .unwrap();
                    parent_canvas
                        .append_child(&canvas)
                        .expect("Append canvas to HTML body");
                }
                None => {
                    let window_sys = web_sys::window().unwrap();
                    let document = window_sys.document().unwrap();
                    let body = document.body().unwrap();
                    body.append_child(&canvas)
                        .expect("Append canvas to HTML body");
                }
            }
        }

        let mut handler = px_backend::Context::new(&window, size).await;
        let screen = DrawingSprite::new(Sprite::new(size.0, size.1));
        let textsheet_decal =
            crate::decals::Decal::new(&mut handler, SmartDrawingTrait::get_textsheet(&screen));
        Engine {
            /* FRONTEND */
            size,
            title,

            /* TIME */
            timer: instant::Instant::now(),
            frame_count: 0u64,
            frame_timer: 0f64,
            elapsed: 0f64,
            /* BACKEND */
            handler,
            screen,
            textsheet_decal,
            k_pressed: std::collections::HashSet::new(),
            k_held: std::collections::HashSet::new(),
            k_released: std::collections::HashSet::new(),
            mouse: Mouse::new(),
            window: {
                window.set_visible(true);
                window
            },
            event_loop: Some(event_loop),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]

    /// Make a Engine but sync
    fn new_sync(title: String, size: (u32, u32, u32)) -> Self {
        futures::executor::block_on(Self::new(title, size))
    }
    /// Return the current Target size in pixel
    pub fn size(&self) -> Vu2d {
        self.screen.get_size()
    }

    /// Get The status of a key
    #[inline]
    pub fn get_key(&self, keycode: inputs::Keycodes) -> Input {
        Input::new(
            self.k_pressed.has(keycode),
            self.k_held.has(keycode),
            self.k_released.has(keycode),
        )
    }
    /// Get the status of a Mouse Button
    pub fn get_mouse_btn(&self, btn: MouseBtn) -> Input {
        self.mouse.buttons[match btn {
            MouseBtn::Left => 0,
            MouseBtn::Right => 1,
            MouseBtn::Middle => 2,
        }]
    }

    /// Get the mouse location (in pixel) on the screen
    /// Will be defaulted to (0,0) at the start of the program
    pub fn get_mouse_location(&self) -> (u32, u32) {
        self.mouse.pos
    }
    /// Get the scroll wheel direction (If Any) during the frame
    pub fn get_mouse_wheel(&self) -> MouseWheel {
        self.mouse.wheel
    }
    /// Get all Keys pressed during the last frame
    pub fn get_pressed(&self) -> std::collections::HashSet<inputs::Keycodes> {
        self.k_pressed.clone().iter().map(|k| k.key).collect()
    }

    /// Create a GPU version of [`Sprite`]
    pub fn create_decal(&mut self, sprite: &Sprite) -> Decal {
        Decal::new(&mut self.handler, sprite)
    }

    /// Tell the GPU to destroy everything related to that [`Decal`]
    /// it takes the decal by reference since it is not possible to pass by value.
    /// trying to draw the decal afterwards will just not render anything, but may affect
    /// performance if you try to draw lots of "zombie" decals
    pub fn destroy_decal(&mut self, decal: &Decal) {
        decal.0.destroy(&mut self.handler);
        decal.1.set(false);
    }
}
