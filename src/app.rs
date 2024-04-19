use std::{
	io::prelude::*,
	path, path::Path,
	fs, fs::File,
};

use tracker::track;

use relm4::{
	gtk, gtk::prelude::*, gtk::{gio, glib},
	adw, adw::prelude::*,
	actions::{
		RelmAction, RelmActionGroup, AccelsPlus,
	},
	main_application, Component, ComponentParts, ComponentSender,
	Controller, SimpleComponent,
};

use crate::{
	config::{APP_ID},
	i18n::i18n,
};

use sourceview5::prelude::*;

#[derive(Debug, PartialEq)]
pub(super) enum WindowPage {
	Home,
	Editor,
}

#[derive(Debug, PartialEq)]
pub(super) enum SidebarPage {
	Sections,
	Formatting,
	SpellCheck,
	FindReplace,
}

#[tracker::track]
pub(super) struct App {
	visible_sidebar_page: SidebarPage,
	visible_window_page: WindowPage,
	is_stat_dialog_visible: bool,
	is_page_empty: bool,
	word_count: usize,
	is_dark: bool,
	ideas: Vec<String>,
	title: String,
	text: String,
	headings: Vec<String>,
}

#[derive(Debug)]
pub enum AppMsg {
	SwitchSidebarPage(SidebarPage),
	SwitchWindowPage(WindowPage),
	ChangeTheme(bool),
	ChangeText(String),
	ChangeTitle(String),
	ToggleStatDialog,
	Quit,
}

relm4::new_action_group!(AppActionGroup, "app");
relm4::new_stateless_action!(QuitAction, AppActionGroup, "quit");
relm4::new_stateless_action!(FormattingAction, AppActionGroup, "formatting");
relm4::new_stateless_action!(DeleteAction, AppActionGroup, "delete");

fn strip_headings(src: &str) -> Vec<&str> {
	lazy_static::lazy_static! {
		static ref RE: regex::Regex = regex::Regex::new(r"#+ .+").unwrap();
	}
	RE.find_iter(src)
		.map(|x| x.as_str())
		.collect()
}

struct Writing {
	id: String,
}

impl App {
	fn load_writings() {
		let dir = Path::new("/home/kinten/.local/share/austeur/writings");
		if !dir.is_dir() {
			return ();
		};
		fs::read_dir(dir).unwrap()
			.map(|res| res.map(|e| e.path()))
        	.collect::<Result<Vec<_>, std::io::Error>>()
        	.into_iter()
        	.map(|paths| paths.into_iter().filter_map(|path| {
        		let mut file = match File::open(path) {
        			Ok(val) => val,
        			Err(..) => {
        				return None;
        			},
        		};
        		let mut contents = String::new();
        		let result = match file.read_to_string(&mut contents) {
        			Ok(..) => true,
        			Err(..) => {
        				return None;
        			},
        		};
        		let contents_str = contents.as_str();
        		let parsed = match json::parse(contents_str) {
        			Ok(val) => val,
        			Err(..) => {
        				return None;
        			},
        		};
        		println!("parsed: {}", parsed);
        		Some(parsed)
        	}).collect::<Vec<_>>())
        	.collect::<Vec<_>>();
	}
}

#[relm4::component(pub)]
impl SimpleComponent for App {
	type Init = (SidebarPage,sourceview5::StyleSchemeManager,WindowPage);
	type Input = AppMsg;
	type Output = ();

	additional_fields! {
		text_style_manager: sourceview5::StyleSchemeManager,
	}

	menu! {
		primary_menu: {
			section! {
				&i18n("_Formatting") => FormattingAction,
			},

			section! {
				&i18n("_Delete") => DeleteAction,
			}
		}
	}

