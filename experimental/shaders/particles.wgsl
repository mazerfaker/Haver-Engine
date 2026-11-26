// Standalone compute prototype. Host dispatch and rendering are not integrated.
struct Particle { position: vec2<f32>, velocity: vec2<f32>, age: f32, lifetime: f32, active: u32, pad: u32 }
struct Params { gravity: vec2<f32>, dt: f32, count: u32 }
@group(0) @binding(0) var<storage,read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: Params;
@compute @workgroup_size(64) fn step(@builtin(global_invocation_id) id: vec3<u32>) {
 let i = id.x;
 if i >= min(params.count,arrayLength(&particles)) { return; }
 var p = particles[i];
 if p.active == 0u { return; }
 let dt = clamp(params.dt,0.0,0.033);
 p.age += dt;
 if p.age >= p.lifetime { p.active = 0u; } else { p.velocity += params.gravity*dt; p.position += p.velocity*dt; }
 particles[i] = p;
}
