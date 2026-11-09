//! Procedural shaders for the six selected bodies and Earth's moon.
//!
//! None of these shaders reads an image. Every visible detail is generated
//! from the surface sample and mathematical functions.

use crate::math::Vec3;

/// One invocation's view of the surface of a sphere.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceSample {
    /// Point on the unit sphere, also useful as a procedural coordinate.
    pub position: Vec3,
    /// Outward-facing unit normal at the surface point.
    pub normal: Vec3,
    /// Direction from the surface toward the camera.
    pub view_direction: Vec3,
    /// Direction from the surface toward our single light source.
    pub light_direction: Vec3,
    /// Diffuse lighting term: `normal dot light_direction`, clamped to 0..1.
    pub diffuse: f32,
    /// Edge factor: near 0 in the center and near 1 at the silhouette.
    pub rim: f32,
    /// Vertical coordinate from the south pole (0) to the north pole (1).
    pub latitude: f32,
    /// Wrapped horizontal coordinate around the sphere, from 0 to 1.
    pub longitude: f32,
}

/// A shader maps surface information to a color.
pub trait Shader {
    fn shade(&self, sample: SurfaceSample) -> Vec3;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Planet {
    Sun,
    Venus,
    Neptune,
    Earth,
    Saturn,
    Uranus,
}

impl Planet {
    pub const ALL: [Self; 6] = [
        Self::Sun,
        Self::Venus,
        Self::Neptune,
        Self::Earth,
        Self::Saturn,
        Self::Uranus,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Sun => "Sol",
            Self::Venus => "Venus",
            Self::Neptune => "Neptuno",
            Self::Earth => "Tierra",
            Self::Saturn => "Saturno",
            Self::Uranus => "Urano",
        }
    }

    pub fn file_stem(self) -> &'static str {
        match self {
            Self::Sun => "sun",
            Self::Venus => "venus",
            Self::Neptune => "neptune",
            Self::Earth => "earth",
            Self::Saturn => "saturn",
            Self::Uranus => "uranus",
        }
    }

    pub fn is_saturn(self) -> bool {
        matches!(self, Self::Saturn)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "sol" | "sun" => Some(Self::Sun),
            "venus" => Some(Self::Venus),
            "neptuno" | "neptune" => Some(Self::Neptune),
            "tierra" | "earth" => Some(Self::Earth),
            "saturno" | "saturn" => Some(Self::Saturn),
            "urano" | "uranus" => Some(Self::Uranus),
            _ => None,
        }
    }

    pub fn next(self, direction: i32) -> Self {
        let current_index = Self::ALL
            .iter()
            .position(|planet| *planet == self)
            .unwrap_or(0);
        let length = Self::ALL.len() as i32;
        let next_index = (current_index as i32 + direction).rem_euclid(length) as usize;
        Self::ALL[next_index]
    }
}

pub struct CelestialShader {
    pub planet: Planet,
}

impl CelestialShader {
    pub fn new(planet: Planet) -> Self {
        Self { planet }
    }
}

impl Shader for CelestialShader {
    fn shade(&self, sample: SurfaceSample) -> Vec3 {
        match self.planet {
            Planet::Sun => shade_sun(sample),
            Planet::Venus => shade_venus(sample),
            Planet::Neptune => shade_neptune(sample),
            Planet::Earth => shade_earth(sample),
            Planet::Saturn => shade_saturn(sample),
            Planet::Uranus => shade_uranus(sample),
        }
    }
}

/// Moon: a procedural gray surface with maria, crater fields and a cold rim.
/// It is rendered as a second sphere beside the Earth; it does not use a
/// texture or material.
pub struct MoonShader;

