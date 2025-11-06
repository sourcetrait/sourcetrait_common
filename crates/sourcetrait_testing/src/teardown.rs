use crate::*;

static STATIC_TEARDOWN_QUEUE: LazyLock<Mutex<Vec<Teardown>>> = LazyLock::new(|| {
    shutdown_hooks::add_shutdown_hook(teardown_queue);
    Mutex::new(Vec::new())
});

pub(crate) fn teardown_queue_push(teardown: Teardown) {
    let mut queue = STATIC_TEARDOWN_QUEUE.lock().unwrap();
    queue.push(teardown);
}

extern "C" fn teardown_queue() {
    let mut teardown_list = STATIC_TEARDOWN_QUEUE.lock().unwrap();
    while let Some(mut teardown) = teardown_list.pop() {
        teardown.destroy();
    }
}

pub(crate) struct Teardown {
    pub(crate) base_temp_dir: Option<PathBuf>,
    pub(crate) func: Option<extern "C" fn()>
}

impl Teardown {
    pub(crate) fn destroy(&mut self) {
        if let Some(dir) = self.base_temp_dir.take() {
            if dir.exists() && std::fs::remove_dir_all(&dir).is_err() {
                eprintln!("Unable to delete base temp dir: {}", dir.to_str().unwrap());
            }
        }

        if let Some(func) = self.func.take() {
            func()
        }
    }
}
