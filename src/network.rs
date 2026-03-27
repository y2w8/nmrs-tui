use nmrs::{ConnectionError, Device, Network, NetworkManager, WifiSecurity};

#[derive()]
pub struct Manager {
    pub network_manager: NetworkManager,
    pub current_connection: Option<Network>,
    pub devices: Vec<Device>,
    pub enabled: bool,
}

impl Manager {
    pub async fn new() -> anyhow::Result<Self> {
        let network_manager = NetworkManager::new().await?;
        let current_connection = network_manager.current_network().await?;

        // Get the device
        let devices = network_manager.list_devices().await?;

        let enabled = network_manager.wifi_enabled().await.unwrap_or(false);

        Ok(Self {
            network_manager,
            current_connection,
            devices,
            enabled,
        })
    }

    pub async fn get_devices(&mut self) -> anyhow::Result<Vec<Device>> {
        let devices = self.network_manager.list_devices().await?;
        let wireless_devices: Vec<Device> = devices
            .into_iter()
            .filter(|dev| dev.is_wireless())
            .collect();
        if wireless_devices.is_empty() {
            anyhow::bail!("No wireless device found")
        } else {
            Ok(wireless_devices)
        }
    }

    pub async fn get_wifi_scan(&mut self) -> anyhow::Result<Vec<Network>> {
        self.network_manager.scan_networks().await?;
        self.current_connection = self.network_manager.current_network().await?;

        let mut networks = self.network_manager.list_networks().await?;
        networks.sort_by(|a, b| b.strength.cmp(&a.strength));
        Ok(networks)
    }

    pub async fn get_saved_networks(&mut self) -> anyhow::Result<Vec<String>> {
        let saved_networks = self.network_manager.list_saved_connections().await?;
        Ok(saved_networks)
    }

    pub async fn forget(&mut self, ssid: &str) -> anyhow::Result<()> {
        self.network_manager.forget(ssid).await?;
        Ok(())
    }

    pub async fn connect(&mut self, ssid: &str, password: &str) -> anyhow::Result<()> {
        match self
            .network_manager
            .connect(
                ssid,
                WifiSecurity::WpaPsk {
                    psk: password.into(),
                },
            )
            .await
        {
            Ok(_) => info!("Connected successfully"),
            Err(ConnectionError::AuthFailed) => {
                error!("Wrong password!");
            }
            Err(ConnectionError::NotFound) => {
                error!("Network not found or out of range");
            }
            Err(ConnectionError::Timeout) => {
                error!("Connection timed out");
            }
            Err(ConnectionError::DhcpFailed) => {
                error!("Failed to obtain IP address");
            }
            Err(e) => {
                error!("Error: {}", e);
            }
        };
        Ok(())
    }

    pub fn is_connected(&mut self, ssid: &String) -> bool {
        self.current_connection.is_some() && self.current_connection.clone().unwrap().ssid == *ssid
    }
}

