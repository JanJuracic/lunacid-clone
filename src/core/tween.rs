//! Generic transform tweening system with easing support.

use bevy::prelude::*;

/// Component for smooth transform interpolation.
#[derive(Component)]
pub struct SmoothTransform {
    /// Target translation (None = don't animate)
    pub target_translation: Option<Vec3>,
    /// Target rotation (None = don't animate)
    pub target_rotation: Option<Quat>,
    /// Interpolation speed multiplier (higher = faster)
    pub translation_speed: f32,
    pub rotation_speed: f32,
}

impl Default for SmoothTransform {
    fn default() -> Self {
        Self {
            target_translation: None,
            target_rotation: None,
            translation_speed: 12.0,
            rotation_speed: 12.0,
        }
    }
}

impl SmoothTransform {
    pub fn new(translation_speed: f32, rotation_speed: f32) -> Self {
        Self {
            translation_speed,
            rotation_speed,
            ..default()
        }
    }
}

/// System that interpolates transforms toward their targets.
pub fn update_smooth_transforms(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &SmoothTransform)>,
) {
    let dt = time.delta_secs();

    for (mut transform, smooth) in query.iter_mut() {
        // Interpolate translation
        if let Some(target) = smooth.target_translation {
            let t = (smooth.translation_speed * dt).min(1.0);
            transform.translation = transform.translation.lerp(target, t);
        }

        // Interpolate rotation
        if let Some(target) = smooth.target_rotation {
            let t = (smooth.rotation_speed * dt).min(1.0);
            transform.rotation = transform.rotation.slerp(target, t);
        }
    }
}
