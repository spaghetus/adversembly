use adversembly::{vm::Vm, widget::Widget};
use eframe::NativeOptions;
use egui::CentralPanel;
use smol::lock::RwLock;
use std::sync::Arc;

struct App {
	vm_widget: Widget,
	write_nibble: u8,
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		CentralPanel::default().show(ctx, |ui| {
			self.vm_widget.show(ui);
			ui.separator();
			if ui.button("Run cycle").clicked() {
				smol::block_on(self.vm_widget.vm.cycle());
			}
			ui.horizontal(|ui| {
				ui.label("Write:");
				ui.add(
					egui::widgets::DragValue::new(&mut self.vm_widget.write_target)
						.speed(0.2)
						.hexadecimal(2, false, true),
				);
				ui.add(
					egui::widgets::DragValue::new(&mut self.write_nibble)
						.speed(0.2)
						.hexadecimal(1, false, true)
						.clamp_range(0..=15),
				);
				if ui.button("Write").clicked() {
					smol::block_on(
						self.vm_widget
							.vm
							.write(self.vm_widget.write_target, self.write_nibble.into()),
					)
				}
			});
		});
	}
}

fn main() {
	let memory = Arc::new(RwLock::new([None; 256]));
	let app = App {
		vm_widget: Widget {
			write_target: 0,
			vm: Vm::new(memory),
			other_vms: vec![],
		},
		write_nibble: 0,
	};
	eframe::run_native(
		"Adversembly Emulator",
		NativeOptions::default(),
		Box::new(move |_| Box::new(app)),
	)
	.unwrap();
}
