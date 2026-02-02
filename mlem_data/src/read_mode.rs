use nih_plug::prelude::Enum;

#[derive(Enum, Debug, PartialEq)]
pub enum DataReadMode {
    #[id = "bit8"]
    Bit8,
    
    #[id = "bit16"]
    Bit16,
    
    #[id = "bit32"]
    Bit32
}