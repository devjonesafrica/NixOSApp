//! System settings page (hostname, etc.)

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SystemPage {
        pub hostname_entry: RefCell<Option<adw::EntryRow>>,
        pub current_hostname: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SystemPage {
        const NAME: &'static str = "NixosToolkitSystemPage";
        type Type = super::SystemPage;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for SystemPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for SystemPage {}
    impl BoxImpl for SystemPage {}
}

glib::wrapper! {
    pub struct SystemPage(ObjectSubclass<imp::SystemPage>)
        @extends gtk::Box, gtk::Widget;
}

impl SystemPage {
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

        // Title
        let title = gtk::Label::builder()
            .label("System Settings")
            .css_classes(["title-1"])
            .halign(gtk::Align::Start)
            .build();
        self.append(&title);

        // Description
        let desc = gtk::Label::builder()
            .label("Configure system-level settings. These will be applied through NixOS configuration.")
            .wrap(true)
            .halign(gtk::Align::Start)
            .css_classes(["dim-label"])
            .build();
        self.append(&desc);

        // Hostname group
        let hostname_group = adw::PreferencesGroup::builder()
            .title("Network Identity")
            .build();

        // Get current hostname
        let current_hostname = std::fs::read_to_string("/etc/hostname")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "nixos".to_string());

        *imp.current_hostname.borrow_mut() = Some(current_hostname.clone());

        let hostname_entry = adw::EntryRow::builder()
            .title("Hostname")
            .text(&current_hostname)
            .show_apply_button(true)
            .build();

        // Add icon
        hostname_entry.add_prefix(&gtk::Image::from_icon_name("computer-symbolic"));

        // Connect apply button
        hostname_entry.connect_apply(glib::clone!(@weak self as page => move |entry| {
            let new_hostname = entry.text().to_string();
            page.set_hostname(&new_hostname);
        }));

        hostname_group.add(&hostname_entry);
        *imp.hostname_entry.borrow_mut() = Some(hostname_entry);

        self.append(&hostname_group);

        // Info banner about hostname changes
        let info_row = adw::ActionRow::builder()
            .title("Note")
            .subtitle("Hostname changes require a system rebuild and may require a reboot to take full effect.")
            .build();
        info_row.add_prefix(&gtk::Image::from_icon_name("dialog-information-symbolic"));
        hostname_group.add(&info_row);

        // Future settings placeholder
        let future_group = adw::PreferencesGroup::builder()
            .title("More Settings")
            .description("Additional system settings coming in future updates")
            .build();

        let coming_soon = adw::ActionRow::builder()
            .title("Timezone")
            .subtitle("Coming soon")
            .sensitive(false)
            .build();
        coming_soon.add_prefix(&gtk::Image::from_icon_name("preferences-system-time-symbolic"));
        future_group.add(&coming_soon);

        let locale_row = adw::ActionRow::builder()
            .title("Locale")
            .subtitle("Coming soon")
            .sensitive(false)
            .build();
        locale_row.add_prefix(&gtk::Image::from_icon_name("preferences-desktop-locale-symbolic"));
        future_group.add(&locale_row);

        self.append(&future_group);
    }

    fn set_hostname(&self, hostname: &str) {
        let imp = self.imp();

        // Validate hostname
        if hostname.is_empty() {
            tracing::warn!("Empty hostname not allowed");
            return;
        }

        if !hostname.chars().all(|c| c.is_alphanumeric() || c == '-') {
            tracing::warn!("Invalid hostname characters");
            return;
        }

        // Check if actually changed
        let current = imp.current_hostname.borrow();
        if current.as_ref() == Some(&hostname.to_string()) {
            tracing::info!("Hostname unchanged");
            return;
        }

        // Update main window state
        if let Some(window) = self.root().and_then(|r| r.downcast::<crate::window::MainWindow>().ok()) {
            window.update_app_state(|state| {
                state.set_hostname(hostname);
            });
        }

        tracing::info!("Hostname will be changed to: {}", hostname);
    }

    pub fn get_hostname(&self) -> Option<String> {
        self.imp()
            .hostname_entry
            .borrow()
            .as_ref()
            .map(|e| e.text().to_string())
    }
}

impl Default for SystemPage {
    fn default() -> Self {
        Self::new()
    }
}
