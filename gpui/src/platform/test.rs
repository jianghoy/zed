use crate::ClipboardItem;
use pathfinder_geometry::vector::Vector2F;
use std::{
    any::Any,
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

pub(crate) struct Platform {
    dispatcher: Arc<dyn super::Dispatcher>,
    fonts: Arc<dyn super::FontSystem>,
    current_clipboard_item: RefCell<Option<ClipboardItem>>,
    last_prompt_for_new_path_args: RefCell<Option<(PathBuf, Box<dyn FnOnce(Option<PathBuf>)>)>>,
}

struct Dispatcher;

pub struct Window {
    size: Vector2F,
    scale_factor: f32,
    current_scene: Option<crate::Scene>,
    event_handlers: Vec<Box<dyn FnMut(super::Event)>>,
    resize_handlers: Vec<Box<dyn FnMut(&mut dyn super::WindowContext)>>,
}

impl Platform {
    fn new() -> Self {
        Self {
            dispatcher: Arc::new(Dispatcher),
            fonts: Arc::new(super::current::FontSystem::new()),
            current_clipboard_item: Default::default(),
            last_prompt_for_new_path_args: Default::default(),
        }
    }

    pub(crate) fn simulate_new_path_selection(
        &self,
        result: impl FnOnce(PathBuf) -> Option<PathBuf>,
    ) {
        let (dir_path, callback) = self
            .last_prompt_for_new_path_args
            .take()
            .expect("prompt_for_new_path was not called");
        callback(result(dir_path));
    }

    pub(crate) fn did_prompt_for_new_path(&self) -> bool {
        self.last_prompt_for_new_path_args.borrow().is_some()
    }
}

impl super::Platform for Platform {
    fn on_menu_command(&self, _: Box<dyn FnMut(&str, Option<&dyn Any>)>) {}

    fn on_become_active(&self, _: Box<dyn FnMut()>) {}

    fn on_resign_active(&self, _: Box<dyn FnMut()>) {}

    fn on_event(&self, _: Box<dyn FnMut(crate::Event) -> bool>) {}

    fn on_open_files(&self, _: Box<dyn FnMut(Vec<std::path::PathBuf>)>) {}

    fn run(&self, _on_finish_launching: Box<dyn FnOnce() -> ()>) {
        unimplemented!()
    }

    fn dispatcher(&self) -> Arc<dyn super::Dispatcher> {
        self.dispatcher.clone()
    }

    fn fonts(&self) -> std::sync::Arc<dyn super::FontSystem> {
        self.fonts.clone()
    }

    fn activate(&self, _ignoring_other_apps: bool) {}

    fn open_window(
        &self,
        _: usize,
        options: super::WindowOptions,
        _executor: Rc<super::executor::Foreground>,
    ) -> Box<dyn super::Window> {
        Box::new(Window::new(options.bounds.size()))
    }

    fn key_window_id(&self) -> Option<usize> {
        None
    }

    fn set_menus(&self, _menus: Vec<crate::Menu>) {}

    fn quit(&self) {}

    fn prompt_for_paths(
        &self,
        _: super::PathPromptOptions,
        _: Box<dyn FnOnce(Option<Vec<std::path::PathBuf>>)>,
    ) {
    }

    fn prompt_for_new_path(&self, path: &Path, f: Box<dyn FnOnce(Option<std::path::PathBuf>)>) {
        *self.last_prompt_for_new_path_args.borrow_mut() = Some((path.to_path_buf(), f));
    }

    fn write_to_clipboard(&self, item: ClipboardItem) {
        *self.current_clipboard_item.borrow_mut() = Some(item);
    }

    fn read_from_clipboard(&self) -> Option<ClipboardItem> {
        self.current_clipboard_item.borrow().clone()
    }
}

impl Window {
    fn new(size: Vector2F) -> Self {
        Self {
            size,
            event_handlers: Vec::new(),
            resize_handlers: Vec::new(),
            scale_factor: 1.0,
            current_scene: None,
        }
    }
}

impl super::Dispatcher for Dispatcher {
    fn is_main_thread(&self) -> bool {
        true
    }

    fn run_on_main_thread(&self, task: async_task::Runnable) {
        task.run();
    }
}

impl super::WindowContext for Window {
    fn size(&self) -> Vector2F {
        self.size
    }

    fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    fn present_scene(&mut self, scene: crate::Scene) {
        self.current_scene = Some(scene);
    }
}

impl super::Window for Window {
    fn on_event(&mut self, callback: Box<dyn FnMut(crate::Event)>) {
        self.event_handlers.push(callback);
    }

    fn on_resize(&mut self, callback: Box<dyn FnMut(&mut dyn super::WindowContext)>) {
        self.resize_handlers.push(callback);
    }
}

pub(crate) fn platform() -> Platform {
    Platform::new()
}
