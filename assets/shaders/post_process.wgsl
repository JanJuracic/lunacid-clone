// Horror post-processing shader: film grain + CRT scanlines + vignette
// Combines multiple effects for Silent Hill 2 style atmosphere

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct PostProcessSettings {
    // Film grain
    grain_intensity: f32,
    grain_speed: f32,
    grain_coarseness: f32,
    // CRT scanlines
    scanline_intensity: f32,
    scanline_count: f32,
    // Vignette
    vignette_intensity: f32,
    vignette_radius: f32,
    // Animation
    time: f32,
}

@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

// Hash function for pseudo-random noise
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Film grain noise - animated per frame
fn film_grain(uv: vec2<f32>, time: f32, intensity: f32, speed: f32, coarseness: f32) -> f32 {
    let noise_uv = uv * coarseness + vec2<f32>(time * speed, time * 1234.5 * speed);
    let grain = hash(noise_uv) * 2.0 - 1.0;
    return grain * intensity;
}

// CRT scanlines effect
fn scanlines(uv: vec2<f32>, count: f32, intensity: f32) -> f32 {
    let line = sin(uv.y * count * 3.14159265) * 0.5 + 0.5;
    return 1.0 - (1.0 - line) * intensity;
}

// Vignette darkening at edges
fn vignette(uv: vec2<f32>, intensity: f32, radius: f32) -> f32 {
    let center = uv - vec2<f32>(0.5);
    let dist = length(center);
    let vig = smoothstep(radius, radius - 0.3, dist);
    return mix(1.0 - intensity, 1.0, vig);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Sample the screen texture
    var color = textureSample(screen_texture, texture_sampler, uv);

    // Apply film grain
    let grain = film_grain(uv, settings.time, settings.grain_intensity, settings.grain_speed, settings.grain_coarseness);
    color = vec4<f32>(color.rgb + vec3<f32>(grain), color.a);

    // Apply CRT scanlines
    let scan = scanlines(uv, settings.scanline_count, settings.scanline_intensity);
    color = vec4<f32>(color.rgb * scan, color.a);

    // Apply vignette
    let vig = vignette(uv, settings.vignette_intensity, settings.vignette_radius);
    color = vec4<f32>(color.rgb * vig, color.a);

    // Clamp to valid range
    color = clamp(color, vec4<f32>(0.0), vec4<f32>(1.0));

    return color;
}
