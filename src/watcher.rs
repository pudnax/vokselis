use color_eyre::eyre::Result;
use notify::{event::ModifyKind, Config, EventKind, Watcher as WatcherTrait};
use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use winit::event_loop::EventLoop;

use crate::shader_compiler::ShaderCompiler;

pub trait ReloadablePipeline {
    fn reload(&mut self, device: &wgpu::Device, module: wgpu::ShaderModule);
}

pub struct Watcher {
    watcher: notify::PollWatcher,
    pub hash_dump: HashMap<PathBuf, Rc<RefCell<dyn ReloadablePipeline>>>,
}

impl Watcher {
    pub fn new(
        device: Arc<wgpu::Device>,
        event_loop: &EventLoop<(PathBuf, wgpu::ShaderModule)>,
    ) -> Result<Self> {
        let mut watcher = notify::PollWatcher::with_delay(
            watch_callback(device, event_loop),
            Duration::from_millis(3),
        )?;
        watcher.configure(Config::PreciseEvents(true))?;

        Ok(Self {
            watcher,
            hash_dump: HashMap::new(),
        })
    }

    pub fn register(
        &mut self,
        path: &Path,
        pipeline: Rc<RefCell<dyn ReloadablePipeline>>,
    ) -> Result<()> {
        self.watcher
            .watch(path, notify::RecursiveMode::NonRecursive)?;
        self.hash_dump.insert(path.to_path_buf(), pipeline.clone());
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
                kind: EventKind::Modify(ModifyKind::Metadata(..) | ModifyKind::Data(..)),
                paths,
                ..
            } = res
            {
                for path in paths {
                    let path = path.canonicalize().expect("Failed canonicalize path");
                    if let Ok(x) = shader_compiler.create_shader_module(&path) {
                        let device_ref = device.upgrade().unwrap();
                        let module = unsafe {
                            device_ref.create_shader_module_unchecked(
                                &wgpu::ShaderModuleDescriptor {
                                    label: None,
                                    source: wgpu::ShaderSource::SpirV(x.into()),
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