	view! {
		adw::StyleManager::default() {
			connect_dark_notify[sender] => move |style_manager| {
				sender.input(AppMsg::ChangeTheme(style_manager.is_dark()));
			},
		},

		editor_title_text_buffer = sourceview5::Buffer {
			#[track = "model.changed(App::is_dark())"]
			set_style_scheme: {
				text_style_manager.scheme({
					if model.is_dark { "austeur-default-dark" } else { "austeur-default" }
				}.as_ref()).as_ref()
			},

			connect_changed[sender] => move |buffer| {
				let (start, end) = buffer.bounds();
				let text = buffer.slice(&start, &end, true);
				sender.input(AppMsg::ChangeTitle(text.to_string()));
			},
		},

		text_view_buffer = sourceview5::Buffer {
			#[track = "model.changed(App::is_dark())"]
			set_style_scheme: {
				text_style_manager.scheme({
					if model.is_dark { "austeur-default-dark" } else { "austeur-default" }
				}.as_ref()).as_ref()
			},

			connect_changed[sender] => move |buffer| {
				let (start, end) = buffer.bounds();
				let text = buffer.slice(&start, &end, true);
				sender.input(AppMsg::ChangeText(text.into()));
			},
		},

		stat_dialog = adw::Window {
    		set_hide_on_close: true,
    		set_modal: true,
    		set_transient_for: Some(root),
    		#[track = "model.changed(App::is_stat_dialog_visible())"]
    		set_visible: model.is_stat_dialog_visible,

    		connect_close_request[sender] => move |_| {
				sender.input(AppMsg::ToggleStatDialog);
				gtk::Inhibit(true)
			},

    		adw::ToolbarView {
    			add_top_bar = &adw::HeaderBar {
    				#[wrap(Some)]
    				set_title_widget = &adw::WindowTitle {
    					set_title: &i18n("Statistics"),
    				},
    			},

    			#[wrap(Some)]
    			set_content = &gtk::Box {
    				set_width_request: 300,
    				set_height_request: 300,
    			},
    		}
    	},

		#[root]
		main_window = adw::ApplicationWindow::new(&main_application()) {
			set_default_width: 940,
			set_default_height: 400,
			set_resizable: false,

			connect_close_request[sender] => move |_| {
				sender.input(AppMsg::Quit);
				glib::Propagation::Stop
			},

			#[transition = "Crossfade"]
			match model.visible_window_page {
				WindowPage::Home => {
					gtk::Box {
						set_orientation: gtk::Orientation::Vertical,

						adw::HeaderBar {

							#[wrap(Some)]
							set_title_widget = &gtk::SearchEntry {
							},

							pack_start = &gtk::Button {
								add_css_class: "thin",
								add_css_class: "bg-accent",
								add_css_class: "text-accent-fg",
								set_halign: gtk::Align::End,

								connect_clicked[sender] => move |_| {
									sender.input(AppMsg::SwitchWindowPage(WindowPage::Editor));
								},

								gtk::Box {
									set_spacing: 4,

									gtk::Image {
										set_icon_name: Some("plus-symbolic"),
									},

									gtk::Label {
										set_label: &i18n("New Idea"),
									},
								},
							},

							pack_end = &gtk::MenuButton {
								set_icon_name: "open-menu-symbolic",
							},

							pack_end = &gtk::ToggleButton {
								set_icon_name: "loupe-symbolic",
							},
						},

						gtk::ScrolledWindow {
							set_vexpand: true,
							set_hscrollbar_policy: gtk::PolicyType::Never,
							set_vscrollbar_policy: gtk::PolicyType::Automatic,

							adw::Clamp {
								set_maximum_size: 300,

								gtk::Box {
									set_margin_top: 12,
									set_margin_bottom: 12,
									set_spacing: 12,
									set_orientation: gtk::Orientation::Vertical,

									adw::PreferencesGroup {
										set_title: "Ideas",
										set_description: Some(&i18n("Quickly write an idea with the + button above and it'll appear here")),
										#[watch]
										set_visible: model.title.len() > 0 || model.text.len() > 0,

										gtk::ListBox {
											add_css_class: "boxed-list",
											set_selection_mode: gtk::SelectionMode::None,

											adw::ActionRow {
												#[watch]
												set_title: {
													if model.title.len() > 0 {
														model.title.as_str()
													} else {
														""
													}
												},

												#[watch]
												set_subtitle: {
													if model.text.len() > 0 {
														model.text.as_str()
													} else {
														""
													}
												},

												set_subtitle_lines: 2,
												set_activatable_widget: Some(&activatable_button),

												add_suffix: activatable_button = &gtk::Button {
													set_valign: gtk::Align::Center,
													add_css_class: "flat",

													connect_clicked[sender] => move |_| {
														sender.input(AppMsg::SwitchWindowPage(WindowPage::Editor));
													},

													gtk::Image {
														set_icon_name: Some("go-next-symbolic"),
														add_css_class: "dim-label",
													},
												},
											},
										}
									},
								},
							},
						},
					}
				}

				WindowPage::Editor => {
					adw::NavigationSplitView {
						#[wrap(Some)]
						set_sidebar = &adw::NavigationPage {
							#[wrap(Some)]
							set_child = &adw::ToolbarView {
								add_top_bar = &adw::HeaderBar {
									#[wrap(Some)]
									set_title_widget = &gtk::Box {
										add_css_class: "linked",

										gtk::ToggleButton {
											set_icon_name: "text-justify-left-symbolic",
											set_tooltip_text: Some(&i18n("Sections")),
											add_css_class: "wide",
											add_css_class: "flat",
											#[track = "model.changed(App::visible_sidebar_page())"]
											set_active: model.visible_sidebar_page == SidebarPage::Sections,

											connect_clicked[sender] => move |_| {
												sender.input(AppMsg::SwitchSidebarPage(SidebarPage::Sections));
											},
										},

										gtk::ToggleButton {
											set_icon_name: "text-squiggly-symbolic",
											set_tooltip_text: Some(&i18n("Spell Check")),
											add_css_class: "wide",
											add_css_class: "flat",
											#[track = "model.changed(App::visible_sidebar_page())"]
											set_active: model.visible_sidebar_page == SidebarPage::SpellCheck,

											connect_clicked[sender] => move |_| {
												sender.input(AppMsg::SwitchSidebarPage(SidebarPage::SpellCheck));
											},
										},

										gtk::ToggleButton {
											set_icon_name: "loupe-symbolic",
											set_tooltip_text: Some(&i18n("Find & Replace")),
											add_css_class: "wide",
											add_css_class: "flat",
											#[track = "model.changed(App::visible_sidebar_page())"]
											set_active: model.visible_sidebar_page == SidebarPage::FindReplace,

											connect_clicked[sender] => move |_| {
												sender.input(AppMsg::SwitchSidebarPage(SidebarPage::FindReplace));
											},
										},
									},

									pack_start = &gtk::Button {
										set_icon_name: "pip-out-symbolic",
										set_tooltip_text: Some(&i18n("View Projects")),

										connect_clicked[sender] => move |_| {
											sender.input(AppMsg::SwitchWindowPage(WindowPage::Home));
										}
									},

									pack_end = &gtk::MenuButton {
										set_icon_name: "view-more-symbolic",
										set_tooltip_text: Some(&i18n("Menu")),
										set_primary: true,
										set_popover: Some(&{
											let popover = gtk::PopoverMenu::from_model(Some(&primary_menu));
											popover.add_css_class("destruction-at-last");
											popover
										}),
									},
								},

								#[wrap(Some)]
								set_content = match model.visible_sidebar_page {
									SidebarPage::Sections => {
										gtk::Box {
											if model.headings.len() > 0 {
												gtk::Box {

												}
											} else {
												adw::StatusPage {
													set_hexpand: true,
													set_title: &i18n("No Chapters"),
													set_description: Some(&i18n("Start writing and your chapters will be listed here")),
													add_css_class: "compact"
												}
											}
										}
									},

									SidebarPage::Formatting => {
										gtk::Label {
											set_label: "formatting",
										}
									},

									SidebarPage::SpellCheck => {
										gtk::Label {
											set_label: "spellcheck",
										}
									},

									SidebarPage::FindReplace => {
										gtk::Label {
											set_label: "findreplace",
										}
									}
								},
							},
						},
						#[wrap(Some)]
						set_content = &adw::NavigationPage {
							#[wrap(Some)]
							set_child = &adw::ToolbarView {
								add_top_bar = if model.is_page_empty {
									adw::HeaderBar {
										pack_end = &gtk::Button {
											set_tooltip_text: Some(&i18n("Generate Prompt")),
											#[iterate]
											add_css_class: vec!["thin", "outlined", "primary"],

											gtk::Box {
												set_spacing: 4,

												gtk::Image {
													set_icon_name: Some("lightbulb-symbolic"),
												},

												gtk::Label {
													set_label: "Prompt"
												},
											},
										},

										pack_end = &gtk::Button {
											set_tooltip_text: Some(&i18n("New from File")),
											#[iterate]
											add_css_class: vec!["thin", "outlined", "primary"],

											gtk::Box {
												set_spacing: 4,

												gtk::Image {
													set_icon_name: Some("paper-symbolic"),
												},

												gtk::Label {
													set_label: "Import"
												},
											},
										},
									}
								} else {
									adw::HeaderBar {
										#[wrap(Some)]
										set_title_widget = &gtk::Label {
											set_label: "Rô-bô Pilot",
											add_css_class: "font-bold",
										},

										pack_end = &gtk::Button {
											set_tooltip_text: Some(&i18n("Show Statistics")),
											#[track = "model.changed(App::word_count())"]
											set_label: &format!("{}", model.word_count),
											set_use_underline: true,
											#[iterate]
											add_css_class: vec!["font-medium", "thin", "outlined", "primary"],

											connect_clicked[sender] => move |_| {
												sender.input(AppMsg::ToggleStatDialog);
											},
										},

										pack_start = &gtk::Button {
											set_tooltip_text: Some(&i18n("Generate Prompt")),
											#[iterate]
											add_css_class: vec!["thin", "outlined", "primary"],

											gtk::Box {
												set_spacing: 4,

												gtk::Image {
													set_icon_name: Some("lightbulb-symbolic"),
												},

												gtk::Label {
													set_label: "Prompt"
												},
											},
										},
									}
								},

								#[wrap(Some)]
								set_content = &gtk::ScrolledWindow {
									set_vscrollbar_policy: gtk::PolicyType::Automatic,
									set_hscrollbar_policy: gtk::PolicyType::Never,

									adw::Clamp {
										set_maximum_size: 800,

										gtk::Box {
											set_orientation: gtk::Orientation::Vertical,
											set_margin_top: 6,
											set_margin_bottom: 24,
											set_spacing: 6,

											sourceview5::View::with_buffer(&text_view_buffer) {
												set_hexpand: true,
												set_vexpand: true,
												set_wrap_mode: gtk::WrapMode::WordChar,
												set_accepts_tab: false,
												set_left_margin: 16,
												set_right_margin: 8,
											},
										},
									},
								},
							},
						},
					}
				}
			}
		}
	}

