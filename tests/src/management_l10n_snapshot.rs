//! Snapshot tests for management decision and incident localization.
//!
//! This test verifies that all management decisions and incidents can be properly
//! localized with their parameters, ensuring the sign display and formatting work correctly.

mod harness;

use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Once, OnceLock},
};

use dishaster_core::{
    convert::{RealizationContext, TemplateRealize, ViewParams},
    models::{ManagementDecisionTemplateDef, ManagementIncidentTemplateDef},
    views::{ParamValue, ParamsMap},
};
use dishaster_data::{DataLoader, GameDataAssets};
use dishrupt_core::{ModelId, prelude::EcoString};
use dishrupt_l10n::{
    L10N_SERVICE, build_arc_loader,
    fluent::{FluentValue, types::FluentNumber},
    langid, tr,
};
use dishrupt_rng::Prng;
use harness::data_dir;
use serde::Serialize;

// === Test Setup ===

/// Initialize localization system once
fn init_localization() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        // Test CWD is tests/, need to go up one level to workspace root
        let locales_path = "../godot/locales/";
        L10N_SERVICE.set_locales(build_arc_loader(locales_path, langid!("zh-CN")));
    });
}

/// Load game data from assets
fn load_data() -> &'static GameDataAssets {
    static DATA: OnceLock<GameDataAssets> = OnceLock::new();
    DATA.get_or_init(|| {
        let mut loader = DataLoader::from_fs(data_dir()).expect("Failed to create loader");
        loader.load_all_data().expect("Failed to load game data")
    })
}

// === Snapshot Structures ===

/// Snapshot of a single decision/incident with localization
#[derive(Debug, Serialize)]
struct ManagementSnapshot {
    id: String,
    seed: u64,
    /// Localized strings in zh-CN
    localized: LocalizedManagement,
}

/// Localized strings for a decision
#[derive(Debug, Serialize)]
struct LocalizedManagement {
    title: String,
    effects: String,
}

// Copied from `dishaster-godot-ui` for parameter conversion
#[allow(missing_docs)]
pub trait ToFluent<'s> {
    type Output;

    fn to_fluent(&'s self) -> Self::Output;
}

impl<'s> ToFluent<'s> for ParamsMap {
    type Output = HashMap<EcoString, FluentValue<'s>>;

    fn to_fluent(&'s self) -> HashMap<EcoString, FluentValue<'s>> {
        let mut map = HashMap::new();
        for (k, v) in self.0.iter() {
            map.insert(k.clone(), v.to_fluent());
        }
        map
    }
}

impl<'s> ToFluent<'s> for ParamValue {
    type Output = FluentValue<'s>;

    fn to_fluent(&'s self) -> FluentValue<'s> {
        match self {
            ParamValue::String(s) => FluentValue::String(Cow::Borrowed(s.as_str())),
            ParamValue::Int(i) => {
                FluentValue::Number(FluentNumber::new(*i as f64, Default::default()))
            }
            ParamValue::Float(f) => FluentValue::Number(FluentNumber::new(*f, Default::default())),
        }
    }
}

/// Generate a snapshot for a decision with the given seed
fn snapshot_decision(
    id: &ModelId,
    template: &ManagementDecisionTemplateDef,
    seed: u64,
) -> ManagementSnapshot {
    let mut rng = Prng::new(seed);
    let ctx = RealizationContext { rng: &mut rng };
    let model = template.realize(ctx);
    let params = model.params();

    let fluent_args = params.to_fluent();

    let key = format!("mgmt--{id}");
    let localized = LocalizedManagement {
        title: tr!("{}.title", key),
        effects: tr!("{}.effects", key; fluent_args),
    };

    ManagementSnapshot {
        id: id.to_string(),
        seed,
        localized,
    }
}

/// Generate a snapshot for an incident with the given seed
fn snapshot_incident(
    id: &ModelId,
    template: &ManagementIncidentTemplateDef,
    seed: u64,
) -> ManagementSnapshot {
    let mut rng = Prng::new(seed);
    let ctx = RealizationContext { rng: &mut rng };
    let model = template.realize(ctx);
    let params = model.params();

    let fluent_args = params.to_fluent();

    let key = format!("mgmt--{id}");
    let localized = LocalizedManagement {
        title: tr!("{}.title", key),
        effects: tr!("{}.effects", key; fluent_args),
    };

    ManagementSnapshot {
        id: id.to_string(),
        seed,
        localized,
    }
}

// === Snapshot Tests ===

#[test]
fn test_management_decisions_l10n() {
    init_localization();
    let data = load_data();

    // Test with multiple seeds to verify parameter ranges and sign display
    let test_seeds = [0x1234567890abcdef, 0xfedcba0987654321, 0xaaaaaaaaaaaaaaaa];

    let mut items = Vec::new();

    for decision in data.models.mgmt_decisions.iter() {
        for &seed in &test_seeds {
            items.push(snapshot_decision(&decision.id, &decision.def, seed));
        }
    }
    for incident in data.models.mgmt_incidents.iter() {
        for &seed in &test_seeds {
            items.push(snapshot_incident(&incident.id, &incident.def, seed));
        }
    }

    insta::with_settings!({
        prepend_module_to_snapshot => false,
        snapshot_path => "../snapshots",
        omit_expression => true,
    }, {
        insta::assert_yaml_snapshot!("management_l10n", items);
    });
}