impl Shader for MoonShader {
    fn shade(&self, sample: SurfaceSample) -> Vec3 {
        let position = sample.position;

        // Layer 1: a cool gray base gradient.
        let base = Vec3::new(0.20, 0.22, 0.25).lerp(
            Vec3::new(0.66, 0.68, 0.72),
            (sample.latitude * 0.65 + 0.18).clamp(0.0, 1.0),
        );

        // Layer 2: broad maria generated from low-frequency waves.
        let maria_signal = procedural_noise(position * 2.4 + Vec3::new(2.0, 0.0, 4.0));
        let maria_mask = smoothstep(0.46, 0.66, maria_signal);
        let with_maria = base.lerp(Vec3::new(0.10, 0.12, 0.15), maria_mask * 0.48);

        // Layer 3: crater-like surface variation at a smaller scale.
        let crater_signal = procedural_noise(position * 7.0 + Vec3::new(5.0, 1.0, 3.0));
        let crater_mask = smoothstep(0.58, 0.80, crater_signal);
        let with_craters = with_maria.lerp(Vec3::new(0.78, 0.80, 0.84), crater_mask * 0.28);

        // Layer 4: diffuse lighting and a subtle atmospheric-looking rim.
        let lit = apply_diffuse(with_craters, sample, 0.22);
        lit + Vec3::new(0.34, 0.42, 0.58) * sample.rim.powf(2.8) * 0.16
    }
}

/// Sol: hot base color, convective cells, sunspots and a warm rim glow.
fn shade_sun(sample: SurfaceSample) -> Vec3 {
    let position = sample.position;

    // Layer 1: a yellow-orange temperature gradient.
    let temperature =
        (sample.latitude * 0.72 + procedural_noise(position * 2.0) * 0.28).clamp(0.0, 1.0);
    let cool_orange = Vec3::new(0.75, 0.08, 0.005);
    let hot_yellow = Vec3::new(1.0, 0.62, 0.035);
    let base = cool_orange.lerp(hot_yellow, temperature);

    // Layer 2: convection cells made from overlapping sine waves.
    let convection =
        ((position.y * 18.0 + position.x * 5.0 + position.z * 3.0).sin() * 0.5 + 0.5).powf(0.7);
    let convection_color = Vec3::new(1.0, 0.86, 0.18);
    let with_convection = base.lerp(convection_color, convection * 0.22);

    // Layer 3: broad plasma filaments. They add activity without creating
    // isolated circular marks on the surface.
    let filament_signal =
        (position.y * 16.0 + (position.x * 6.0).sin() * 1.4 + position.z * 4.0).sin() * 0.5 + 0.5;
    let filaments = smoothstep(0.52, 0.80, filament_signal);
    let with_filaments = with_convection.lerp(Vec3::new(1.0, 0.75, 0.08), filaments * 0.20);

    // Layer 4: emissive edge glow. The Sun does not need normal lighting to
    // remain visible because it is the light source in our scene.
    let glow = edge_glow(sample);
    with_filaments + Vec3::new(1.0, 0.34, 0.018) * glow * 0.42
}

/// Venus: pale golden body, warm cloudy bands, streaks and atmosphere.
fn shade_venus(sample: SurfaceSample) -> Vec3 {
    let position = sample.position;

    // Layer 1: cream-to-gold base gradient.
    let base = Vec3::new(0.53, 0.29, 0.08).lerp(Vec3::new(0.98, 0.72, 0.30), sample.latitude);

    // Layer 2: thick horizontal cloud bands.
    let band_signal = (sample.latitude * 22.0 + position.z * 2.3).sin() * 0.5 + 0.5;
    let cloud_bands = smoothstep(0.33, 0.72, band_signal);
    let cloudy_color = Vec3::new(1.0, 0.88, 0.56);
    let with_bands = base.lerp(cloudy_color, cloud_bands * 0.48);

    // Layer 3: diagonal-looking atmospheric streaks from a second wave.
    let streak_signal = (position.x * 13.0
        + position.y * 4.0
        + position.z * 8.0
        + sample.longitude * std::f32::consts::TAU)
        .sin()
        * 0.5
        + 0.5;
    let streaks = smoothstep(0.58, 0.82, streak_signal);
    let with_streaks = with_bands.lerp(Vec3::new(0.73, 0.45, 0.16), streaks * 0.24);

    // Layer 4: diffuse light plus a soft golden atmosphere at the silhouette.
    let lit = apply_diffuse(with_streaks, sample, 0.18);
    lit + Vec3::new(1.0, 0.68, 0.23) * sample.rim.powf(2.5) * 0.22
}

