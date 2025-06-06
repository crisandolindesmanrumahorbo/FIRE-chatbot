use std::{
    env, ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

pub struct Config {
    pub llama_url: &'static str,
    pub tele_url: &'static str,
    pub tele_token: &'static str,
    pub vapid_private_key: &'static str,
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
        tele_url: Box::leak(
            env::var("TELE_URL")
                .expect("TELE_URL must be set")
                .into_boxed_str(),
        ),
        tele_token: Box::leak(
            env::var("TELE_TOKEN")
                .expect("TELE_TOKEN must be set")
                .into_boxed_str(),
        ),
        vapid_private_key: Box::leak(
            env::var("VAPID_PRIVATE_KEY")
                .expect("VAPID_PRIVATE_KEY must be set")
                .into_boxed_str(),
        ),
    });
    CONFIG.store(Box::into_raw(config), Ordering::Release);
}

#[inline]
pub fn get_config() -> &'static Config {
    unsafe { &*CONFIG.load(Ordering::Acquire) }
}
