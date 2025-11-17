//! Permanent effects from management decisions
//!
//! These effects persist across days and are saved in the player profile.

use crate::prelude::*;

/// Container for all permanent effects from management decisions
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PermanentEffects {
    /// Currently active music effect (only one can be active, replaces previous)
    pub music: Option<MusicEffect>,

    /// Active advertising campaigns (can stack)
    pub campaigns: Vec<CampaignEffect>,

    /// Active motivational slogans (can stack)
    pub slogans: Vec<SloganEffect>,

    /// Luxury dishes that have been unlocked (can only unlock once per dish)
    pub luxury_dishes: FxHashSet<ModelId>,

    /// Daily incident: attraction multiplier (reset each day)
    #[serde(skip)]
    pub daily_attraction_multiplier: f32,
}

/// Effect from playing music decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicEffect {
    /// Multiplier for eating time (e.g., 0.9 means 10% faster)
    pub eating_time_multiplier: f32,
    /// Change to satisfaction (can be positive or negative)
    pub satisfaction_change: f32,
}

/// Campaign target type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CampaignTarget {
    /// Campaign targets the entire canteen
    Canteen,
    /// Campaign targets a specific window service
    Window(ModelId),
}

/// Effect from advertising campaign decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignEffect {
    /// Target of the campaign
    pub target: CampaignTarget,
    /// Current attraction boost multiplier (starts high, decays over time)
    pub current_boost: f32,
    /// Number of days remaining
    pub days_remaining: u32,
    /// Decay rate per day (0..1)
    pub decay_rate: f32,
}

impl CampaignEffect {
    /// Apply daily decay to the campaign effect
    pub fn apply_daily_decay(&mut self) {
        if self.days_remaining > 0 {
            self.days_remaining -= 1;
            self.current_boost *= 1.0 - self.decay_rate;
        }
    }

    /// Check if the campaign has expired
    pub fn is_expired(&self) -> bool {
        self.days_remaining == 0
    }
}

/// Effect from motivational slogan decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloganEffect {
    /// Trust threshold (diners with trust below this will be unhappy)
    pub trust_threshold: f32,
    /// Satisfaction boost for trusting diners
    pub satisfaction_boost: f32,
    /// Satisfaction penalty for distrusting diners
    pub satisfaction_penalty: f32,
}

impl PermanentEffects {
    /// Get the effective eating time multiplier from music
    pub fn get_eating_time_multiplier(&self) -> f32 {
        self.music
            .as_ref()
            .map(|m| m.eating_time_multiplier)
            .unwrap_or(1.0)
    }

    /// Get the satisfaction change from music
    pub fn get_music_satisfaction_change(&self) -> f32 {
        self.music
            .as_ref()
            .map(|m| m.satisfaction_change)
            .unwrap_or(0.0)
    }

    /// Get total attraction boost for canteen (includes daily incident effects)
    pub fn get_canteen_attraction_boost(&self) -> f32 {
        let campaign_boost = self
            .campaigns
            .iter()
            .filter(|c| matches!(c.target, CampaignTarget::Canteen))
            .map(|c| c.current_boost - 1.0) // Convert multiplier to additive bonus
            .sum::<f32>()
            + 1.0; // Convert back to multiplier

        // Apply daily incident multiplier
        campaign_boost * self.daily_attraction_multiplier
    }

    /// Get attraction boost for a specific window
    pub fn get_window_attraction_boost(&self, window_id: &ModelId) -> f32 {
        self.campaigns
            .iter()
            .filter(|c| matches!(&c.target, CampaignTarget::Window(id) if id == window_id))
            .map(|c| c.current_boost - 1.0)
            .sum::<f32>()
            + 1.0
    }

    /// Calculate satisfaction adjustment from slogans based on diner trust
    pub fn get_slogan_satisfaction_adjustment(&self, diner_trust: f32) -> f32 {
        self.slogans
            .iter()
            .map(|slogan| {
                if diner_trust >= slogan.trust_threshold {
                    slogan.satisfaction_boost
                } else {
                    -slogan.satisfaction_penalty
                }
            })
            .sum()
    }

    /// Check if a luxury dish has been unlocked
    pub fn has_luxury_dish(&self, dish_id: &ModelId) -> bool {
        self.luxury_dishes.contains(dish_id)
    }

    /// Apply daily decay to all campaigns
    pub fn apply_daily_decay(&mut self) {
        for campaign in &mut self.campaigns {
            campaign.apply_daily_decay();
        }
        // Remove expired campaigns
        self.campaigns.retain(|c| !c.is_expired());
    }

    /// Reset daily incident effects (called at start of new day)
    pub fn reset_daily_effects(&mut self) {
        self.daily_attraction_multiplier = 1.0;
    }
}