// use std::collections::HashMap;
// use zbus::{Connection, Proxy};
//
// #[derive(Clone)]
// pub struct WifiNetwork {
//     pub ssid: String,
//     pub signal: u8,
//     pub security: String,
//     pub active: bool,
// }
//
// const NM_DEVICE_TYPE_WIFI: u32 = 2;
//
// pub async fn get_saved_networks() -> zbus::Result<Vec<String>> {
//     let connection = Connection::system().await?;
//
//     let proxy = Proxy::new(
//         &connection,
//         "org.freedesktop.NetworkManager",
//         "/org/freedesktop/NetworkManager/Settings",
//         "org.freedesktop.NetworkManager.Settings",
//     )
//     .await?;
//
//     let connections_paths: Vec<zbus::zvariant::OwnedObjectPath> =
//         proxy.call("ListConnections", &()).await?;
//
//     let mut network_names = Vec::new();
//
//     for path in connections_paths {
//         let conn_proxy = Proxy::new(
//             &connection,
//             "org.freedesktop.NetworkManager",
//             path,
//             "org.freedesktop.NetworkManager.Settings.Connection",
//         )
//         .await?;
//
//         let settings: HashMap<String, HashMap<String, zbus::zvariant::OwnedValue>> =
//             conn_proxy.call("GetSettings", &()).await?;
//
//         if let Some(connection_map) = settings.get("connection")
//             && let Some(id_value) = connection_map.get("id")
//             && let Ok(id_str) = id_value.downcast_ref::<zbus::zvariant::Str>()
//         {
//             network_names.push(id_str.to_string());
//         }
//     }
//
//     Ok(network_names)
// }
//
// pub async fn get_devices() -> zbus::Result<Vec<String>> {
//     let conn = Connection::system().await?;
//     let proxy = Proxy::new(
//         &conn,
//         "org.freedesktop.NetworkManager",
//         "/org/freedesktop/NetworkManager",
//         "org.freedesktop.NetworkManager",
//     )
//     .await?;
//     let device_paths: Vec<zbus::zvariant::OwnedObjectPath> =
//         proxy.get_property("AllDevices").await?;
//
//     let mut names = Vec::new();
//     for path in device_paths {
//         let dev_proxy = Proxy::new(
//             &conn,
//             "org.freedesktop.NetworkManager",
//             path,
//             "org.freedesktop.NetworkManager.Device",
//         )
//         .await?;
//         let interface: String = dev_proxy.get_property("Interface").await?;
//         names.push(interface);
//     }
//     Ok(names)
// }
//
// pub async fn get_wifi_scan(device_interface: &str) -> zbus::Result<Vec<WifiNetwork>> {
//     let conn = Connection::system().await?;
//     let nm_proxy = Proxy::new(
//         &conn,
//         "org.freedesktop.NetworkManager",
//         "/org/freedesktop/NetworkManager",
//         "org.freedesktop.NetworkManager",
//     )
//     .await?;
//
//     // 1. نجلب مسار الـ Active Connection عشان نعرف وش المشبوك عليه حالياً
//     let active_connections: Vec<zbus::zvariant::OwnedObjectPath> =
//         nm_proxy.get_property("ActiveConnections").await?;
//     let mut active_ssids = Vec::new();
//
//     for a_path in active_connections {
//         let a_proxy = Proxy::new(
//             &conn,
//             "org.freedesktop.NetworkManager",
//             a_path,
//             "org.freedesktop.NetworkManager.Connection.Active",
//         )
//         .await?;
//         // نجلب الـ ID (اسم الشبكة المتصل بها)
//         let id: String = a_proxy.get_property("Id").await?;
//         active_ssids.push(id);
//     }
//
//     // 2. نبحث عن كرت الواي فاي فقط (نفلتر Loopback و Bluetooth)
//     let device_paths: Vec<zbus::zvariant::OwnedObjectPath> =
//         nm_proxy.get_property("AllDevices").await?;
//     let mut target_device_path = None;
//
//     for path in device_paths {
//         let dev_proxy = Proxy::new(
//             &conn,
//             "org.freedesktop.NetworkManager",
//             &path,
//             "org.freedesktop.NetworkManager.Device",
//         )
//         .await?;
//
//         let dtype: u32 = dev_proxy.get_property("DeviceType").await?;
//         let iface: String = dev_proxy.get_property("Interface").await?;
//
//         // شرطين: لازم يكون النوع واي فاي (2) والاسم يطابق wlan0
//         if dtype == NM_DEVICE_TYPE_WIFI && iface == device_interface {
//             target_device_path = Some(path);
//             break;
//         }
//     }
//
//     let mut networks = Vec::new();
//     if let Some(path) = target_device_path {
//         let wifi_proxy = Proxy::new(
//             &conn,
//             "org.freedesktop.NetworkManager",
//             &path,
//             "org.freedesktop.NetworkManager.Device.Wireless",
//         )
//         .await?;
//         let ap_paths: Vec<zbus::zvariant::OwnedObjectPath> =
//             wifi_proxy.call("GetAllAccessPoints", &()).await?;
//
//         for ap_path in ap_paths {
//             let ap_proxy = Proxy::new(
//                 &conn,
//                 "org.freedesktop.NetworkManager",
//                 ap_path,
//                 "org.freedesktop.NetworkManager.AccessPoint",
//             )
//             .await?;
//             let ssid_bytes: Vec<u8> = ap_proxy.get_property("Ssid").await?;
//
//             if let Ok(ssid_name) = String::from_utf8(ssid_bytes)
//                 && !ssid_name.is_empty()
//                 && !networks.iter().any(|n: &WifiNetwork| n.ssid == ssid_name)
//             {
//                 let strength: u8 = ap_proxy.get_property("Strength").await?;
//                 let rsn_flags: u32 = ap_proxy.get_property("RsnFlags").await?;
//
//                 // هل هذه الشبكة هي النشطة حالياً؟
//                 let is_active = active_ssids.contains(&ssid_name);
//
//                 networks.push(WifiNetwork {
//                     ssid: ssid_name,
//                     signal: strength,
//                     security: if rsn_flags > 0 {
//                         "WPA2/3".to_string()
//                     } else {
//                         "Open".to_string()
//                     },
//                     active: is_active,
//                 });
//             }
//         }
//     }
//
//     networks.sort_by(|a, b| b.signal.cmp(&a.signal));
//     Ok(networks)
// }
//
// // pub async fn connect_to_wifi(ssid: &str, password: &str) -> anyhow::Result<()> {
// //     let connection = zbus::Connection::system().await?;
// //
// //     let status = std::process::Command::new("nmcli")
// //         .args(["dev", "wifi", "connect", ssid, "password", password])
// //         .status()?;
// //
// //     if status.success() {
// //         Ok(())
// //     } else {
// //         Err(anyhow::anyhow!("Failed to connect via nmcli"))
// //     }
// // }
//
// use zbus::zvariant::{Dict, OwnedObjectPath, OwnedValue, Value};
//
// pub async fn connect_to_wifi(ssid: &str, password: &str) -> anyhow::Result<()> {
//     let connection = zbus::Connection::system().await?;
//
//     // نستخدم المسار الرئيسي للمدير
//     let proxy = zbus::Proxy::new(
//         &connection,
//         "org.freedesktop.NetworkManager",
//         "/org/freedesktop/NetworkManager", // المسار الرئيسي
//         "org.freedesktop.NetworkManager",    // الواجهة الرئيسية
//     ).await?;
//
//     // بناء الإعدادات (نفس كودك السابق)
//     let mut conn_dict = HashMap::new();
//     conn_dict.insert("type".to_string(), Value::from("802-11-wireless"));
//     conn_dict.insert("id".to_string(), Value::from(ssid));
//
//     let mut wifi_dict = HashMap::new();
//     wifi_dict.insert("ssid".to_string(), Value::from(ssid.as_bytes()));
//
//     let mut security_dict = HashMap::new();
//     security_dict.insert("key-mgmt".to_string(), Value::from("wpa-psk"));
//     security_dict.insert("psk".to_string(), Value::from(password));
//
//     let mut full_settings = HashMap::new();
//     full_settings.insert("connection".to_string(), conn_dict);
//     full_settings.insert("802-11-wireless".to_string(), wifi_dict);
//     full_settings.insert("802-11-wireless-security".to_string(), security_dict);
//
//     // ملاحظة هامة: "/" في الـ device_path قد لا تعمل في كل الأنظمة
//     // NetworkManager يفضل مسار كرت الشبكة الفعلي (مثل /org/freedesktop/NetworkManager/Devices/3)
//     let device_path = zbus::zvariant::ObjectPath::from_str_unchecked("/");
//     let specific_object = zbus::zvariant::ObjectPath::from_str_unchecked("/");
//
//     // الترتيب الصحيح للـ Turbofish حسب إصدار zbus 5
//     proxy.call::<&str, _, (OwnedObjectPath, OwnedObjectPath)>(
//         "AddAndActivateConnection",
//         &(full_settings, &device_path, &specific_object),
//     ).await?;
//
//     Ok(())
// }
//
// pub async fn forget_network(ssid: &str) -> anyhow::Result<()> {
//     let connection = zbus::Connection::system().await?;
//     let proxy = zbus::Proxy::new(
//         &connection,
//         "org.freedesktop.NetworkManager",
//         "/org/freedesktop/NetworkManager/Settings",
//         "org.freedesktop.NetworkManager.Settings",
//     ).await?;
//
//     // جلب كل المسارات للاتصالات المحفوظة
//     let connections: Vec<zbus::zvariant::OwnedObjectPath> = proxy.call("ListConnections", &()).await?;
//
//     for path in connections {
//         let conn_proxy = zbus::Proxy::new(
//             &connection,
//             "org.freedesktop.NetworkManager",
//             &path,
//             "org.freedesktop.NetworkManager.Settings.Connection",
//         ).await?;
//
//         // جلب إعدادات هذا الاتصال
//         let settings: std::collections::HashMap<String, std::collections::HashMap<String, zbus::zvariant::OwnedValue>> =
//             conn_proxy.call("GetSettings", &()).await?;
//
//         // إذا كان الـ SSID يطابق اللي نبي نحذفه
// if let Some(conn_settings) = settings.get("connection") {
//     if let Some(id_value) = conn_settings.get("id") {
//         // 1. نحاول نفك الـ Value إلى نوع zbus::zvariant::Str
//         // 2. نحوله لـ &str العادي حق Rust
// if let Ok(id_str) = id_value.downcast_ref::<zbus::zvariant::Str>() {
//     if id_str.as_str() == ssid {
//         conn_proxy.call::<&str, (), ()>("Delete", &()).await?;
//         return Ok(());
//     }
// }
//     }
// }    }
//     Ok(())
// }
