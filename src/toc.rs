use crate::{
	app::{
		AppMsg
	},
};

use relm4::{
	prelude::*,
	factory, factory::{ FactoryComponent },
	gtk, gtk::prelude::*
};

#[derive(Debug)]
pub(super) struct Section {
	index: u8,
	name: String,
}

#[derive(Debug)]
pub(super) enum SectionMessage {
	
}

#[derive(Debug)]
pub(super) enum SectionOutput {
	ScrollToHere,
}

#[factory(pub)]
impl FactoryComponent for Section {
	type Init = (u8, String);
	type Input = SectionMessage;
	type Output = SectionOutput;
	type CommandOutput = ();
	type ParentWidget = gtk::Box;

	view! {
		root = gtk::ToggleButton {
			set_label: &self.name,
		}
	}

	fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
		Self {
			index: value.0,
			name: value.1
		}
	}
}
