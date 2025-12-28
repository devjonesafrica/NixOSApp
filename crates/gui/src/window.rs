//! Main application window with sidebar navigation

use crate::integration::detect_system;
use crate::pages::{ApplyPage, BundlesPage, MaintenancePage, OnboardingPage, ProfilesPage, SystemPage};
use crate::state::AppState;
use adw::prelude::*;
use adw::subclass::prelude::*;
use common::{IntegrationStatus, SystemInfo};
use gtk::{gio, glib};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct MainWindow {
        pub split_view: adw::NavigationSplitView,
        pub sidebar_list: gtk::ListBox,
        pub content_stack: gtk::Stack,
        pub status_banner: adw::Banner,
        pub app_state: RefCell<AppState>,
        pub system_info: RefCell<SystemInfo>,
    }

    impl Default for MainWindow {
        fn default() -> Self {
            Self {
                split_view: adw::NavigationSplitView::new(),
                sidebar_list: gtk::ListBox::new(),
                content_stack: gtk::Stack::new(),
                status_banner: adw::Banner::new(""),
                app_state: RefCell::new(AppState::default()),
                system_info: RefCell::new(SystemInfo::default()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "NixosToolkitMainWindow";
        type Type = super::MainWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for MainWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
            self.obj().setup_actions();
            self.obj().detect_and_update();
        }
    }

    impl WidgetImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
    impl ApplicationWindowImpl for MainWindow {}
    impl AdwApplicationWindowImpl for MainWindow {}
}

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl MainWindow {
    pub fn new(app: &crate::app::NixosToolkitApp) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", app)
            .property("default-width", 1000)
            .property("default-height", 700)
            .property("title", "NixOS Toolkit")
            .build();
        window
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        // Setup sidebar
        imp.sidebar_list.add_css_class("navigation-sidebar");
        imp.sidebar_list.set_selection_mode(gtk::SelectionMode::Single);

        // Navigation items
        let nav_items = [
            ("onboarding", "Getting Started", "go-home-symbolic"),
            ("profiles", "Desktop Profiles", "user-desktop-symbolic"),
            ("bundles", "Software Bundles", "package-x-generic-symbolic"),
            ("system", "System Settings", "preferences-system-symbolic"),
            ("maintenance", "Maintenance", "user-trash-symbolic"),
            ("apply", "Apply Changes", "emblem-synchronizing-symbolic"),
        ];

        for (id, label, icon) in nav_items {
            let row = adw::ActionRow::builder()
                .title(label)
                .activatable(true)
                .build();
            row.add_prefix(&gtk::Image::from_icon_name(icon));
            row.set_widget_name(id);
            imp.sidebar_list.append(&row);
        }

        // Connect sidebar selection
        imp.sidebar_list.connect_row_selected(
            glib::clone!(@weak self as window => move |_, row| {
                if let Some(row) = row {
                    let name = row.widget_name();
                    window.imp().content_stack.set_visible_child_name(&name);
                    window.imp().split_view.set_show_content(true);
                }
            }),
        );

        // Setup content stack
        imp.content_stack.set_transition_type(gtk::StackTransitionType::Crossfade);

        // Add pages
        let onboarding = OnboardingPage::new();
        let profiles = ProfilesPage::new();
        let bundles = BundlesPage::new();
        let system = SystemPage::new();
        let maintenance = MaintenancePage::new();
        let apply = ApplyPage::new();

        imp.content_stack.add_named(&onboarding, Some("onboarding"));
        imp.content_stack.add_named(&profiles, Some("profiles"));
        imp.content_stack.add_named(&bundles, Some("bundles"));
        imp.content_stack.add_named(&system, Some("system"));
        imp.content_stack.add_named(&maintenance, Some("maintenance"));
        imp.content_stack.add_named(&apply, Some("apply"));

        // Create sidebar navigation page
        let sidebar_toolbar = adw::ToolbarView::new();
        let sidebar_header = adw::HeaderBar::new();
        sidebar_header.set_show_title(true);
        sidebar_toolbar.add_top_bar(&sidebar_header);

        let sidebar_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&imp.sidebar_list)
            .build();
        sidebar_toolbar.set_content(Some(&sidebar_scroll));

        let sidebar_page = adw::NavigationPage::builder()
            .title("NixOS Toolkit")
            .child(&sidebar_toolbar)
            .build();

        // Create content navigation page
        let content_toolbar = adw::ToolbarView::new();

        // Add banner at top of content
        imp.status_banner.set_revealed(false);
        content_toolbar.add_top_bar(&imp.status_banner);

        let content_header = adw::HeaderBar::new();
        content_toolbar.add_top_bar(&content_header);
        content_toolbar.set_content(Some(&imp.content_stack));

        let content_page = adw::NavigationPage::builder()
            .title("Content")
            .child(&content_toolbar)
            .build();

        // Configure split view
        imp.split_view.set_sidebar(Some(&sidebar_page));
        imp.split_view.set_content(Some(&content_page));
        imp.split_view.set_min_sidebar_width(220.0);
        imp.split_view.set_max_sidebar_width(320.0);

        // Add breakpoint for responsive design
        let breakpoint = adw::Breakpoint::new(adw::BreakpointCondition::new_length(
            adw::BreakpointConditionLengthType::MaxWidth,
            600.0,
            adw::LengthUnit::Sp,
        ));
        breakpoint.add_setter(&imp.split_view, "collapsed", &true.to_value());
        self.add_breakpoint(breakpoint);

        self.set_content(Some(&imp.split_view));

        // Select first row by default
        if let Some(first_row) = imp.sidebar_list.row_at_index(0) {
            imp.sidebar_list.select_row(Some(&first_row));
        }
    }

    fn setup_actions(&self) {
        // Refresh action
        let refresh_action = gio::ActionEntry::builder("refresh")
            .activate(|window: &Self, _, _| {
                window.detect_and_update();
            })
            .build();

        self.add_action_entries([refresh_action]);
    }

    fn detect_and_update(&self) {
        let imp = self.imp();

        // Detect system configuration
        let info = detect_system();
        *imp.system_info.borrow_mut() = info.clone();

        // Update banner based on integration status
        if !info.is_nixos {
            imp.status_banner.set_title("Not running on NixOS");
            imp.status_banner.add_css_class("error");
            imp.status_banner.set_revealed(true);
        } else {
            match info.integration_status {
                IntegrationStatus::Integrated => {
                    imp.status_banner.set_title("Integrated - Ready to apply changes");
                    imp.status_banner.remove_css_class("error");
                    imp.status_banner.remove_css_class("warning");
                    imp.status_banner.add_css_class("success");
                    imp.status_banner.set_revealed(true);
                }
                IntegrationStatus::NotIntegrated => {
                    imp.status_banner.set_title("Setup required - See Getting Started");
                    imp.status_banner.remove_css_class("error");
                    imp.status_banner.remove_css_class("success");
                    imp.status_banner.add_css_class("warning");
                    imp.status_banner.set_revealed(true);
                }
                IntegrationStatus::Unknown => {
                    imp.status_banner.set_revealed(false);
                }
            }
        }

        // Update pages with system info
        if let Some(onboarding) = imp
            .content_stack
            .child_by_name("onboarding")
            .and_then(|w| w.downcast::<OnboardingPage>().ok())
        {
            onboarding.update_system_info(&info);
        }

        tracing::info!(
            "System detection complete: is_nixos={}, mode={:?}, integration={:?}",
            info.is_nixos,
            info.config_mode,
            info.integration_status
        );
    }

    pub fn get_system_info(&self) -> SystemInfo {
        self.imp().system_info.borrow().clone()
    }

    pub fn get_app_state(&self) -> AppState {
        self.imp().app_state.borrow().clone()
    }

    pub fn update_app_state<F>(&self, f: F)
    where
        F: FnOnce(&mut AppState),
    {
        f(&mut self.imp().app_state.borrow_mut());
    }
}
