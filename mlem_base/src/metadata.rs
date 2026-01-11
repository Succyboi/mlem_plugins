use nih_plug::{ prelude::* };
use crate::consts;

pub struct PluginMetadata {
    pub build_type: &'static str,
    pub build_is_debug: bool,
    pub build_id: &'static str,

    pub icon: &'static str,
    pub name: &'static str,
    pub homepage_url: &'static str,
    pub support_url: &'static str,
    pub version: &'static str,

    pub authors: &'static str,
    pub description: &'static str,
    pub license_name: &'static str,
    pub license_contents: &'static str,
    pub credits: &'static str,

    pub vendor: &'static str,
    pub email: &'static str,
    pub identifier: &'static str,
    pub class_identifier: [u8; 16],
    pub clap_features: &'static [ClapFeature],
    pub vst3_subcategories: &'static [Vst3SubCategory],

    pub window_width: u32,
    pub window_height: u32
}

impl PluginMetadata {
    pub const fn new(
        icon: &'static str,
        name: &'static str,
        homepage_url: &'static str,
        support_url: &'static str,
        version: &'static str,
        authors: &'static str,
        description: &'static str,
        license_name: &'static str,
        license_contents: &'static str,
        credits: &'static str,
        vendor: &'static str,
        email: &'static str,
        identifier: &'static str,
        class_identifier: [u8; 16],
        
        clap_features: &'static [ClapFeature],
        vst3_subcategories: &'static [Vst3SubCategory],
        
        window_width: u32,
        window_height: u32
    ) -> Self {
        Self {
            build_type: consts::BUILD_TYPE,
            build_is_debug: consts::BUILD_IS_DEBUG,
            build_id: consts::BUILD_ID,

            icon,
            name,
            homepage_url,
            support_url,
            version,
            authors,
            description,
            license_name,
            license_contents,
            credits,
            vendor,
            email,
            identifier,
            class_identifier,
            
            clap_features,
            vst3_subcategories,
            
            window_width,
            window_height
        }
    }
}