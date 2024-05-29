use log::info;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::adapter::FlexispotMqttAdapter;
use crate::flexispot::command::FlexispotPreset;

#[derive(Debug)]
pub struct MqttController<A: FlexispotMqttAdapter + Send + Sync + 'static> {
    mqtt_options: MqttOptions,
    adapter: Arc<Mutex<A>>,
}

impl<A: FlexispotMqttAdapter + Send + Sync + 'static> MqttController<A> {
    pub fn new(mqtt_options: MqttOptions, adapter: Arc<Mutex<A>>) -> Self {
        Self {
            mqtt_options,
            adapter,
        }
    }

    pub fn run(self) -> Result<(), String> {
        let (client, mut connection) = Client::new(
            self.mqtt_options,
            10, // cap = 1 では処理が中断された
        );
        info!("init client");
        for p in [
            "desk-switch/profile1/set",
            "desk-switch/profile2/set",
            "desk-switch/profile3/set",
            "desk-switch/profile4/set",
        ] {
            client.subscribe(p, QoS::AtMostOnce).unwrap();
            info!("subscribe {p}");
        }
        info!("subscribed");

        let adapter = Arc::clone(&self.adapter);
        thread::spawn(move || {
            let mut i = 0;
            loop {
                i += 1;
                client
                    .publish("online", QoS::AtLeastOnce, false, "true")
                    .unwrap();
                let h = (i * 10 % 100).to_string();
                client
                    .publish("desk/current-height", QoS::AtMostOnce, false, h.clone())
                    .unwrap();
                client
                    .publish("desk/target-height", QoS::AtMostOnce, false, h.clone())
                    .unwrap();
                client
                    .publish("desk/height-state", QoS::AtMostOnce, false, "STOPPED")
                    .unwrap();

                let height = adapter.lock().unwrap().get_desk_height();
                info!(
                    "current_height={}",
                    height.map(|h| h.to_string()).unwrap_or("None".into())
                );
                thread::sleep(Duration::from_secs(1));
                i %= 100;

                client
                    .publish("desk-switch/profile1/get", QoS::AtLeastOnce, false, "false")
                    .unwrap();
            }
        });

        // Iterate to poll the eventloop for connection progress
        for notification in connection.iter() {
            info!("Notification = {:?}", notification);
            let Ok(notification) = notification else {
                continue;
            };
            if let Event::Incoming(Packet::Publish(publish)) = notification {
                let topic = publish.topic;
                let payload = publish
                    .payload
                    .iter()
                    .map(|&b| b as char)
                    .collect::<String>();
                info!("{topic} {payload}");

                match (topic.as_str(), payload.as_str()) {
                    ("desk-switch/profile1/set", "true") => {
                        info!("[begin] desk-switch profile set 1");
                        let adapter = self.adapter.lock().unwrap();
                        adapter.wakeup();
                        adapter.friendly_sleep();
                        adapter.call_preset(&FlexispotPreset::Preset1);
                        info!("[ end ] desk-switch profile set 1");
                    }
                    ("desk-switch/profile2/set", "true") => {
                        let adapter = self.adapter.lock().unwrap();
                        info!("[begin] desk-switch profile set 2");
                        adapter.wakeup();
                        adapter.friendly_sleep();
                        adapter.call_preset(&FlexispotPreset::Preset2);
                        info!("[ end ] desk-switch profile set 2");
                    }
                    ("desk-switch/profile3/set", "true") => {
                        info!("[begin] desk-switch profile set 3");
                        let adapter = self.adapter.lock().unwrap();
                        adapter.wakeup();
                        adapter.friendly_sleep();
                        adapter.call_preset(&FlexispotPreset::Preset3);
                        info!("[ end ] desk-switch profile set 3");
                    }
                    ("desk-switch/profile4/set", "true") => {
                        info!("[begin] desk-switch profile set 4");
                        let adapter = self.adapter.lock().unwrap();
                        adapter.wakeup();
                        adapter.friendly_sleep();
                        adapter.call_preset(&FlexispotPreset::Preset4);
                        info!("[ end ] desk-switch profile set 4");
                    }
                    (_, _) => {}
                }
            }
        }
        Ok(())
    }
}