/// Neptune: blue base, latitude bands, a Great Dark Spot and cold rim light.
fn shade_neptune(sample: SurfaceSample) -> Vec3 {
    let position = sample.position;

    // Layer 1: deep blue base.
    let base = Vec3::new(0.015, 0.05, 0.28).lerp(
        Vec3::new(0.08, 0.34, 0.88),
        (sample.latitude * 0.65 + 0.16).clamp(0.0, 1.0),
    );

    // Layer 2: subtle fast atmospheric bands.
    let bands = (sample.latitude * 25.0 + position.x * 2.0).sin() * 0.5 + 0.5;
    let band_mask = smoothstep(0.42, 0.64, bands);
    let with_bands = base.lerp(Vec3::new(0.15, 0.46, 0.96), band_mask * 0.32);

    // Layer 3: broad cloud-density variations inside the bands. This gives
    // Neptune atmospheric activity without introducing an isolated dark spot.
    let cloud_density = procedural_noise(position * 3.0 + Vec3::new(4.0, 1.0, 7.0));
    let cloud_mask = smoothstep(0.42, 0.76, cloud_density);
    let with_clouds = with_bands.lerp(Vec3::new(0.20, 0.50, 0.98), cloud_mask * 0.18);

    // Layer 4: blue atmospheric rim.
    let lit = apply_diffuse(with_clouds, sample, 0.14);
    lit + Vec3::new(0.16, 0.55, 1.0) * sample.rim.powf(2.7) * 0.36
}

/// Earth: ocean, procedural land, terrain variation, clouds and atmosphere.
fn shade_earth(sample: SurfaceSample) -> Vec3 {
    let position = sample.position;

    // Layer 1: blue ocean with a mild latitude gradient.
    let ocean = Vec3::new(0.008, 0.035, 0.20).lerp(
        Vec3::new(0.015, 0.24, 0.72),
        (sample.latitude * 0.8 + 0.1).clamp(0.0, 1.0),
    );

    // Layer 2: continents from low-frequency procedural noise.
    let terrain = continental_noise(position);
    let continent_mask = smoothstep(0.54, 0.70, terrain);
    let land_color = Vec3::new(0.12, 0.30, 0.045).lerp(
        Vec3::new(0.52, 0.36, 0.12),
        procedural_noise(position * 1.8),
    );
    let with_land = ocean.lerp(land_color, continent_mask);

    // Layer 3: smaller terrain details break up the continents.
    let terrain_detail = procedural_noise(position * 3.4);
    let highland_mask = continent_mask * smoothstep(0.62, 0.83, terrain_detail);
    let with_terrain = with_land.lerp(Vec3::new(0.76, 0.67, 0.28), highland_mask * 0.34);

    // Layer 4: white cloud systems generated independently from the land.
    let cloud_signal = cloud_noise(position);
    let cloud_mask = smoothstep(0.58, 0.76, cloud_signal) * (0.72 + sample.rim * 0.28);
    let with_clouds = with_terrain.lerp(Vec3::new(0.94, 0.97, 1.0), cloud_mask * 0.72);

    // Layer 5: blue atmosphere concentrated around the silhouette.
    let lit = apply_diffuse(with_clouds, sample, 0.12);
    lit + Vec3::new(0.12, 0.44, 1.0) * sample.rim.powf(3.0) * 0.42
}

