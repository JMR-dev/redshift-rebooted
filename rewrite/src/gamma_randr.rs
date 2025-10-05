/// X11 RandR gamma adjustment method
/// Ported from legacy/src/gamma-randr.c

use crate::colorramp::colorramp_fill;
use crate::gamma::GammaMethod;
use crate::types::ColorSetting;
use std::fmt;
use x11rb::connection::Connection;
use x11rb::protocol::randr;
use x11rb::protocol::xproto;
use x11rb::rust_connection::RustConnection;

const RANDR_VERSION_MAJOR: u32 = 1;
const RANDR_VERSION_MINOR: u32 = 3;

/// State for a single CRTC
struct CrtcState {
    crtc: randr::Crtc,
    ramp_size: u16,
    saved_ramps: Vec<u16>, // R, G, B ramps concatenated (3 * ramp_size)
}

/// X11 RandR gamma adjustment method
pub struct RandrGammaMethod {
    conn: Option<RustConnection>,
    screen_num: Option<i32>,
    preferred_screen: usize,
    crtc_filter: Vec<usize>, // If non-empty, only adjust these CRTC indices
    crtcs: Vec<CrtcState>,
}

impl RandrGammaMethod {
    pub fn new() -> Self {
        Self {
            conn: None,
            screen_num: None,
            preferred_screen: 0,
            crtc_filter: Vec::new(),
            crtcs: Vec::new(),
        }
    }

    /// Set which screen to use (None = use default)
    pub fn set_screen(&mut self, screen: i32) {
        self.screen_num = Some(screen);
    }

    /// Set which CRTCs to adjust (empty = all)
    pub fn set_crtcs(&mut self, crtc_indices: Vec<usize>) {
        self.crtc_filter = crtc_indices;
    }

    fn get_screen_root(&self) -> Result<xproto::Window, String> {
        let conn = self.conn.as_ref().ok_or("Not connected to X server")?;

        let screen_num = self.screen_num.unwrap_or(self.preferred_screen as i32);
        let setup = conn.setup();
        let screen = setup
            .roots
            .get(screen_num as usize)
            .ok_or_else(|| format!("Screen {} could not be found", screen_num))?;

        Ok(screen.root)
    }

    fn set_temperature_for_crtc(
        &self,
        crtc_state: &CrtcState,
        setting: &ColorSetting,
        preserve: bool,
    ) -> Result<(), String> {
        let conn = self.conn.as_ref().ok_or("Not connected to X server")?;
        let ramp_size = crtc_state.ramp_size as usize;

        /* Create new gamma ramps */
        let mut gamma_r = vec![0u16; ramp_size];
        let mut gamma_g = vec![0u16; ramp_size];
        let mut gamma_b = vec![0u16; ramp_size];

        if preserve {
            /* Initialize from saved state */
            gamma_r.copy_from_slice(&crtc_state.saved_ramps[0..ramp_size]);
            gamma_g.copy_from_slice(&crtc_state.saved_ramps[ramp_size..2 * ramp_size]);
            gamma_b.copy_from_slice(&crtc_state.saved_ramps[2 * ramp_size..3 * ramp_size]);
        } else {
            /* Initialize to linear (pure state) */
            for i in 0..ramp_size {
                let value = ((i as f64 / ramp_size as f64) * 65536.0) as u16;
                gamma_r[i] = value;
                gamma_g[i] = value;
                gamma_b[i] = value;
            }
        }

        /* Apply color temperature adjustment */
        colorramp_fill(&mut gamma_r, &mut gamma_g, &mut gamma_b, setting);

        /* Set gamma ramps */
        randr::set_crtc_gamma(
            conn,
            crtc_state.crtc,
            &gamma_r,
            &gamma_g,
            &gamma_b,
        )
        .map_err(|e| format!("Failed to set CRTC gamma: {}", e))?
        .check()
        .map_err(|e| format!("RANDR Set CRTC Gamma returned error: {:?}", e))?;

        Ok(())
    }
}

impl Default for RandrGammaMethod {
    fn default() -> Self {
        Self::new()
    }
}

impl GammaMethod for RandrGammaMethod {
    fn init(&mut self) -> Result<(), String> {
        /* Open X server connection */
        let (conn, preferred_screen) = RustConnection::connect(None)
            .map_err(|e| format!("Failed to connect to X server: {}", e))?;

        self.preferred_screen = preferred_screen;

        /* Query RandR version */
        let ver_reply = randr::query_version(&conn, RANDR_VERSION_MAJOR, RANDR_VERSION_MINOR)
            .map_err(|e| format!("Failed to query RANDR version: {}", e))?
            .reply()
            .map_err(|e| format!("RANDR Query Version returned error: {}", e))?;

        if ver_reply.major_version != RANDR_VERSION_MAJOR
            || ver_reply.minor_version < RANDR_VERSION_MINOR
        {
            return Err(format!(
                "Unsupported RANDR version ({}.{})",
                ver_reply.major_version, ver_reply.minor_version
            ));
        }

        self.conn = Some(conn);
        Ok(())
    }

