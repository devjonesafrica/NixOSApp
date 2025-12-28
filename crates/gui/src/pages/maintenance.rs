//! Maintenance page for garbage collection and system cleanup

use adw::prelude::*;
use adw::subclass::prelude::*;
use common::actions::{default_maintenance_actions, MaintenanceActionDef};
use gtk::glib;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MaintenancePage {
        pub log_view: RefCell<Option<gtk::TextView>>,
        pub is_running: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MaintenancePage {
        const NAME: &'static str = "NixosToolkitMaintenancePage";
        type Type = super::MaintenancePage;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for MaintenancePage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for MaintenancePage {}
    impl BoxImpl for MaintenancePage {}
}

glib::wrapper! {
    pub struct MaintenancePage(ObjectSubclass<imp::MaintenancePage>)
        @extends gtk::Box, gtk::Widget;
}

impl MaintenancePage {
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
            .label("System Maintenance")
            .css_classes(["title-1"])
            .halign(gtk::Align::Start)
            .build();
        self.append(&title);

        // Description
        let desc = gtk::Label::builder()
            .label("Clean up disk space and maintain your NixOS system.")
            .wrap(true)
            .halign(gtk::Align::Start)
            .css_classes(["dim-label"])
            .build();
        self.append(&desc);

        // Actions group
        let actions_group = adw::PreferencesGroup::builder()
            .title("Maintenance Actions")
            .build();

        // Add action rows
        let actions = default_maintenance_actions();
        for action in &actions {
            let row = self.create_action_row(action);
            actions_group.add(&row);
        }

        self.append(&actions_group);

        // Disk usage info
        let disk_group = adw::PreferencesGroup::builder()
            .title("Disk Usage")
            .build();

        let store_size_row = adw::ActionRow::builder()
            .title("Nix Store Size")
            .subtitle("Calculating...")
            .build();
        store_size_row.add_prefix(&gtk::Image::from_icon_name("drive-harddisk-symbolic"));
        disk_group.add(&store_size_row);

        let generations_row = adw::ActionRow::builder()
            .title("System Generations")
            .subtitle("Calculating...")
            .build();
        generations_row.add_prefix(&gtk::Image::from_icon_name("document-open-recent-symbolic"));
        disk_group.add(&generations_row);

        self.append(&disk_group);

        // Refresh button
        let refresh_button = gtk::Button::builder()
            .label("Refresh Disk Info")
            .halign(gtk::Align::Start)
            .build();
        refresh_button.connect_clicked(glib::clone!(@weak store_size_row, @weak generations_row => move |_| {
            // Update store size (async in real implementation)
            store_size_row.set_subtitle("Run 'du -sh /nix/store' to check");
            generations_row.set_subtitle("Run 'nix-env --list-generations -p /nix/var/nix/profiles/system' to check");
        }));
        self.append(&refresh_button);

        // Log section
        let log_group = adw::PreferencesGroup::builder()
            .title("Output Log")
            .build();

        let log_scroll = gtk::ScrolledWindow::builder()
            .height_request(200)
            .build();

        let log_view = gtk::TextView::builder()
            .editable(false)
            .monospace(true)
            .left_margin(12)
            .right_margin(12)
            .top_margin(12)
            .bottom_margin(12)
            .wrap_mode(gtk::WrapMode::Word)
            .build();
        log_view.add_css_class("card");
        log_view.buffer().set_text("# Maintenance log will appear here\n# Select an action above to run it");
        log_scroll.set_child(Some(&log_view));
        *imp.log_view.borrow_mut() = Some(log_view);

        log_group.add(&log_scroll);
        self.append(&log_group);
    }

    fn create_action_row(&self, action: &MaintenanceActionDef) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(&action.name)
            .subtitle(&action.description)
            .activatable(true)
            .build();

        // Add icon
        row.add_prefix(&gtk::Image::from_icon_name(&action.icon));

        // Add run button
        let run_button = gtk::Button::builder()
            .icon_name("media-playback-start-symbolic")
            .valign(gtk::Align::Center)
            .css_classes(["flat"])
            .tooltip_text("Run this action")
            .build();

        let action_id = action.id.clone();
        let action_name = action.name.clone();
        let action_command = action.command.clone();
        let action_warning = action.warning.clone();

        run_button.connect_clicked(glib::clone!(@weak self as page => move |_| {
            page.run_action(&action_id, &action_name, &action_command, action_warning.as_deref());
        }));

        row.add_suffix(&run_button);
        row
    }

    fn run_action(&self, id: &str, name: &str, command: &str, warning: Option<&str>) {
        let imp = self.imp();

        if *imp.is_running.borrow() {
            self.append_log("Another action is already running. Please wait.\n");
            return;
        }

        // Show confirmation for dangerous actions
        if let Some(warning_msg) = warning {
            if let Some(window) = self.root().and_then(|r| r.downcast::<adw::ApplicationWindow>().ok()) {
                let dialog = adw::AlertDialog::builder()
                    .heading(&format!("Run {}?", name))
                    .body(warning_msg)
                    .build();

                dialog.add_responses(&[
                    ("cancel", "Cancel"),
                    ("confirm", "Run Anyway"),
                ]);
                dialog.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                let command = command.to_string();
                let name = name.to_string();
                dialog.connect_response(
                    None,
                    glib::clone!(@weak self as page => move |_, response| {
                        if response == "confirm" {
                            page.execute_command(&name, &command);
                        }
                    }),
                );

                dialog.present(Some(&window));
                return;
            }
        }

        self.execute_command(name, command);
    }

    fn execute_command(&self, name: &str, command: &str) {
        let imp = self.imp();
        *imp.is_running.borrow_mut() = true;

        self.append_log(&format!("\n--- Running: {} ---\n", name));
        self.append_log(&format!("$ sudo {}\n\n", command));

        // Note: In actual implementation, this would spawn the helper process
        // For now, show what would be run
        self.append_log("Note: This action requires root privileges.\n");
        self.append_log(&format!("To run manually: sudo {}\n", command));
        self.append_log("\nThe helper binary will execute this when fully integrated.\n");

        *imp.is_running.borrow_mut() = false;
    }

    fn append_log(&self, text: &str) {
        if let Some(ref view) = *self.imp().log_view.borrow() {
            let buffer = view.buffer();
            let mut end = buffer.end_iter();
            buffer.insert(&mut end, text);

            // Scroll to end
            let mark = buffer.create_mark(None, &buffer.end_iter(), false);
            view.scroll_to_mark(&mark, 0.0, true, 0.0, 1.0);
        }
    }
}

impl Default for MaintenancePage {
    fn default() -> Self {
        Self::new()
    }
}