	fn init(
		init: Self::Init,
		root: &Self::Root,
		sender: ComponentSender<Self>,
	) -> ComponentParts<Self> {
		let (visible_sidebar_page, text_style_manager, visible_window_page) = init;

		let mut ideas = Vec::<String>::new();
		ideas.push("51a".to_string()); // placeholder

		let a = App::load_writings();

		let model = Self {
			visible_sidebar_page,
			visible_window_page,
			is_stat_dialog_visible: false,
			is_page_empty: true,
			word_count: 0,
			is_dark: adw::StyleManager::default().is_dark(),
			ideas,
			title: "".to_string(),
			text: "".try_into().unwrap(),
			headings: Vec::new(),
            tracker: 0,
		};

		let widgets = view_output!();

		let app = main_application();

		let mut actions = RelmActionGroup::<AppActionGroup>::new();

	    let quit_action = {
	        let app = app.clone();
	        RelmAction::<QuitAction>::new_stateless(move |_| {
	            app.quit();
	        })
	    };
	    actions.add_action(quit_action);
		app.set_accelerators_for_action::<QuitAction>(&["<Control>q"]);

		let delete_action = {
			RelmAction::<DeleteAction>::new_stateless(move |_| {

			})
		};
		actions.add_action(delete_action);
		app.set_accelerators_for_action::<DeleteAction>(&["<Control>d"]);

	    actions.register_for_main_application();

		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
		self.reset();

		match message {
			AppMsg::Quit => main_application().quit(),
			AppMsg::SwitchSidebarPage(page) => {
				self.set_visible_sidebar_page(page);
			},
			AppMsg::SwitchWindowPage(page) => {
				self.set_visible_window_page(page);
			},
			AppMsg::ChangeTheme(is_dark) => {
				self.set_is_dark(is_dark);
			},
			AppMsg::ChangeText(text) => {
				let words_count::WordsCount { words , .. } = words_count::count(text.clone());
				self.set_word_count(words);
				self.set_is_page_empty(*self.get_word_count() <= 0);
				let text_a = text.as_str();
				let headings: Vec<String> = strip_headings(text_a).into_iter().map(|x: &str| x.to_owned()).collect();
				println!("headings: {}", {
					let headings = headings.clone();
					headings.into_iter().fold("".to_string(), |acc: String, x: String| acc + &x)
				});
				self.set_headings(headings);
				self.set_text(text);
			},
			AppMsg::ChangeTitle(text) => {
				self.set_title(text);
			},
			AppMsg::ToggleStatDialog => {
				self.set_is_stat_dialog_visible(!self.get_is_stat_dialog_visible());
			},
		}
	}

	fn shutdown(&mut self, widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
		widgets.save_window_size().unwrap();
	}
}

impl AppWidgets {
	fn load_editor(&self) -> Result<(), glib::BoolError> {
		//

		Ok(())
	}

	fn save_window_size(&self) -> Result<(), glib::BoolError> {
		let settings = gio::Settings::new(APP_ID);
		let (width, height) = self.main_window.default_size();

		settings.set_int("window-width", width)?;
		settings.set_int("window-height", height)?;

		settings.set_boolean("is-maximized", self.main_window.is_maximized())?;

		Ok(())
	}

	fn load_window_size(&self) {
		let settings = gio::Settings::new(APP_ID);

		let width = settings.int("window-width");
		let height = settings.int("window-height");
		let is_maximized = settings.boolean("is-maximized");

		self.main_window.set_default_size(width, height);

		if is_maximized {
			self.main_window.maximize();
		}
	}
}
