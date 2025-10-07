/* Unit tests for GammaRestoreGuard functionality */

use redshift_rebooted::gamma::{DummyGammaMethod, GammaMethod};
use redshift_rebooted::gamma_guard::GammaRestoreGuard;
use redshift_rebooted::types::ColorSetting;

#[test]
fn test_gamma_guard_restores_on_drop() {
    /* Create a gamma method */
    let mut gamma = DummyGammaMethod::new();
    gamma.init().expect("Init failed");
    gamma.start().expect("Start failed");

    /* Set a custom temperature */
    let custom_setting = ColorSetting {
        temperature: 3500,
        brightness: 0.9,
        gamma: [1.0, 0.8, 0.7],
    };
    gamma.set_temperature(&custom_setting, false).expect("Set temp failed");

    /* Create guard - this should restore gamma when dropped */
    {
        let _guard = GammaRestoreGuard::new(&mut gamma);
        /* Guard goes out of scope here and should restore */
    }

    /* The gamma method should have been called to restore to 6500K
       (we can't directly verify this with DummyGammaMethod, but the guard should have called it) */
}

#[test]
fn test_gamma_guard_can_be_disabled() {
    /* Create a gamma method */
    let mut gamma = DummyGammaMethod::new();
    gamma.init().expect("Init failed");
    gamma.start().expect("Start failed");

    /* Set a custom temperature */
    let custom_setting = ColorSetting {
        temperature: 3500,
        brightness: 0.9,
        gamma: [1.0, 0.8, 0.7],
    };
    gamma.set_temperature(&custom_setting, false).expect("Set temp failed");

    /* Create guard and disable restoration */
    {
        let mut guard = GammaRestoreGuard::new(&mut gamma);
        guard.disable_restore();
        /* Guard goes out of scope but should NOT restore */
    }

    /* Gamma should remain at custom setting (not restored) */
}

#[test]
fn test_gamma_guard_get_mut() {
    /* Create a gamma method */
    let mut gamma = DummyGammaMethod::new();
    gamma.init().expect("Init failed");
    gamma.start().expect("Start failed");

    /* Create guard */
    let mut guard = GammaRestoreGuard::new(&mut gamma);

    /* Use guard to set temperature */
    let setting = ColorSetting {
        temperature: 4000,
        brightness: 1.0,
        gamma: [1.0, 1.0, 1.0],
    };

    /* Should be able to get mutable reference and use it */
    guard.get_mut().set_temperature(&setting, false).expect("Set temp through guard failed");
}

#[test]
#[should_panic(expected = "panic test")]
fn test_gamma_guard_restores_on_panic() {
    /* Create a gamma method */
    let mut gamma = DummyGammaMethod::new();
    gamma.init().expect("Init failed");
    gamma.start().expect("Start failed");

    /* Set a custom temperature */
    let custom_setting = ColorSetting {
        temperature: 3500,
        brightness: 0.9,
        gamma: [1.0, 0.8, 0.7],
    };
    gamma.set_temperature(&custom_setting, false).expect("Set temp failed");

    /* Create guard */
    let _guard = GammaRestoreGuard::new(&mut gamma);

    /* Panic - guard should still restore gamma */
    panic!("panic test");
}

#[test]
fn test_multiple_guards_sequential() {
    /* Create a gamma method */
    let mut gamma = DummyGammaMethod::new();
    gamma.init().expect("Init failed");
    gamma.start().expect("Start failed");

    /* First guard */
    {
        let mut guard = GammaRestoreGuard::new(&mut gamma);
        let setting = ColorSetting {
            temperature: 3000,
            brightness: 0.8,
            gamma: [1.0, 0.9, 0.8],
        };
        guard.get_mut().set_temperature(&setting, false).expect("Failed");
    } /* Restores here */

    /* Second guard */
    {
        let mut guard = GammaRestoreGuard::new(&mut gamma);
        let setting = ColorSetting {
            temperature: 5000,
            brightness: 0.95,
            gamma: [1.0, 1.0, 0.9],
        };
        guard.get_mut().set_temperature(&setting, false).expect("Failed");
    } /* Restores here too */
}

#[test]
fn test_guard_restores_neutral_values() {
    /* The guard should restore to specific neutral values:
       - Temperature: 6500K
       - Brightness: 1.0
       - Gamma: [1.0, 1.0, 1.0]
    */
    let mut gamma = DummyGammaMethod::new();
    gamma.init().expect("Init failed");
    gamma.start().expect("Start failed");

    /* Set extreme values */
    let extreme_setting = ColorSetting {
        temperature: 2000,
        brightness: 0.5,
        gamma: [0.5, 0.6, 0.7],
    };
    gamma.set_temperature(&extreme_setting, false).expect("Set temp failed");

    /* Create and drop guard */
    {
        let _guard = GammaRestoreGuard::new(&mut gamma);
    }

    /* Guard should have called set_temperature with neutral values */
    /* Note: With DummyGammaMethod we can't verify the exact call,
       but in real usage with RandrGammaMethod, the display would be reset */
}
