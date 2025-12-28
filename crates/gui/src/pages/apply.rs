//! Apply changes page with preview and log viewer

use adw::prelude::*;
use adw::subclass::prelude::*;
use common::actions::{default_bundles, default_profiles};
use common::nix::generate_preview;
use gtk::glib;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ApplyPage {
        pub preview_view: RefCell<Option<gtk::TextView>>,
        pub log_view: RefCell<Option<gtk::TextView>>,
        pub apply_button: RefCell<Option<gtk::Button>>,
        pub is_applying: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ApplyPage {
        const NAME: &'static str = "NixosToolkitApplyPage";
        type Type = super::ApplyPage;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for ApplyPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for ApplyPage {}
    impl BoxImpl for ApplyPage {}
}

glib::wrapper! {
    pub struct ApplyPage(ObjectSubclass<imp::ApplyPage>)
        @extends gtk::Box, gtk::Widget;
}

impl ApplyPage {
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
            .label("Apply Changes")
            .css_classes(["title-1"])
            .halign(gtk::Align::Start)
            .build();
        self.append(&title);

        // Description
        let desc = gtk::Label::builder()
            .label("Review your configuration and apply changes to the system.")
            .wrap(true)
            .halign(gtk::Align::Start)
            .css_classes(["dim-label"])
            .build();
        self.append(&desc);

        // Preview section
        let preview_group = adw::PreferencesGroup::builder()
            .title("Configuration Preview")
            .description("Nix files that will be written")
            .build();

        let preview_scroll = gtk::ScrolledWindow::builder()
            .height_request(250)
            .build();

        let preview_view = gtk::TextView::builder()
            .editable(false)
            .monospace(true)
            .left_margin(12)
            .right_margin(12)
            .top_margin(12)
            .bottom_margin(12)
            .wrap_mode(gtk::WrapMode::Word)
            .build();
        preview_view.add_css_class("card");
        preview_view.buffer().set_text("# No changes to preview\n# Select a profile or bundles to see the configuration");
        preview_scroll.set_child(Some(&preview_view));
        *imp.preview_view.borrow_mut() = Some(preview_view);

        preview_group.add(&preview_scroll);
        self.append(&preview_group);

        // Refresh preview button
        let refresh_button = gtk::Button::builder()
            .label("Refresh Preview")
            .halign(gtk::Align::Start)
            .build();
        refresh_button.connect_clicked(glib::clone!(@weak self as page => move |_| {
            page.refresh_preview();
        }));
        self.append(&refresh_button);

        // Action buttons
        let button_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .halign(gtk::Align::Start)
            .margin_top(12)
            .build();

        let apply_button = gtk::Button::builder()
            .label("Apply Changes")
            .css_classes(["suggested-action", "pill"])
            .build();
        apply_button.connect_clicked(glib::clone!(@weak self as page => move |_| {
            page.show_apply_dialog();
        }));
        *imp.apply_button.borrow_mut() = Some(apply_button.clone());

        let dry_run_button = gtk::Button::builder()
            .label("Dry Run")
            .css_classes(["pill"])
            .tooltip_text("Build configuration without activating")
            .build();
        dry_run_button.connect_clicked(glib::clone!(@weak self as page => move |_| {
            page.do_dry_run();
        }));

        button_box.append(&apply_button);
        button_box.append(&dry_run_button);
        self.append(&button_box);

        // Log section
        let log_group = adw::PreferencesGroup::builder()
            .title("Build Log")
            .description("Output from nixos-rebuild")
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
        log_view.buffer().set_text("# Build log will appear here when you apply changes");
        log_scroll.set_child(Some(&log_view));
        *imp.log_view.borrow_mut() = Some(log_view);

        log_group.add(&log_scroll);
        self.append(&log_group);
    }

    fn refresh_preview(&self) {
        let imp = self.imp();

        // Get current state from main window
        if let Some(window) = self.root().and_then(|r| r.downcast::<crate::window::MainWindow>().ok()) {
            let state = window.get_app_state();
            let profiles = default_profiles();
            let bundles = default_bundles();

            // Find selected profile
            let profile = state
                .selected_profile
                .as_ref()
                .and_then(|id| profiles.iter().find(|p| &p.id == id));

            // Find enabled bundles
            let enabled_bundles: Vec<_> = bundles
                .iter()
                .filter(|b| state.enabled_bundles.contains(&b.id))
                .collect();

            // Generate preview
            let preview = generate_preview(profile, &enabled_bundles, state.hostname.as_deref());

            if let Some(ref view) = *imp.preview_view.borrow() {
                view.buffer().set_text(&preview);
            }
        }
    }

    fn show_apply_dialog(&self) {
        if let Some(window) = self.root().and_then(|r| r.downcast::<adw::ApplicationWindow>().ok()) {
            let dialog = adw::MessageDialog::builder()
                .transient_for(&window)
                .modal(true)
                .heading("Apply Configuration?")
                .body("This will run 'nixos-rebuild switch' with your selected configuration.\n\nMake sure you have reviewed the preview above.")
                .build();

            dialog.add_responses(&[
                ("cancel", "Cancel"),
                ("apply", "Apply"),
            ]);
            dialog.set_response_appearance("apply", adw::ResponseAppearance::Suggested);
            dialog.set_default_response(Some("cancel"));
            dialog.set_close_response("cancel");

            dialog.connect_response(
                None,
                glib::clone!(@weak self as page => move |_, response| {
                    if response == "apply" {
                        page.do_apply();
                    }
                }),
            );

            dialog.present();
        }
    }

    fn do_apply(&self) {
        let imp = self.imp();

        if *imp.is_applying.borrow() {
            tracing::warn!("Already applying, ignoring request");
            return;
        }

        *imp.is_applying.borrow_mut() = true;

        // Disable apply button
        if let Some(ref button) = *imp.apply_button.borrow() {
            button.set_sensitive(false);
            button.set_label("Applying...");
        }

        // Clear log
        if let Some(ref view) = *imp.log_view.borrow() {
            view.buffer().set_text("Starting nixos-rebuild switch...\n\n");
        }

        // TODO: Implement actual helper invocation
        // For now, just show a message
        self.append_log("Note: Full apply functionality requires the helper binary.\n");
        self.append_log("To apply changes manually:\n");
        self.append_log("1. Review the generated files in /etc/nixos/nixos-toolkit/\n");
        self.append_log("2. Run: sudo nixos-rebuild switch\n");

        // Re-enable button after simulated delay
        glib::timeout_add_local_once(
            std::time::Duration::from_secs(2),
            glib::clone!(@weak self as page => move || {
                let imp = page.imp();
                *imp.is_applying.borrow_mut() = false;
                if let Some(ref button) = *imp.apply_button.borrow() {
                    button.set_sensitive(true);
                    button.set_label("Apply Changes");
                }
                page.append_log("\nReady.\n");
            }),
        );
    }

    fn do_dry_run(&self) {
        self.append_log("\n--- Dry Run ---\n");
        self.append_log("Would run: nixos-rebuild dry-build\n");
        self.refresh_preview();
        self.append_log("Preview updated. No changes applied.\n");
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

impl Default for ApplyPage {
    fn default() -> Self {
        Self::new()
    }
}
