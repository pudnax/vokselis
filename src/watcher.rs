use color_eyre::eyre::Result;
use notify::{
    event::{AccessKind, AccessMode},
    Config, EventKind, Watcher as WatcherTrait,
};
use winit::event_loop::EventLoop;

use std::{
    cell::RefCell,
    ffi::OsStr,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use crate::utils::{shader_compiler::ShaderCompiler, ContiniousHashMap};
use crate::SHADER_FOLDER;

pub trait ReloadablePipeline {
    fn reload(&mut self, device: &wgpu::Device, module: &wgpu::ShaderModule);
}

impl ReloadablePipeline for Rc<RefCell<dyn ReloadablePipeline>> {
    fn reload(&mut self, device: &wgpu::Device, module: &wgpu::ShaderModule) {
        self.borrow_mut().reload(device, module);
    }
}

pub struct Watcher {
    _watcher: notify::RecommendedWatcher,
    pub hash_dump: ContiniousHashMap<PathBuf, Rc<RefCell<dyn ReloadablePipeline>>>,
}

impl Watcher {
    pub fn new(
        device: Arc<wgpu::Device>,
        event_loop: &EventLoop<(PathBuf, wgpu::ShaderModule)>,
    ) -> Result<Self> {
        let mut watcher = notify::recommended_watcher(watch_callback(device, event_loop))?;
        watcher.configure(Config::PreciseEvents(true))?;
        watcher.watch(
            Path::new(SHADER_FOLDER),
            notify::RecursiveMode::NonRecursive,
        )?;

        Ok(Self {
            _watcher: watcher,
            hash_dump: ContiniousHashMap::new(),
        })
    }

    pub fn register(
        &mut self,
        path: &impl AsRef<Path>,
        pipeline: Rc<RefCell<dyn ReloadablePipeline>>,
    ) -> Result<()> {
        self.hash_dump
            .push_value(path.as_ref().canonicalize()?, pipeline.clone());
        Ok(())
    }
}

fn watch_callback(
    device: Arc<wgpu::Device>,
    event_loop: &EventLoop<(PathBuf, wgpu::ShaderModule)>,
) -> impl FnMut(notify::Result<notify::Event>) {
    let proxy = event_loop.create_proxy();
    let device = Arc::downgrade(&device);
    let mut shader_compiler = ShaderCompiler::new();
    move |event| match event {
        Ok(res) => {
            if let notify::event::Event {
                kind: EventKind::Access(AccessKind::Close(AccessMode::Write)),
                paths,
                ..
            } = res
            {
                for path in paths
                    .into_iter()
                    .filter(|p| p.extension() == Some(OsStr::new("wgsl")))
                {
                    if let Ok(x) = shader_compiler.create_shader_module(&path) {
                        let device_ref = device.upgrade().unwrap();
                        let module = unsafe {
                            device_ref.create_shader_module_spirv(
                                &wgpu::ShaderModuleDescriptorSpirV {
                                    label: path.to_str(),
                                    source: x.into(),
                                },
                            )
                        };
                        proxy
                            .send_event((path, module))
                            .expect("Event Loop have been dropped");
                        crate::utils::green_blink();
                    };
                }
            }
        }
        Err(err) => {
            eprintln!("File watcher error: {err}");
        }
    }
}
