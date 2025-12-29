//! Bundle selection page

use adw::prelude::*;
use adw::subclass::prelude::*;
use common::actions::{default_bundles, BundleDef};
use common::{ArmCompat, CpuArch};
use gtk::glib;
use std::cell::RefCell;
use std::collections::HashSet;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct BundlesPage {
        pub enabled_bundles: RefCell<HashSet<String>>,
        pub bundle_rows: RefCell<Vec<(String, adw::ActionRow, gtk::Switch)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BundlesPage {
        const NAME: &'static str = "NixosToolkitBundlesPage";
        type Type = super::BundlesPage;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for BundlesPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for BundlesPage {}
    impl BoxImpl for BundlesPage {}
}

glib::wrapper! {
    pub struct BundlesPage(ObjectSubclass<imp::BundlesPage>)
        @extends gtk::Box, gtk::Widget;
}

impl BundlesPage {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .property("spacing", 24)
            .property("margin-start", 24)
            .property("margin-end", 24)
            .property("margin-top", 24)
            .property("margin-bottom", 24)
            .build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();
        let is_arm = CpuArch::detect().is_arm();

        // Title
        let title = gtk::Label::builder()
            .label("Software Bundles")
            .css_classes(["title-1"])
            .halign(gtk::Align::Start)
            .build();
        self.append(&title);

        // Description
        let desc = gtk::Label::builder()
            .label("Enable software bundles to install curated collections of applications.")
            .wrap(true)
            .halign(gtk::Align::Start)
            .css_classes(["dim-label"])
            .build();
        self.append(&desc);

        // ARM warning banner
        if is_arm {
            let arm_banner = adw::Banner::builder()
                .title("Running on ARM64 - some packages may not be available")
                .revealed(true)
                .build();
            arm_banner.add_css_class("warning");
            self.append(&arm_banner);
        }

        // Bundles group
        let bundles_group = adw::PreferencesGroup::builder()
            .title("Available Bundles")
            .build();

        // Add bundle rows
        let bundles = default_bundles();
        let mut bundle_rows = Vec::new();

        for bundle in &bundles {
            let (row, switch) = self.create_bundle_row(bundle, is_arm);
            bundles_group.add(&row);
            bundle_rows.push((bundle.id.clone(), row, switch));
        }

        *imp.bundle_rows.borrow_mut() = bundle_rows;
        self.append(&bundles_group);

        // Summary section
        let summary_group = adw::PreferencesGroup::builder()
            .title("Selected Packages")
            .description("Packages that will be installed")
            .build();

        let summary_label = gtk::Label::builder()
            .label("No bundles selected")
            .wrap(true)
            .halign(gtk::Align::Start)
            .css_classes(["dim-label"])
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let summary_frame = gtk::Frame::new(None);
        summary_frame.set_child(Some(&summary_label));
        summary_frame.add_css_class("card");

        summary_group.add(&summary_frame);
        self.append(&summary_group);
    }

    fn create_bundle_row(&self, bundle: &BundleDef, is_arm: bool) -> (adw::ActionRow, gtk::Switch) {
        // Build subtitle with ARM warning if needed
        let subtitle = if is_arm && bundle.arm_compat != ArmCompat::Full {
            if let Some(ref note) = bundle.arm_note {
                format!("{}\n⚠️ {}", bundle.description, note)
            } else {
                format!("{}\n⚠️ {}", bundle.description, bundle.arm_compat.display_name())
            }
        } else {
            bundle.description.clone()
        };

        let row = adw::ActionRow::builder()
            .title(&bundle.name)
            .subtitle(&subtitle)
            .build();

        // Add icon
        row.add_prefix(&gtk::Image::from_icon_name(&bundle.icon));

        // Add ARM compatibility indicator for ARM systems
        if is_arm && bundle.arm_compat != ArmCompat::Full {
            let compat_icon = match bundle.arm_compat {
                ArmCompat::None => gtk::Image::from_icon_name("action-unavailable-symbolic"),
                ArmCompat::Limited | ArmCompat::Partial => gtk::Image::from_icon_name("dialog-warning-symbolic"),
                ArmCompat::Full => gtk::Image::from_icon_name("emblem-ok-symbolic"),
            };
            compat_icon.add_css_class("warning");
            row.add_prefix(&compat_icon);
        }

        // Add package count
        let count_label = gtk::Label::builder()
            .label(&format!("{} packages", bundle.packages.len()))
            .css_classes(["dim-label"])
            .build();
        row.add_suffix(&count_label);

        // Add switch (disabled for ARM-incompatible bundles on ARM)
        let switch = gtk::Switch::builder()
            .valign(gtk::Align::Center)
            .sensitive(!(is_arm && bundle.arm_compat == ArmCompat::None))
            .build();
        row.add_suffix(&switch);

        // Store bundle ID and connect toggle
        let bundle_id = bundle.id.clone();
        let packages = bundle.packages.clone();

        switch.connect_state_set(glib::clone!(@weak self as page => @default-return glib::Propagation::Proceed, move |_, state| {
            page.toggle_bundle(&bundle_id, state, &packages);
            glib::Propagation::Proceed
        }));

        (row, switch)
    }

    fn toggle_bundle(&self, bundle_id: &str, enabled: bool, packages: &[String]) {
        let imp = self.imp();

        // Update internal state
        if enabled {
            imp.enabled_bundles.borrow_mut().insert(bundle_id.to_string());
        } else {
            imp.enabled_bundles.borrow_mut().remove(bundle_id);
        }

        // Update main window state
        if let Some(window) = self.root().and_then(|r| r.downcast::<crate::window::MainWindow>().ok()) {
            window.update_app_state(|state| {
                if enabled {
                    state.enable_bundle(bundle_id);
                } else {
                    state.disable_bundle(bundle_id);
                }
            });
        }

        tracing::info!(
            "Bundle {} {} (packages: {})",
            bundle_id,
            if enabled { "enabled" } else { "disabled" },
            packages.join(", ")
        );
    }

    pub fn get_enabled_bundles(&self) -> HashSet<String> {
        self.imp().enabled_bundles.borrow().clone()
    }
}

impl Default for BundlesPage {
    fn default() -> Self {
        Self::new()
    }
}
