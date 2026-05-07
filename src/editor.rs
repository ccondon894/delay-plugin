use crate::DelayPluginParams;
use std::sync::Arc;
use nih_plug::prelude::*;
use egui::Vec2;
use nih_plug_egui::{
    create_egui_editor,
    resizable_window::ResizableWindow,
    widgets ,
};

pub fn create(params: Arc<DelayPluginParams>) -> Option<Box<dyn Editor>> {
    let params = params.clone();
    let egui_state = params.egui_state.clone();
    create_egui_editor(
        params.egui_state.clone(), // egui state
        (), // user state
        Default::default(), // New Egui settings. Just use defaults
        |_, _, _| {},                      // build closure (now takes 3 args)
        move |egui_ctx, setter, _queue, _state| { // update closure now takes 4 args
            //update closure
            ResizableWindow::new("res-wind")
                .min_size(Vec2::new(300.0, 128.0))
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

                    ui.label("Cutoff");
                    ui.add(
                        widgets::ParamSlider::for_param(&params.cutoff, setter)
                            .with_width(ui.available_width())
                    );
                });
        },
    )
    
}