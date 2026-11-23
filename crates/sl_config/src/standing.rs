//! Where a resolved configuration stands among the shipped size tiers: the tier it extends, what it adds, and how
//! many of the registered lints it turns on next to the smallest and the biggest size tier.

use crate::{Config, Env};
use sl_core::Level;

/// Where a configuration stands among the size tiers ([`Env::ladder`])
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Standing {
    /// The size tier the configuration extends directly (the last one named), if any
    pub tier: Option<String>,
    /// The other names it extends directly: sets, the tiers beside the size tiers, shared files
    pub beside: Vec<String>,
    /// The registered lints it turns on: level above allow
    pub on: u32,
    /// The registered lints
    pub total: u32,
    /// The lints the smallest and the biggest size tier turn on alone; None without a ladder of two tiers
    pub scale: Option<(u32, u32)>,
}

impl Standing {
    /// The standing of `config`, resolved with `env`
    pub(crate) fn of(config: &Config, env: &Env<'_>) -> Self {
        let tier = config
            .extends
            .iter()
            .rev()
            .find(|e| env.ladder.contains(&e.as_str()))
            .cloned();
        let beside = config
            .extends
            .iter()
            .filter(|e| Some(*e) != tier.as_ref())
            .cloned()
            .collect();
        let scale = match env.ladder {
            [low, .., high] => {
                let on = |t: &str| Config::resolve_tiers(&[t], env).map(|c| lints_on(&c));
                on(low).ok().zip(on(high).ok())
            }
            _ => None,
        };
        Self {
            tier,
            beside,
            on: lints_on(config),
            total: count(config.lints.len()),
            scale,
        }
    }

    /// Where the configuration sits between the smallest size tier (0) and the biggest (1), clamped; without a
    /// scale, the share of registered lints it turns on
    pub fn position(&self) -> f64 {
        let (low, high) = match self.scale {
            Some((low, high)) if high > low => (f64::from(low), f64::from(high)),
            _ => (0.0, f64::from(self.total.max(1))),
        };
        ((f64::from(self.on) - low) / (high - low)).clamp(0.0, 1.0)
    }

    /// The tier as a badge names it: the size tier, `+N` for the other names extended (`medium +2`); without a
    /// size tier, the first name extended, or `custom`
    pub fn label(&self) -> String {
        let (name, rest) = match (&self.tier, self.beside.split_first()) {
            (Some(t), _) => (t.as_str(), self.beside.len()),
            (None, Some((first, rest))) => (first.as_str(), rest.len()),
            (None, None) => ("custom", 0),
        };
        if rest == 0 {
            name.to_owned()
        } else {
            format!("{name} +{rest}")
        }
    }
}

/// The lints a configuration turns on at the project's own level
fn lints_on(config: &Config) -> u32 {
    count(
        config
            .lints
            .values()
            .filter(|l| l.level > Level::Allow)
            .count(),
    )
}

/// A count of registered lints, which stays far below `u32::MAX`
fn count(n: usize) -> u32 {
    u32::try_from(n).expect("fewer than 2^32 registered lints")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standing(
        tier: Option<&str>,
        beside: &[&str],
        on: u32,
        scale: Option<(u32, u32)>,
    ) -> Standing {
        Standing {
            tier: tier.map(str::to_owned),
            beside: beside.iter().map(|s| (*s).to_owned()).collect(),
            on,
            total: 200,
            scale,
        }
    }

    #[test]
    fn label_names_the_size_tier_and_counts_the_rest() {
        assert_eq!(standing(Some("big"), &[], 0, None).label(), "big");
        assert_eq!(
            standing(Some("medium"), &["ai", "units-strict"], 0, None).label(),
            "medium +2"
        );
        assert_eq!(
            standing(None, &["panics", "docs"], 0, None).label(),
            "panics +1"
        );
        assert_eq!(standing(None, &[], 0, None).label(), "custom");
    }

    #[test]
    fn position_is_clamped_to_the_scale() {
        let s = |on| standing(None, &[], on, Some((50, 150))).position();
        assert!((s(100) - 0.5).abs() < 1e-9);
        assert!(s(10).abs() < 1e-9);
        assert!((s(190) - 1.0).abs() < 1e-9);
        assert!((standing(None, &[], 50, None).position() - 0.25).abs() < 1e-9);
    }
}
