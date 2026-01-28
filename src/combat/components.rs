//! Combat-related components.

use bevy::prelude::*;

// Re-export from core to avoid duplication
pub use crate::core::{DamageEvent, DeathEvent, Element};

/// Component for entities that can take damage.
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            maximum: max,
        }
    }

    pub fn take_damage(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.current);
        self.current -= actual;
        actual
    }

    pub fn heal(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.maximum - self.current);
        self.current += actual;
        actual
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn percentage(&self) -> f32 {
        self.current / self.maximum
    }
}

/// Elemental resistances (percentage reduction, 0.0 to 1.0).
#[derive(Component, Default)]
pub struct Resistances {
    pub physical: f32,
    pub fire: f32,
    pub ice: f32,
    pub lightning: f32,
    pub poison: f32,
    pub holy: f32,
    pub dark: f32,
}

impl Resistances {
    pub fn get(&self, element: Element) -> f32 {
        match element {
            Element::Physical => self.physical,
            Element::Fire => self.fire,
            Element::Ice => self.ice,
            Element::Lightning => self.lightning,
            Element::Poison => self.poison,
            Element::Holy => self.holy,
            Element::Dark => self.dark,
        }
    }
}

/// Weapon definition component.
#[derive(Component)]
pub struct Weapon {
    pub name: String,
    pub base_damage: f32,
    pub element: Element,
    /// Attack range in units
    pub reach: f32,
    /// Damage reduction when blocking (0.0 to 1.0)
    pub block_efficiency: f32,
    /// Stamina cost per swing
    pub stamina_cost: f32,
    /// Attack cooldown in seconds
    pub attack_cooldown: f32,
    /// Path to the .glb model file
    pub model_path: String,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            name: "Fists".to_string(),
            base_damage: 5.0,
            element: Element::Physical,
            reach: 1.5,
            block_efficiency: 0.3,
            stamina_cost: 10.0,
            attack_cooldown: 0.5,
            model_path: String::new(),
        }
    }
}

/// Marker for the currently equipped weapon entity.
#[derive(Component)]
pub struct EquippedWeapon;

/// Combat state for an entity (player or enemy).
#[derive(Component, Default)]
pub struct CombatState {
    /// Is currently in attack animation
    pub is_attacking: bool,
    /// Is currently blocking
    pub is_blocking: bool,
    /// Cooldown timer after attack
    pub attack_cooldown: f32,
    /// Invincibility frames remaining
    pub i_frames: f32,
    /// Whether the current attack has already consumed stamina and done hit detection
    pub attack_executed: bool,
}

impl CombatState {
    pub fn can_attack(&self) -> bool {
        !self.is_attacking && self.attack_cooldown <= 0.0
    }

    pub fn can_block(&self) -> bool {
        !self.is_attacking
    }
}

/// Stamina resource for combat actions.
#[derive(Component)]
pub struct Stamina {
    pub current: f32,
    pub maximum: f32,
    pub regen_rate: f32,
    /// Delay before stamina starts regenerating after use
    pub regen_delay: f32,
    pub regen_timer: f32,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            maximum: 100.0,
            regen_rate: 20.0,
            regen_delay: 0.5,
            regen_timer: 0.0,
        }
    }
}

impl Stamina {
    pub fn use_stamina(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            self.regen_timer = self.regen_delay;
            true
        } else {
            false
        }
    }

    pub fn regenerate(&mut self, delta: f32) {
        if self.regen_timer > 0.0 {
            self.regen_timer -= delta;
        } else {
            self.current = (self.current + self.regen_rate * delta).min(self.maximum);
        }
    }
}

/// Event sent when an attack is executed.
#[derive(Event)]
pub struct AttackEvent {
    pub attacker: Entity,
    pub damage: f32,
    pub element: Element,
    pub direction: Vec3,
}

/// Marker component for entities that have died (prevents multiple death events).
#[derive(Component)]
pub struct Dead;

/// Marker for attack hitbox sensor.
#[derive(Component)]
pub struct AttackHitbox {
    pub owner: Entity,
    pub damage: f32,
    pub element: Element,
}

/// Screen shake effect resource.
#[derive(Resource, Default)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: f32,
    pub timer: f32,
}

impl ScreenShake {
    pub fn shake(&mut self, intensity: f32, duration: f32) {
        // Only override if new shake is stronger
        if intensity > self.intensity || self.timer <= 0.0 {
            self.intensity = intensity;
            self.duration = duration;
            self.timer = duration;
        }
    }

    pub fn update(&mut self, delta: f32) -> Vec3 {
        if self.timer <= 0.0 {
            return Vec3::ZERO;
        }

        self.timer -= delta;
        let progress = self.timer / self.duration;
        let current_intensity = self.intensity * progress;

        // Random offset
        let x = (rand::random::<f32>() - 0.5) * 2.0 * current_intensity;
        let y = (rand::random::<f32>() - 0.5) * 2.0 * current_intensity;

        Vec3::new(x, y, 0.0)
    }
}

/// Hit stop effect (brief pause on impact).
#[derive(Resource, Default)]
pub struct HitStop {
    pub duration: f32,
    pub timer: f32,
}

impl HitStop {
    pub fn trigger(&mut self, duration: f32) {
        self.duration = duration;
        self.timer = duration;
    }

    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }

    pub fn update(&mut self, delta: f32) {
        if self.timer > 0.0 {
            self.timer -= delta;
        }
    }
}
