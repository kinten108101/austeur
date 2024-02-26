mod app;
#[rustfmt::skip]
mod config;
mod i18n;

use gettextrs::{gettext, LocaleCategory};

use relm4::{
    actions::{
    	AccelsPlus, RelmAction, RelmActionGroup
    },
    gtk, gtk::{
    	prelude::*,
    	gio, glib,
    },
    adw, adw::prelude::*,
    main_application, RelmApp,
};

use crate::{
	config::{APP_ID, GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE, PKGDATADIR},
	app::{
		App, SidebarPage, WindowPage,
	},
};

fn main() -> () {
    // Initialize GTK
    gtk::init().unwrap();

    relm4_icons::initialize_icons();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Could not bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Could not switch to the text domain");

    glib::set_application_name(&gettext("Austeur"));

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let text_style_manager = sourceview5::StyleSchemeManager::default();
	text_style_manager.set_search_path(&[PKGDATADIR,]);

    gtk::Window::set_default_icon_name(APP_ID);

    let app = main_application();
    app.set_resource_base_path(Some("/com/github/kinten108101/Austeur"));
    let app = RelmApp::from_app(app);
    app.run::<App>((SidebarPage::Sections, text_style_manager, WindowPage::Home));
}
