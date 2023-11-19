#[cfg(feature = "assembler")]
use adversembly::assembler::Syntax;
use adversembly::{
	vm::{load_memory_from_file, Vm},
	widget::Widget,
};
#[cfg(feature = "assembler")]
use chumsky::Parser;
use eframe::NativeOptions;
use egui::{CentralPanel, DragValue};
use std::{
	path::PathBuf,
	time::{Duration, Instant},
};

#[derive(clap::Parser)]
struct Args {
	pub file: Option<PathBuf>,
	#[cfg(feature = "assembler")]
	#[arg(short, long)]
	pub assemble: bool,
}

struct App {
	vm_widget: Widget,
	write_nibble: u8,
	autorun: bool,
	autorun_multiplier: u32,
	frametime: Instant,
	secondtime: Instant,
	iter_count: u128,
	find_max: bool,
	max_rate: u128,
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		CentralPanel::default().show(ctx, |ui| {
			self.vm_widget.show(ui);
			ui.separator();
			if ui.button("Run cycle").clicked() {
				self.vm_widget.vm.cycle(&mut self.vm_widget.memory);
			}
			if self.autorun {
				for _ in 0..self.autorun_multiplier {
					self.vm_widget.vm.cycle(&mut self.vm_widget.memory);
				}
				self.iter_count += self.autorun_multiplier as u128;
				ctx.request_repaint();
			}
			ui.horizontal(|ui| {
				ui.label("Write:");
				ui.add(
					egui::widgets::DragValue::new(&mut self.vm_widget.write_target)
						.speed(0.05)
						.hexadecimal(2, false, true),
				);
				ui.add(
					egui::widgets::DragValue::new(&mut self.write_nibble)
						.speed(0.05)
						.hexadecimal(1, false, true)
						.clamp_range(0..=15),
				);
				if ui.button("Write").clicked() {
					self.vm_widget.vm.write(
						&mut self.vm_widget.memory,
						self.vm_widget.write_target,
						self.write_nibble.into(),
					);
					self.vm_widget.write_target = self.vm_widget.write_target.overflowing_add(1).0;
				}
			});
			ui.horizontal(|ui| {
				ui.checkbox(&mut self.autorun, "Autorun");
				ui.add(
					DragValue::new(&mut self.autorun_multiplier)
						.clamp_range(1..=u32::MAX)
						.speed(0.1),
				);
				ui.checkbox(&mut self.find_max, "Find maximum multiplier");
			});
			if self.autorun {
				let elapsed = self.frametime.elapsed();
				self.frametime = Instant::now();
				let rate = Duration::from_secs(1).as_micros() / elapsed.as_micros();
				ui.label(format!("{rate}fps"));
				if self.find_max {
					if rate > 40 {
						self.autorun_multiplier += 1 + self.autorun_multiplier / 10;
					} else {
						self.autorun_multiplier -= self.autorun_multiplier / 101;
					}
					ui.label(format!("peak {}hz", self.max_rate));
				}
				if self.secondtime.elapsed().as_secs() >= 1 {
					self.max_rate = self.max_rate.max(self.iter_count);
					self.secondtime = Instant::now();
					self.iter_count = 0;
				}
			}
		});
	}
}

fn main() {
	let args = <Args as clap::Parser>::parse();
	let memory = if let Some(f) = args.file {
		let file = std::fs::read_to_string(f).expect("Failed to read file");
		#[cfg(feature = "assembler")]
		if args.assemble {
			let mut memory = [None; 256];
			let program = adversembly::assembler::program()
				.parse(file)
				.expect("Parse failure");
			println!("{program:#?}");
			let program = Syntax::compile(program);
			for (addr, data) in program.iter().enumerate() {
				memory[addr] = Some(*data);
			}
			memory
		} else {
			load_memory_from_file(file)
		}
		#[cfg(not(feature = "assembler"))]
		load_memory_from_file(file)
	} else {
		[None; 256]
	};
	let app = App {
		vm_widget: Widget {
			write_target: 0,
			vm: Vm::new(),
			other_vms: vec![],
			memory,
		},
		write_nibble: 0,
		autorun: false,
		autorun_multiplier: 1,
		frametime: Instant::now(),
		secondtime: Instant::now(),
		iter_count: 0,
		find_max: false,
		max_rate: 0,
	};
	eframe::run_native(
		"Adversembly Emulator",
		NativeOptions::default(),
		Box::new(move |_| Box::new(app)),
	)
	.unwrap();
}
