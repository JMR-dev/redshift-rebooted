/// Location providers
/// Ported from legacy/src/location-*.c

use crate::types::Location;
use log::{debug, error, info, trace};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::oneshot;

/// Trait for location providers
pub trait LocationProvider {
    /// Initialize the provider
    fn init(&mut self) -> Result<(), String>;

    /// Start the provider
    fn start(&mut self) -> Result<(), String>;

    /// Get the current location
    fn get_location(&mut self) -> Result<Location, String>;

    /// Get the provider name
    fn name(&self) -> &str;

    /// Print help information
    fn print_help(&self);

    /// Set an option (key-value pair)
    fn set_option(&mut self, key: &str, value: &str) -> Result<(), String>;
}

/// Manual location provider
/// Ported from legacy/src/location-manual.c
pub struct ManualLocationProvider {
    location: Option<Location>,
}

impl ManualLocationProvider {
    pub fn new() -> Self {
        Self { location: None }
    }

    pub fn with_location(lat: f32, lon: f32) -> Self {
        Self {
            location: Some(Location { lat, lon }),
        }
    }
}

impl Default for ManualLocationProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationProvider for ManualLocationProvider {
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), String> {
        if self.location.is_none() {
            return Err("Latitude and longitude must be set.".to_string());
        }
        Ok(())
    }

    fn get_location(&mut self) -> Result<Location, String> {
        self.location
            .ok_or_else(|| "Location not set".to_string())
    }

    fn name(&self) -> &str {
        "manual"
    }

    fn print_help(&self) {
        println!("Specify location manually.");
        println!();
        println!("  lat=N\t\tLatitude");
        println!("  lon=N\t\tLongitude");
        println!();
        println!("Both values are expected to be floating point numbers,");
        println!("negative values representing west / south, respectively.");
        println!();
    }

    fn set_option(&mut self, key: &str, value: &str) -> Result<(), String> {
        let v: f32 = value
            .parse()
            .map_err(|_| format!("Malformed argument: {}", value))?;

        match key.to_lowercase().as_str() {
            "lat" => {
                let mut loc = self.location.unwrap_or(Location { lat: 0.0, lon: 0.0 });
                loc.lat = v;
                self.location = Some(loc);
                Ok(())
            }
            "lon" => {
                let mut loc = self.location.unwrap_or(Location { lat: 0.0, lon: 0.0 });
                loc.lon = v;
                self.location = Some(loc);
                Ok(())
            }
            _ => Err(format!("Unknown method parameter: `{}`", key)),
        }
    }
}

/// GeoClue2 location provider (automatic location detection)
/// Ported from legacy/src/location-geoclue2.c
pub struct GeoClue2LocationProvider {
    location: Arc<Mutex<Option<Location>>>,
    error: Arc<Mutex<Option<String>>>,
    thread_handle: Option<thread::JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl GeoClue2LocationProvider {
    pub fn new() -> Self {
        Self {
            location: Arc::new(Mutex::new(None)),
            error: Arc::new(Mutex::new(None)),
            thread_handle: None,
            shutdown_tx: None,
        }
    }
}

impl Default for GeoClue2LocationProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationProvider for GeoClue2LocationProvider {
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), String> {
        debug!("Starting GeoClue2 location provider");
        let location = Arc::clone(&self.location);
        let error = Arc::clone(&self.error);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Spawn a thread to run the tokio runtime for GeoClue2
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(async move {
                if let Err(e) = geoclue2_async_task(location.clone(), error.clone(), shutdown_rx).await {
                    error!("GeoClue2 error: {}", e);
                    let mut err = error.lock().unwrap();
                    *err = Some(format!("GeoClue2 error: {}", e));
                }
            });
        });

        self.thread_handle = Some(handle);
        self.shutdown_tx = Some(shutdown_tx);

        // Wait a moment for initial location
        debug!("Waiting for initial location from GeoClue2");
        thread::sleep(std::time::Duration::from_millis(500));

        Ok(())
    }

    fn get_location(&mut self) -> Result<Location, String> {
        // Check for errors first
        if let Some(err_msg) = self.error.lock().unwrap().as_ref() {
            return Err(err_msg.clone());
        }

        let loc = self.location.lock().unwrap();
        loc.ok_or_else(|| "Location not yet available from GeoClue2".to_string())
    }

    fn name(&self) -> &str {
        "geoclue2"
    }

    fn print_help(&self) {
        println!("Use the location as discovered by a GeoClue2 provider.");
        println!();
    }

    fn set_option(&mut self, key: &str, _value: &str) -> Result<(), String> {
        Err(format!("Unknown method parameter: `{}`", key))
    }
}

