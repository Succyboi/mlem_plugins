use mlem_base::metadata::PluginMetadata;
use nih_plug::prelude::*;

pub const PLUGIN_METADATA: PluginMetadata = PluginMetadata::new(
    "\u{E628}",
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
    "com.mlemrecords.mlem_meter", 
    *b"meterMLEM       ", 
    
    &[ClapFeature::AudioEffect, ClapFeature::Stereo], 
    &[Vst3SubCategory::Fx, Vst3SubCategory::Tools], 

    256,
    146
);