/// Saturn: golden horizontal bands, turbulent belts, storms and a rim glow.
fn shade_saturn(sample: SurfaceSample) -> Vec3 {
    let position = sample.position;

    // Layer 1: broad golden latitude bands.
    let bands = (sample.latitude * 15.0).sin() * 0.5 + 0.5;
    let base = Vec3::new(0.40, 0.22, 0.08).lerp(
        Vec3::new(0.91, 0.67, 0.31),
        bands * 0.72 + sample.latitude * 0.18,
    );

    // Layer 2: soft turbulent variations inside the bands.
    let turbulence = procedural_noise(position * 5.0);
    let with_turbulence = base.lerp(Vec3::new(0.98, 0.78, 0.44), turbulence * 0.18);

    // Layer 3: a lighter equatorial zone.
    let equator = 1.0 - smoothstep(0.08, 0.38, sample.normal.y.abs());
    let with_equator = with_turbulence.lerp(Vec3::new(0.96, 0.80, 0.52), equator * 0.28);

    // Layer 4: lighting and a warm edge.
    let lit = apply_diffuse(with_equator, sample, 0.16);
    lit + Vec3::new(1.0, 0.72, 0.33) * sample.rim.powf(2.8) * 0.20
}

/// Uranus: cyan base, very subtle bands, polar haze and a cool rim.
fn shade_uranus(sample: SurfaceSample) -> Vec3 {
    let position = sample.position;

    // Layer 1: cyan body gradient.
    let base = Vec3::new(0.04, 0.34, 0.42)
        .lerp(Vec3::new(0.30, 0.82, 0.82), sample.latitude * 0.65 + 0.15);

    // Layer 2: Uranus has much less visible banding than Neptune.
    let bands = (sample.latitude * 17.0 + position.z).sin() * 0.5 + 0.5;
    let with_bands = base.lerp(Vec3::new(0.62, 0.90, 0.86), bands * 0.10);

    // Layer 3: pale polar haze.
    let polar_mask = smoothstep(0.58, 0.96, sample.normal.y.abs());
    let with_poles = with_bands.lerp(Vec3::new(0.70, 0.94, 0.89), polar_mask * 0.42);

    // Layer 4: diffuse lighting and cool atmospheric rim.
    let lit = apply_diffuse(with_poles, sample, 0.16);
    lit + Vec3::new(0.32, 0.92, 0.92) * sample.rim.powf(2.6) * 0.23
}

fn apply_diffuse(color: Vec3, sample: SurfaceSample, ambient: f32) -> Vec3 {
    let recomputed_diffuse = sample.normal.dot(sample.light_direction).max(0.0);
    let diffuse = (sample.diffuse + recomputed_diffuse) * 0.5;
    color * (ambient + diffuse * (1.0 - ambient))
}

/// A small deterministic noise substitute made only from sine waves.
///
/// It is not random and it is not a texture. The same point always produces
/// the same value, which makes it ideal for repeatable procedural patterns.
fn procedural_noise(position: Vec3) -> f32 {
    let low = (position.x * 1.7 + position.y * 2.1 + position.z * 1.3).sin();
    let medium = (position.x * 3.8 - position.y * 2.9 + position.z * 3.4).sin();
    let high = (position.x * 7.5 + position.y * 6.2 - position.z * 5.7).sin();
    let value = low * 0.55 + medium * 0.30 + high * 0.15;
    value * 0.5 + 0.5
}

/// Lower-frequency waves are better suited for continent-sized shapes.
fn continental_noise(position: Vec3) -> f32 {
    let large = (position.x * 1.25 + position.y * 0.85 - position.z * 1.10).sin();
    let medium = (position.x * 2.05 - position.y * 1.70 + position.z * 0.90).sin();
    let shape = (position.x * 1.10 + position.y * 2.00 + position.z * 1.80).sin();
    (large * 0.52 + medium * 0.30 + shape * 0.18) * 0.5 + 0.5
}

/// Cloud noise uses a few broad waves, avoiding a dotted texture-like result.
fn cloud_noise(position: Vec3) -> f32 {
    let broad = (position.x * 3.6 + position.y * 2.7 + position.z * 4.1).sin();
    let stretched = (position.x * 6.4 - position.y * 2.0 + position.z * 3.2).sin();
    let diagonal = (position.x * 2.2 + position.y * 7.0 - position.z * 5.3).sin();
    (broad * 0.50 + stretched * 0.25 + diagonal * 0.25) * 0.5 + 0.5
}

fn edge_glow(sample: SurfaceSample) -> f32 {
    let facing = sample.normal.dot(sample.view_direction).max(0.0);
    (1.0 - facing).powf(2.0)
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
