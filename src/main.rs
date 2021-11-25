pub mod main_window;
pub mod model;

use main_window::MainWindow;
use gtk::prelude::*;
use gtk::Application;
use std::error::Error;
use std::sync::Arc;

use btleplug::api::{Central, CentralEvent, Manager as _};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use tokio::task;
use uuid::Uuid;
use regex::Regex;
use model::BeaconInfo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;
    let shared_central = Arc::new(get_central(&manager).await);

    let app = Application::builder()
        .application_id("site.riddleling.sub_win")
        .build();

    app.connect_activate(move |app| {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let central_clone1 = Arc::clone(&shared_central);
        task::spawn(async move {
            let _ = central_event_loop_up(central_clone1, tx).await;
        });

        let central_clone2 = Arc::clone(&shared_central);
        build_ui(&app, central_clone2, rx);
    });
    app.run();
    Ok(())
}

fn build_ui(app: &Application, central: Arc<Adapter>, rx: glib::Receiver<BeaconInfo>) {
    let win = MainWindow::new(app);
    win.set_title("Beacon Searcher");
    win.set_central(central);
    win.set_rx(rx);
    win.show_all();
}

///
/// - BLE Central
///

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

async fn central_event_loop_up(central: Arc<Adapter>, sender: glib::Sender<BeaconInfo>) -> Result<(), Box<dyn Error>> {
    let mut events = central.events().await?;
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::ManufacturerDataAdvertisement {
                id,
                manufacturer_data,
            } => {
                // eprintln!("ID: {:?} , ManufacturerData: {:?}", id, manufacturer_data);
                let apple_company_id = 0x004C;
                if let Some(data) = manufacturer_data.get(&apple_company_id) {
                    if data[0] == 0x02 && data[1] == 0x15 && data.len() == 23 {
                        let id_string = capture_id(format!("{:?}", id));
                        eprintln!("id_string: {:?}", id_string);

                        let mut uuid_string = "".to_string();
                        if let Ok(uuid) = Uuid::from_fields(
                            u32::from_be_bytes([data[2], data[3], data[4], data[5]]),
                            u16::from_be_bytes([data[6], data[7]]),
                            u16::from_be_bytes([data[8], data[9]]),
                            &data[10 .. 18]
                        ) {
                            uuid_string = uuid.to_string();
                        }

                        eprintln!("UUID: {:?}", uuid_string);

                        let major = u16::from_be_bytes([data[18], data[19]]);
                        let minor = u16::from_be_bytes([data[20], data[21]]);
                        let tx_power = data[22] as i8;
                        eprintln!("Major: {:?} , Minor: {:?} , TxPower: {:?}", major, minor, tx_power);
                        
                        let beacon = BeaconInfo {
                            peripheral_id: id_string,
                            proximity_uuid: uuid_string,
                            major: major as u32,
                            minor: minor as u32,
                            tx_power: tx_power
                        };

                        match sender.send(beacon) {
                            Ok(_beacon) => eprintln!("> send beacon value"),
                            Err(error) => eprintln!("> send error: {:?}", error)
                        }
                        eprintln!("-------------------------");
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn capture_id(text: String) -> String {
    let re = Regex::new(r"PeripheralId\((?P<id>.+?)\)").unwrap();
    match re.captures(&text) {
        Some(caps) => {
            let capture_id = &caps["id"];
            capture_id.to_string()
        }
        None => "".to_string()
    }
}
