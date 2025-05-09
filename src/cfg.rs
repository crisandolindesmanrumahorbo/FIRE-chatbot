use std::{
    env, ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

pub struct Config {
    pub llama_url: &'static str,
}

static CONFIG: AtomicPtr<Config> = AtomicPtr::new(ptr::null_mut());

pub fn init_config() {
    dotenvy::dotenv().ok();
    let config = Box::new(Config {
        llama_url: Box::leak(
            env::var("LLAMA_URL")
                .expect("LLAMA_URL must be set")
                .into_boxed_str(),
        ),
    });
    CONFIG.store(Box::into_raw(config), Ordering::Release);
}

#[inline]
pub fn get_config() -> &'static Config {
    unsafe { &*CONFIG.load(Ordering::Acquire) }
}
