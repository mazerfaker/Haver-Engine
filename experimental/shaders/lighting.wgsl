// Standalone prototype. Not registered in Bevy's render graph.
struct LightParams { position: vec2<f32>, radius: f32, intensity: f32, color: vec4<f32>, ambient: vec4<f32> }
@group(0) @binding(0) var<uniform> light: LightParams;
@group(0) @binding(1) var albedo: texture_2d<f32>;
@group(0) @binding(2) var normals: texture_2d<f32>;
@group(0) @binding(3) var shadow: texture_2d<f32>;
@group(0) @binding(4) var nearest_sampler: sampler;
@fragment fn fragment(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
 let base = textureSample(albedo, nearest_sampler, uv);
 let n = normalize(textureSample(normals, nearest_sampler, uv).xyz * 2.0 - 1.0 + vec3<f32>(0.0,0.0,0.0001));
 let delta = light.position - uv;
 let direction = normalize(vec3<f32>(delta, 0.15));
 let diffuse = max(dot(n,direction), 0.0);
 let falloff = max(1.0 - length(delta)/max(light.radius,0.0001),0.0);
 let visibility = textureSample(shadow,nearest_sampler,uv).r;
 return vec4<f32>(base.rgb * (light.ambient.rgb + light.color.rgb * light.intensity * diffuse * falloff * visibility), base.a);
}
