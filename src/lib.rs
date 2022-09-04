mod plugin;
mod client;
mod config;

pub use {
    std::boxed::Box,
    plugin::GeyserRedisPlugin,
    solana_geyser_plugin_interface::
        geyser_plugin_interface::{ 
            self, GeyserPlugin,
        },
};

#[no_mangle]
#[allow(improper_ctype_definitions)]
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin{
    let geyser_redis_plugin: Box<dyn GeyserPlugin> = Box::new(GeyserRedisPlugin{
        client: None,
    });
}
