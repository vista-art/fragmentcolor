pub mod controllers;
mod events;
mod platform;
mod renderer;
mod shapes;

use cfg_if::cfg_if;
use controllers::ControllerOptions;
use events::EventManager;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, sync::RwLock};
#[cfg(wasm)]
use {gloo_utils::format::JsValueSerdeExt, wasm_bindgen::prelude::*};

use crate::{
    events::VipArguments,
    renderer::{window, Renderer},
};

#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
#[derive(Serialize, Deserialize, Default)]
pub struct Options {
    pub window: Option<window::WindowOptions>,
    pub controllers: Option<ControllerOptions>,
}

#[derive(Clone)]
#[cfg_attr(wasm, wasm_bindgen(getter_with_clone))]
pub struct PLRender {
    event_manager: Arc<RwLock<EventManager>>,
    renderer: Option<Arc<RwLock<Renderer>>>,
    // controllers: Controllers,
}

#[cfg_attr(wasm, wasm_bindgen)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[cfg(not(wasm))]
unsafe impl Send for PLRender {}

#[cfg_attr(wasm, wasm_bindgen)]
impl PLRender {
    #[cfg_attr(wasm, wasm_bindgen(constructor))]
    pub fn new() -> PLRender {
        Self::init_logger();

        Self {
            event_manager: Arc::new(RwLock::new(EventManager::new())),
            renderer: None,
        }
    }

    #[cfg(wasm)]
    pub fn config(&mut self, options: JsValue) {
        let options = JsValue::into_serde(&options).expect("Couldn't deserialize options");
        let renderer = self.config_renderer(options);

        let future = async move {
            renderer.write().unwrap().initialize().await;
        };

        wasm_bindgen_futures::spawn_local(future);
    }

    #[cfg(not(wasm))]
    pub async fn config(&mut self, options: Options) {
        let renderer = self.config_renderer(options);
        renderer.write().unwrap().initialize().await;
    }

    pub fn run(&mut self) {
        let mut event_manager = self.event_manager_write_lock();
        let runner = event_manager.get_event_loop_runnner();
        drop(event_manager); // release to avoid deadlock

        //@TODO make platform-specific code similar to what Winit does.
        //      they have a platform module with #path[] definitions
        //      that will route the call to the correct platform at
        //      compile time. All platform-specific code have the
        //      same module name. In this case, it would be like
        //      platform::run(event_handler).

        #[cfg(wasm)]
        wasm_bindgen_futures::spawn_local(runner.run_event_loop());

        #[cfg(not(wasm))]
        pollster::block_on(runner.run_event_loop()); // this function never returns
    }

    fn config_renderer(&mut self, options: Options) -> Arc<RwLock<Renderer>> {
        let mut event_manager = self.event_manager_write_lock();
        let window = window::init_window(
            event_manager.event_loop(),
            &options.window.unwrap_or_default(),
        ); // @TODO support multiple windows
        let renderer = Arc::new(RwLock::new(Renderer::new(window)));
        event_manager.attach_renderer(renderer.clone());
        drop(event_manager);

        self.renderer = Some(renderer.clone());
        for controller_option in options.controllers.iter() {
            if controller_option.gaze.is_some() {
                let gaze_options = controller_option.gaze.as_ref().unwrap().clone();
                let gaze = Box::new(controllers::gaze::Gaze::new(gaze_options));
                let mut renderer = self.renderer_write_lock();
                renderer.add_controller(gaze);
            }
        } // @TODO make this dynamic: remove "Gaze" from the code
          //       - move the enrichment definitions to another crate
          //       - build enrichments from Javascript

        renderer
    }

    #[cfg(wasm)]
    pub fn trigger(&self, controller: &str, event: &str, params: Box<[JsValue]>) {
        let params: VipArguments = params
            .iter()
            .map(|param| {
                let js_str = js_sys::JSON::stringify(param).expect("Couldn't stringify param");
                js_str.as_string().expect("Couldn't get param as string")
            })
            .collect();

        self.event_manager()
            .trigger(controller, event, params)
            .expect("Event loop closed");
    }

    #[cfg(not(wasm))]
    pub fn trigger(&self, controller: &str, event: &str, params: VipArguments) {
        self.event_manager()
            .trigger(controller, event, params)
            .expect("Event loop closed");
    }

    pub fn resolution(&self) -> Resolution {
        let renderer = self.renderer();
        let (width, height) = renderer.window_size.into();
        Resolution { width, height }
    }

    fn init_logger() {
        cfg_if! { if #[cfg(wasm)] {
            crate::platform::web::utils::set_panic_hook();
            console_log::init_with_level(log::Level::Info).unwrap_or(());
        } else {
            env_logger::try_init().unwrap_or(());
        }}
    }

    fn event_manager_write_lock(&self) -> std::sync::RwLockWriteGuard<'_, EventManager> {
        self.event_manager
            .write()
            .expect("Couldn't get event manager write lock")
    }

    fn event_manager(&self) -> std::sync::RwLockReadGuard<'_, EventManager> {
        self.event_manager
            .read()
            .expect("Couldn't get event manager read lock")
    }

    fn renderer_write_lock(&self) -> std::sync::RwLockWriteGuard<'_, Renderer> {
        self.renderer
            .as_ref()
            .expect("Renderer not initialized")
            .write()
            .expect("Couldn't get renderer write lock")
    }

    fn renderer(&self) -> std::sync::RwLockReadGuard<'_, Renderer> {
        self.renderer
            .as_ref()
            .expect("Renderer not initialized")
            .read()
            .expect("Couldn't get renderer read lock")
    }
}
