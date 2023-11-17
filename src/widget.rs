use egui::Color32;
use egui_extras::{Column, TableBuilder};

use crate::{nibble::u4, vm::Vm};

pub struct Widget {
	pub vm: Vm,
	pub other_vms: Vec<Vm>,
	pub write_target: u8,
	pub memory: [Option<u4>; 256],
}

const CELL_DIM: f32 = 15.0;
impl Widget {
	pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
		ui.vertical(|ui| {
			let table = TableBuilder::new(ui);
			let mut vm = self.vm.clone();
			table.columns(Column::exact(CELL_DIM), 17).body(|mut body| {
				body.row(CELL_DIM, |mut row| {
					row.col(|_ui| {});
					for n in 0..16 {
						row.col(|ui| {
							ui.label(char::from_digit(n, 16).unwrap().to_string());
						});
					}
				});
				for high in 0..16u8 {
					body.row(CELL_DIM, |mut row| {
						row.col(|ui| {
							ui.label(char::from_digit(high as u32, 16).unwrap().to_string());
						});
						for low in 0..16u8 {
							row.col(|ui| {
								let addr = u4::combine([high.into(), low.into()]);
								let is_open = vm.is_open_bus(&self.memory, addr);
								let old_read_reg = vm.last_read;
								let value: u8 = vm.read(&self.memory, addr).into();
								vm.last_read = old_read_reg;
								let value = char::from_digit(value as u32, 16).unwrap();
								let value = format!(
									"{value}{}",
									match addr {
										a if a == vm.instruction_pointer => "i",
										a if a == vm.stack_pointer => "s",
										_ => "",
									}
								);
								let color = if addr == self.write_target {
									Color32::GREEN
								} else if is_open {
									Color32::DARK_GRAY
								} else {
									Color32::WHITE
								};
								ui.colored_label(color, value);
							});
						}
					});
				}
			})
		})
		.response
	}
}
