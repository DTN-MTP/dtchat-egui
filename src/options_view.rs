use crate::main_view::MirroredData;
use eframe::egui;
use egui_file_dialog::FileDialog;

pub struct OptionsView {
    pub selected_algo: String,
    pub selected_cp_path: String,
    file_dialog: FileDialog,
}

impl OptionsView {
    pub fn new() -> Self {
        Self {
            selected_algo: "VolCgrHybridParenting".to_string(),
            selected_cp_path: "./db/ion.cp".to_string(),
            file_dialog: FileDialog::new(),
        }
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        _data: &mut MirroredData,
    ) -> Option<(String, String)> {
        self.file_dialog.update(ui.ctx());
        let mut update = false;
        ui.vertical(|ui| {
            ui.add_space(5.0);
            ui.heading("Options");
            ui.add_space(5.0);
            ui.separator();
            ui.add_space(15.0);

            //Contact Plan
            ui.horizontal(|ui| {
                ui.label("Change Contact Plan :");
                if ui.button("Explore 🌟").clicked() {
                    self.file_dialog.pick_file();
                }
            });

            if let Some(path) = self.file_dialog.take_picked() {
                self.selected_cp_path = path.display().to_string();
                update = true;
            }

            ui.add_space(10.0);

            //Algorithme
            ui.horizontal(|ui| {
                ui.label("Change Algo :");
                let old_algo = self.selected_algo.clone();

                let algos: Vec<&str> = vec![
                    "VolCgrNodeParenting",
                    "VolCgrHybridParenting",
                    "VolCgrHybridParentingHop",
                    "VolCgrNodeParentingHop",
                    #[cfg(feature = "contact_work_area")]
                    "VolCgrContactParenting",
                    #[cfg(feature = "contact_work_area")]
                    "VolCgrContactParentingHop",
                    #[cfg(feature = "contact_suppression")]
                    "CgrFirstEndingHybridParentingHop",
                    #[cfg(feature = "contact_suppression")]
                    "CgrFirstEndingHybridParenting",
                    #[cfg(feature = "contact_suppression")]
                    "CgrFirstEndingNodeParentingHop",
                    #[cfg(feature = "contact_suppression")]
                    "CgrFirstEndingNodeParenting",
                    #[cfg(all(feature = "contact_work_area", feature = "contact_suppression"))]
                    "CgrFirstEndingContactParenting",
                    #[cfg(all(feature = "contact_work_area", feature = "contact_suppression"))]
                    "CgrFirstEndingContactParentingHop",
                    #[cfg(all(feature = "contact_suppression", feature = "first_depleted"))]
                    "CgrFirstDepletedHybridParentingHop",
                    #[cfg(all(feature = "contact_suppression", feature = "first_depleted"))]
                    "CgrFirstDepletedHybridParenting",
                    #[cfg(all(feature = "contact_suppression", feature = "first_depleted"))]
                    "CgrFirstDepletedNodeParentingHop",
                    #[cfg(all(feature = "contact_suppression", feature = "first_depleted"))]
                    "CgrFirstDepletedNodeParenting",
                    #[cfg(all(
                        feature = "contact_work_area",
                        feature = "contact_suppression",
                        feature = "first_depleted"
                    ))]
                    "CgrFirstDepletedContactParentingHop",
                    #[cfg(all(
                        feature = "contact_work_area",
                        feature = "contact_suppression",
                        feature = "first_depleted"
                    ))]
                    "CgrFirstDepletedContactParenting",
                ];

                egui::ComboBox::from_id_salt("algo_selector")
                    .selected_text(&self.selected_algo)
                    .show_ui(ui, |ui| {
                        for algo in algos {
                            ui.selectable_value(&mut self.selected_algo, algo.to_string(), algo);
                        }
                    });
                if old_algo != self.selected_algo {
                    update = true;
                }
            });
        });
        if update {
            Some((self.selected_cp_path.clone(), self.selected_algo.clone()))
        } else {
            None
        }
    }
}
