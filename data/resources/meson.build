# Resources
resources = gnome.compile_resources(
	'resources',
	'resources.gresource.xml',
	gresource_bundle: true,
	source_dir: meson.current_build_dir(),
	install: true,
	install_dir: pkgdatadir,
)

install_data(
	'austeur-default.xml',
	install_dir: pkgdatadir,
)

install_data(
	'austeur-default-dark.xml',
	install_dir: pkgdatadir,
)
