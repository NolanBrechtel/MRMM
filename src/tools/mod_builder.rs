use egui::text_edit;
use serde::{Deserialize, Serialize};
use crate::mod_types::{ModType, Modification, MultiPak};

#[derive(Debug)]
pub struct ModBuilder {
    building_multi_pak: bool,
    modification: ModType,

}
fn toggle(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}
impl ModBuilder {
    pub fn new() -> Self {
        Self {
            building_multi_pak: false,
            modification: ModType::Complete(Modification::new()),
        }
    }

}
impl eframe::App for ModBuilder {
    fn update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(_ctx, |ui| {
            ui.heading("Mod Builder");
            ui.add_space(10.0);

            // Custom sliding switch
            ui.horizontal(|ui| {
                ui.label("Regular Mod");
                if toggle(ui, &mut self.building_multi_pak).changed() {
                    if self.building_multi_pak {
                        println!("Switch to MultiPak");
                        self.modification = ModType::MultiPak(MultiPak::default())
                    } else {
                        println!("Switch to Single Pak");
                        self.modification = ModType::Complete(Modification::new())
                    }
                }
                ui.label("MultiPak Mod");
            });
            ui.separator();

            match &mut self.modification {
                ModType::Complete(ref mut cm) => {
                    ui.horizontal(|ui| {
                        ui.label("Name: ");
                        if ui.add(text_edit::TextEdit::singleline(&mut cm.name)).changed() {
                            println!("{}", cm.name);
                        }
                    });
                }
                ModType::MultiPak(mp) => {

                }
                _ => {
                    eprintln!("Unknown ModType");
                }
            }
        });
    }
}