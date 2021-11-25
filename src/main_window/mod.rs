mod imp;

use glib::clone;
use gtk::glib;
use gtk::subclass::prelude::*;
use btleplug::platform::Adapter;
use std::sync::Arc;

use crate::model::BeaconInfo;
use crate::model::add_item;

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window, gtk::ApplicationWindow,
        @implements gtk::Buildable;
                    
}

impl MainWindow {
    pub fn new(app: &gtk::Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create MainWindow")
    }

    pub fn set_central(&self, central: Arc<Adapter>) {
        let central_ = &imp::MainWindow::from_instance(self).central;
        central_.set(central).expect("Failed to set central" );
    }

    pub fn set_rx(&self, rx: glib::Receiver<BeaconInfo>) {
        rx.attach(None,  clone!(@weak self as this => @default-return glib::Continue(false),
            move |beacon| {
                let model = imp::MainWindow::from_instance(&this).model.get().unwrap();
                let beacon_ids = &*imp::MainWindow::from_instance(&this).beacon_ids;
                add_item(&model, beacon_ids, beacon);
                glib::Continue(true)
            }
        ));
    }
}
