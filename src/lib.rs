use nih_plug::prelude::*;
use nih_plug_egui::{
    create_egui_editor,
    resizable_window::ResizableWindow,
    widgets, EguiState,
};
use egui::{Vec2};
use std::sync::Arc;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

const MAX_DELAY_TIME: f32 = 2.0;

pub struct DelayPlugin {
    params: Arc<DelayPluginParams>,
    delay_buffers: Vec<Vec<f32>>, // one Vec per channel
    write_index: usize,
    sample_rate: f32,
}

#[derive(Params)]
struct DelayPluginParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "delay_time"]
    pub delay_time: FloatParam,
    #[id = "feedback"]
    pub feedback: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,

    // egui editor state
    #[persist = "editor-state"]
    egui_state: Arc<EguiState>,
}

impl Default for DelayPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(DelayPluginParams::default()),
            delay_buffers: Vec::new(),
            write_index: 0,
            sample_rate: 44100.0,
        }
    }
}

impl Default for DelayPluginParams {
    fn default() -> Self {
        Self {
            // default params for a delay plugin
            delay_time: FloatParam::new(
                "Delay Time",
                0.25,
                FloatRange::Skewed {
                    min: 0.001,
                    max: MAX_DELAY_TIME,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" ms")
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            feedback: FloatParam::new(
                "Feedback",
                0.5,
                FloatRange::Linear {
                    min: 0.0,
                    max: 0.98, // so we don't go into self-oscillation
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0))
            .with_value_to_string(formatters::v2s_f32_percentage(1)),

            mix: FloatParam::new("Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(10.0))
                .with_value_to_string(formatters::v2s_f32_percentage(1)),

            egui_state: EguiState::from_size(300, 180),
        }
    }
}

impl Plugin for DelayPlugin {
    const NAME: &'static str = "DelayPlugin";
    const VENDOR: &'static str = "Chris Condon";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "ccondon894@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let egui_state = params.egui_state.clone();
        create_egui_editor(
            self.params.egui_state.clone(), // egui state
            (), // user state
            Default::default(), // New Egui settings. Just use defaults
            |_, _, _| {},                      // build closure (now takes 3 args)
            move |egui_ctx, setter, _queue, _state| { // update closure now takes 4 args
                //update closure
                ResizableWindow::new("res-wind")
                    .min_size(Vec2::new(128.0, 128.0))
                    .show(egui_ctx, egui_state.as_ref(), |ui| {
                        ui.label("Delay Time");
                        ui.add(
                            widgets::ParamSlider::for_param(&params.delay_time, setter)
                                .with_width(ui.available_width())
                        );

                        ui.label("Feedback");
                        ui.add(
                            widgets::ParamSlider::for_param(&params.feedback, setter)
                                .with_width(ui.available_width())
                        );

                        ui.label("Mix");
                        ui.add(
                            widgets::ParamSlider::for_param(&params.mix, setter)
                                .with_width(ui.available_width())
                        );
                    });
            },
        )
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        self.sample_rate = buffer_config.sample_rate;
        let channel_count = audio_io_layout
            .main_input_channels
            .expect("main input channels must be set")
            .get() as usize;
        // buffer sizing math: buffer size decides how much history the buffer needs to store
        // max delay is 2 seconds, and sample rate is samples per second.
        // So max delay * sample rate is the total number of samples we need to store in the buffer.
        let max_delay_samples = (MAX_DELAY_TIME * self.sample_rate).ceil() as usize;
        self.delay_buffers = vec![vec![0.0; max_delay_samples + 1]; channel_count];
        true
    }

    fn reset(&mut self) {
        // Reset delay buffers and write index to 0
        // remember to never allocate in the audio thread. This would be a heap
        // operation and can block the audio thread, causing glitches/pops.
        for buf in &mut self.delay_buffers {
            buf.fill(0.0);
        }
        self.write_index = 0;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let buf_len = self.delay_buffers[0].len(); // get the buffer length from initialization

        for mut channel_samples in buffer.iter_samples() {
            // Read parameters with smoothing applied
            let delay_time = self.params.delay_time.smoothed.next();
            let feedback = self.params.feedback.smoothed.next();
            let mix = self.params.mix.smoothed.next();
            let delay_samples = (delay_time * self.sample_rate) as usize; // compute number of delay samples
            for (channel_idx, sample) in channel_samples.iter_mut().enumerate() {
                let delay_buffer = &mut self.delay_buffers[channel_idx]; //get delay buffer
                let read_index = (self.write_index + buf_len - delay_samples) % buf_len; // get the read index
                let delayed = delay_buffer[read_index]; //get the delayed buffer sample
                let dry = *sample; // get the dry sample
                delay_buffer[self.write_index] = dry + feedback * delayed; // write the delayed + feedbacked signal to buffer

                *sample = (1.0 - mix) * dry + mix * delayed;
            }
            self.write_index = (self.write_index + 1) % buf_len;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for DelayPlugin {
    const CLAP_ID: &'static str = "com.your-domain.delay-plugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A short description of your plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for DelayPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"Exactly16Chars!!";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(DelayPlugin);
nih_export_vst3!(DelayPlugin);
