#import bevy_sprite::mesh2d_functions as mesh_functions
@group(2) @binding(0) var<storage,read> particles:array<vec4<f32>>;
@group(2) @binding(1) var tex:texture_2d<f32>;
@group(2) @binding(2) var tex_sampler:sampler;
struct Vertex { @builtin(instance_index) instance_index:u32, @location(0) position:vec3<f32>, @location(2) uv:vec2<f32> };
struct Output { @builtin(position) position:vec4<f32>, @location(0) uv:vec2<f32>, @location(1) opacity:f32 };
@vertex fn vertex(v:Vertex)->Output {
 let slot=u32(floor(v.uv.x*0.5))+3u;
 let p=particles[min(slot,arrayLength(&particles)-1u)];
 let shape=particles[2];
 let local=vec3<f32>(p.x-0.5+v.position.x*shape.x,0.5-p.y+v.position.y*shape.y,0.0);
 var out:Output;
 out.position=mesh_functions::mesh2d_position_local_to_clip(mesh_functions::get_world_from_local(v.instance_index),vec4<f32>(local,1.0));
 out.uv=vec2<f32>(v.uv.x-f32(slot-3u)*2.0,v.uv.y);
 out.opacity=clamp(1.0-p.z/max(particles[0].z,0.00001),0.0,1.0);
 return out;
}
@fragment fn fragment(v:Output)->@location(0) vec4<f32> {
 let t=textureSample(tex,tex_sampler,v.uv);
 let color=particles[1];
 let edge=max(0.0,1.0-length((v.uv-0.5)*2.0));
 return vec4<f32>(t.rgb*color.rgb,t.a*edge*color.a*v.opacity);
}
