use mlem_base::metadata::PluginMetadata;
use nih_plug::prelude::*;

pub const PLUGIN_METADATA: PluginMetadata = PluginMetadata::new(
    "\u{E564}",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_HOMEPAGE"),
    env!("CARGO_PKG_HOMEPAGE"),
    env!("CARGO_PKG_VERSION"),
    env!("CARGO_PKG_AUTHORS"),
    env!("CARGO_PKG_DESCRIPTION"),
    "ANTI-CAPITALIST SOFTWARE LICENSE v 1.4",
    include_str!("../LICENSE"),
    concat!(include_str!("../credits.txt"), "\n\n", include_str!("../../mlem_base/credits.txt")),
    "Mlem Records",
    "support@mlemrecords.com", 
    "com.mlemrecords.mlem_data", 
    *b"dataMLEM        ", 
    
    &[ClapFeature::AudioEffect, ClapFeature::Stereo], 
    &[Vst3SubCategory::Fx, Vst3SubCategory::Tools], 

    384,
    384
);
pub const DISCLAIMER: &str = include_str!("../disclaimer.txt");
pub const DEFAULT_DATA: &[u8; 10763] = include_bytes!("../default_data");