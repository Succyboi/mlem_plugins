use nih_plug::prelude::*;

pub const ICON: &str = "\u{E628}";
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(debug_assertions)]        pub const BUILD_TYPE: &str = "debug";
#[cfg(not(debug_assertions))]   pub const BUILD_TYPE: &str = "release";
#[cfg(debug_assertions)]        pub const BUILD_IS_DEBUG: bool = true;
#[cfg(not(debug_assertions))]   pub const BUILD_IS_DEBUG: bool = false;
pub const BUILD_ID: &str = env!("BUILD_ID");

pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const LICENSE_NAME: &str = "ANTI-CAPITALIST SOFTWARE LICENSE v 1.4";
pub const LICENSE_CONTENTS: &str = include_str!("../LICENSE");
pub const CREDITS: &str = include_str!("../credits.txt");

pub const PLUGIN_VENDOR: &str = "Mlem Records";
pub const PLUGIN_EMAIL: &str = "puk@mlemrecords.com";
pub const PLUGIN_ID: &str = "com.mlemrecords.mlem_meter";
pub const PLUGIN_CLASS_ID: [u8; 16] = *b"meterMLEM       ";
pub const PLUGIN_CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
pub const PLUGIN_VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];

#[cfg(debug_assertions)]        pub const DEBUG_CLIP: bool = true;
#[cfg(not(debug_assertions))]   pub const DEBUG_CLIP: bool = false;

pub const MOTD: &str = "May your levels be accurate! ♥";

pub const WINDOW_SIZE_WIDTH: u32 = 256;
pub const WINDOW_SIZE_HEIGHT: u32 = 146;

pub const DARKMODE_DEFAULT: bool = true;