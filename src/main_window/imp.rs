use glib::clone;
use gtk::{glib, gdk};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use std::{cell::Cell, rc::Rc, sync::Arc, cell::RefCell};
use once_cell::unsync::OnceCell;

use btleplug::platform::Adapter;
use btleplug::api::{Central, ScanFilter};

use crate::model::{create_model, set_tree_view_titles, get_row_data_string};

#[derive(Debug, Default)]
pub struct MainWindow {
    pub central: OnceCell<Arc<Adapter>>,
    pub beacon_ids: Rc<RefCell<Vec<String>>>,
    pub model: OnceCell<gtk::ListStore>,
    is_searching: Cell<bool>,
    tree_view: OnceCell<gtk::TreeView>,
    label: OnceCell<gtk::Label>,
    spinner: OnceCell<gtk::Spinner>,
    search_button: OnceCell<gtk::Button>,
}

#[glib::object_subclass]
impl ObjectSubclass for MainWindow {
    const NAME: &'static str = "MainWindow";
    type Type = super::MainWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for MainWindow {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        // header_bar
        let header_bar = gtk::HeaderBar::new();
        header_bar.set_title(Some("Main Window"));
        header_bar.set_show_close_button(true);

        // gtk_box
        let gtk_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .homogeneous(false)
            .spacing(5)
            .build();

        // tree_view
        let model = create_model();
        let tree_view = gtk::TreeView::builder()
            .model(&model)
            .activate_on_single_click(true)
            .build();

        set_tree_view_titles(&tree_view);
        tree_view.connect_row_activated(clone!(@weak obj => move |_, path, _column| {
            let priv_ = MainWindow::from_instance(&obj);
            priv_.on_row_selected(path);
        }));
        
        // scrolled_window
        let scrolled_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&tree_view)
            .build();

        // action_bor
        let action_bor = gtk::ActionBar::new();

        // label
        let label = gtk::Label::builder()
            .margin_start(10)
            .margin_end(5)
            .build();

        // spinner
        let spinner = gtk::Spinner::builder()
            .margin_start(5)
            .margin_end(5)
            .build();

        // search_button
        let search_button = gtk::Button::with_label("Search");
        search_button.set_size_request(80, 25);
        search_button.connect_clicked(clone!(@weak obj => move |_| {
            let priv_ = MainWindow::from_instance(&obj);
            priv_.on_search_button_clicked();
        }));


        action_bor.pack_start(&label);
        action_bor.pack_end(&search_button);
        action_bor.pack_end(&spinner);

        gtk_box.pack_start(&scrolled_window, true, true, 0);
        gtk_box.pack_start(&action_bor, false, false, 0);

        obj.set_titlebar(Some(&header_bar));
        obj.add(&gtk_box);
        obj.set_default_size(700, 400);

        self.is_searching.set(false);
        self.model.set(model).expect("Failed to initialize window state: model");
        self.tree_view.set(tree_view).expect("Failed to initialize window state: tree_view");
        self.label.set(label).expect("Failed to initialize window state: label");
        self.spinner.set(spinner).expect("Failed to initialize window state: spinner");
        self.search_button.set(search_button).expect("Failed to initialize window state: search_button");
    }   
}

impl MainWindow {
    fn on_row_selected(&self, path: &gtk::TreePath) {
        let model = self.model.get().unwrap();
        let string = get_row_data_string(model, path);
        // eprintln!("row: {}", string);

        let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
        clipboard.set_text(&string);

        let label = self.label.get().unwrap();
        if label.text() == "Copied!" { return; }

        label.set_text("Copied!");
        glib::timeout_add_seconds_local(
            3,  
            clone!(@weak label => @default-return glib::Continue(false), 
                move || {
                    label.set_text("");
                    glib::Continue(false)
                }
            )
        );
    }

    fn on_search_button_clicked(&self) {
        let central = self.central.get().unwrap();
        let model = self.model.get().unwrap();
        let beacon_ids = &self.beacon_ids;
        let spinner = self.spinner.get().unwrap();
        let search_button = self.search_button.get().unwrap();
        search_button.set_sensitive(false);

        self.is_searching.set(!self.is_searching.get());
        let is_searching = self.is_searching.get();

        glib::MainContext::default().spawn_local(
            clone!(@weak central, @weak model, @weak spinner, @weak search_button, @strong beacon_ids => 
                async move {
                    if is_searching {
                        beacon_ids.borrow_mut().clear();
                        model.clear();
                        let success = start_search(&central).await;
                        if success {
                            search_button.set_label("Stop");
                            spinner.start();
                        }
                    } else {
                        let success = stop_search(&central).await;
                        if success {
                            search_button.set_label("Search");
                            spinner.stop();
                        }
                    }
                    search_button.set_sensitive(true);
                }
            )
        );
    }
}

impl WidgetImpl for MainWindow {}
impl ContainerImpl for MainWindow {}
impl BinImpl for MainWindow {}
impl WindowImpl for MainWindow {}
impl ApplicationWindowImpl for MainWindow {}


async fn start_search(central: &Adapter) -> bool {
    eprintln!("\n> start_search\n");
    match central.start_scan(ScanFilter::default()).await {
        Ok(_) => true,
        Err(_) => false
    }
}

async fn stop_search(central: &Adapter) -> bool {
    eprintln!("\n> stop_search\n");
    match central.stop_scan().await {
        Ok(_) => true,
        Err(_) => false
    }
}