    fn start(&mut self) -> Result<(), String> {
        let conn = self.conn.as_ref().ok_or("Not initialized")?;
        let root = self.get_screen_root()?;

        /* Get screen resources (list of CRTCs) */
        let res_reply = randr::get_screen_resources_current(conn, root)
            .map_err(|e| format!("Failed to get screen resources: {}", e))?
            .reply()
            .map_err(|e| format!("RANDR Get Screen Resources Current returned error: {}", e))?;

        let crtcs = res_reply.crtcs;

        /* Save CRTC state and gamma ramps */
        for crtc in crtcs {
            /* Get gamma ramp size */
            let gamma_size_reply = randr::get_crtc_gamma_size(conn, crtc)
                .map_err(|e| format!("Failed to get CRTC gamma size: {}", e))?
                .reply()
                .map_err(|e| format!("RANDR Get CRTC Gamma Size returned error: {}", e))?;

            let ramp_size = gamma_size_reply.size;

            if ramp_size == 0 {
                eprintln!("Warning: CRTC has gamma ramp size 0, skipping");
                continue;
            }

            /* Get current gamma ramps */
            let gamma_get_reply = randr::get_crtc_gamma(conn, crtc)
                .map_err(|e| format!("Failed to get CRTC gamma: {}", e))?
                .reply()
                .map_err(|e| format!("RANDR Get CRTC Gamma returned error: {}", e))?;

            /* Save gamma ramps (R, G, B concatenated) */
            let mut saved_ramps = Vec::with_capacity(3 * ramp_size as usize);
            saved_ramps.extend_from_slice(&gamma_get_reply.red);
            saved_ramps.extend_from_slice(&gamma_get_reply.green);
            saved_ramps.extend_from_slice(&gamma_get_reply.blue);

            self.crtcs.push(CrtcState {
                crtc,
                ramp_size,
                saved_ramps,
            });
        }

        if self.crtcs.is_empty() {
            return Err("No usable CRTCs found".to_string());
        }

        Ok(())
    }

    fn set_temperature(&mut self, setting: &ColorSetting, preserve: bool) -> Result<(), String> {
        /* If no CRTC filter is set, adjust all CRTCs */
        if self.crtc_filter.is_empty() {
            for crtc_state in &self.crtcs {
                self.set_temperature_for_crtc(crtc_state, setting, preserve)?;
            }
        } else {
            /* Only adjust specified CRTCs */
            for &crtc_idx in &self.crtc_filter {
                if crtc_idx >= self.crtcs.len() {
                    return Err(format!(
                        "CRTC {} does not exist. Valid CRTCs are [0-{}]",
                        crtc_idx,
                        self.crtcs.len() - 1
                    ));
                }
                self.set_temperature_for_crtc(&self.crtcs[crtc_idx], setting, preserve)?;
            }
        }

        Ok(())
    }

    fn restore(&mut self) {
        if let Some(conn) = &self.conn {
            /* Restore original gamma ramps for all CRTCs */
            for crtc_state in &self.crtcs {
                let ramp_size = crtc_state.ramp_size as usize;
                let gamma_r = &crtc_state.saved_ramps[0..ramp_size];
                let gamma_g = &crtc_state.saved_ramps[ramp_size..2 * ramp_size];
                let gamma_b = &crtc_state.saved_ramps[2 * ramp_size..3 * ramp_size];

                match randr::set_crtc_gamma(conn, crtc_state.crtc, gamma_r, gamma_g, gamma_b) {
                    Ok(cookie) => {
                        if let Err(e) = cookie.check() {
                            eprintln!("Warning: Failed to restore CRTC gamma: {:?}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to send restore CRTC gamma request: {:?}", e);
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "randr"
    }

    fn print_help(&self) {
        println!("Adjust gamma ramps with the X RANDR extension.");
        println!();
        println!("  screen=N    X screen to apply adjustments to");
        println!("  crtc=N      List of comma separated CRTCs to apply adjustments to");
        println!();
    }
}

impl fmt::Display for RandrGammaMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RandR")
    }
}

impl Drop for RandrGammaMethod {
    fn drop(&mut self) {
        self.restore();
    }
}
