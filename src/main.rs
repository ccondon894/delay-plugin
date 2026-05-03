use nih_plug::prelude::*;
use delay_plugin::DelayPlugin;

fn main() {
    nih_export_standalone::<DelayPlugin>();
}