impl Drop for GeoClue2LocationProvider {
    fn drop(&mut self) {
        // Signal shutdown
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Async task that handles GeoClue2 D-Bus communication
async fn geoclue2_async_task(
    location: Arc<Mutex<Option<Location>>>,
    error: Arc<Mutex<Option<String>>>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    use zbus::{Connection, proxy};
    use futures_util::stream::StreamExt;

    // Connect to system D-Bus
    let conn = Connection::system().await?;

    // Define GeoClue2 Manager proxy
    #[proxy(
        interface = "org.freedesktop.GeoClue2.Manager",
        default_service = "org.freedesktop.GeoClue2",
        default_path = "/org/freedesktop/GeoClue2/Manager"
    )]
    trait Manager {
        fn get_client(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    }

    // Define GeoClue2 Client proxy
    #[proxy(
        interface = "org.freedesktop.GeoClue2.Client",
        default_service = "org.freedesktop.GeoClue2"
    )]
    trait Client {
        fn start(&self) -> zbus::Result<()>;
        fn stop(&self) -> zbus::Result<()>;

        #[zbus(property)]
        fn set_desktop_id(&self, desktop_id: &str) -> zbus::Result<()>;

        #[zbus(property)]
        fn set_distance_threshold(&self, threshold: u32) -> zbus::Result<()>;

        #[zbus(property)]
        fn location(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

        #[zbus(signal)]
        fn location_updated(&self, old: zbus::zvariant::OwnedObjectPath, new: zbus::zvariant::OwnedObjectPath) -> zbus::Result<()>;
    }

    // Define GeoClue2 Location proxy
    #[proxy(
        interface = "org.freedesktop.GeoClue2.Location",
        default_service = "org.freedesktop.GeoClue2"
    )]
    trait GeoLocation {
        #[zbus(property)]
        fn latitude(&self) -> zbus::Result<f64>;

        #[zbus(property)]
        fn longitude(&self) -> zbus::Result<f64>;

        #[zbus(property)]
        fn accuracy(&self) -> zbus::Result<f64>;
    }

    // Get GeoClue2 Manager
    let manager = ManagerProxy::new(&conn).await?;
    debug!("Connected to GeoClue2 Manager");

    // Get client path
    let client_path = manager.get_client().await.map_err(|e| {
        format!("Failed to get GeoClue2 client: {}. Make sure location services are enabled and Redshift has permission to access location.", e)
    })?;
    debug!("Got GeoClue2 client path: {:?}", client_path);

    // Create client proxy
    let client = ClientProxy::builder(&conn)
        .path(client_path)?
        .build()
        .await?;

    // Set desktop ID
    if let Err(e) = client.set_desktop_id("redshift").await {
        debug!("Could not set desktop ID: {}", e);
    }

    // Set distance threshold (50km)
    if let Err(e) = client.set_distance_threshold(50000).await {
        debug!("Could not set distance threshold: {}", e);
    }

    // Subscribe to location updates
    let mut location_stream = client.receive_location_updated().await?;

    // Start the client
    debug!("Starting GeoClue2 client...");
    if let Err(e) = client.start().await {
        let err_str = e.to_string();
        if err_str.contains("AccessDenied") || err_str.contains("org.freedesktop.DBus.Error.AccessDenied") {
            return Err("Access to location was denied by GeoClue! Make sure location services are enabled and Redshift has permission to access location services.".into());
        } else {
            return Err(format!("Failed to start GeoClue2 client: {}", err_str).into());
        }
    }
    debug!("GeoClue2 client started, waiting for location updates...");

    // Try to get initial location from Location property
    if let Ok(loc_path) = client.location().await {
        if loc_path.as_str() != "/" {
            debug!("Got initial location path: {:?}", loc_path);
            let geo_location_result = GeoLocationProxy::builder(&conn)
                .path(&loc_path)
                .unwrap()
                .build()
                .await;

            if let Ok(geo_location) = geo_location_result {
                if let (Ok(lat), Ok(lon)) = (geo_location.latitude().await, geo_location.longitude().await) {
                    let mut loc = location.lock().unwrap();
                    *loc = Some(Location {
                        lat: lat as f32,
                        lon: lon as f32,
                    });
                    info!("Initial location from GeoClue2: {:.2}, {:.2}", lat, lon);
                }
            }
        }
    }

    // Wait for location updates or shutdown signal
    loop {
        tokio::select! {
            Some(signal) = location_stream.next() => {
                let args = signal.args()?;
                let new_location_path = &args.new;

                // Get location details
                let geo_location = GeoLocationProxy::builder(&conn)
                    .path(new_location_path)?
                    .build()
                    .await?;

                let lat = geo_location.latitude().await?;
                let lon = geo_location.longitude().await?;

                // Update shared location
                let mut loc = location.lock().unwrap();
                *loc = Some(Location {
                    lat: lat as f32,
                    lon: lon as f32,
                });

                info!("Location updated from GeoClue2: {:.2}, {:.2}", lat, lon);
                trace!("New location path: {:?}", new_location_path);
            }
            _ = &mut shutdown_rx => {
                // Shutdown requested
                debug!("GeoClue2 shutdown requested");
                let _ = client.stop().await;
                return Ok(());
            }
        }
    }
}
