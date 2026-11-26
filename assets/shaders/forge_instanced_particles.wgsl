// Vertex positions are synthesized; instance_index is a hardware draw instance, not a mesh batch index.
@group(0) @binding(0) var<storage,read> particles:array<vec4<f32>>;
struct Out { @builtin(position) position:vec4<f32>, @location(0) uv:vec2<f32>, @location(1) opacity:f32 };
@vertex fn vertex(@builtin(vertex_index) v:u32,@builtin(instance_index) i:u32)->Out {
 let corners=array<vec2<f32>,6>(vec2<f32>(-1.0,-1.0),vec2<f32>(1.0,-1.0),vec2<f32>(1.0,1.0),vec2<f32>(-1.0,-1.0),vec2<f32>(1.0,1.0),vec2<f32>(-1.0,1.0));
 let p=particles[i];let q=corners[v];var o:Out;
 o.position=vec4<f32>(vec2<f32>(2.0*p.x-1.0,1.0-2.0*p.y)+q*vec2<f32>(0.014,0.025),0.0,1.0);
 o.uv=q;o.opacity=clamp(1.0-p.z,0.0,1.0);return o;
}
@fragment fn fragment(v:Out)->@location(0) vec4<f32>{let edge=max(0.0,1.0-length(v.uv));return vec4<f32>(1.0,0.38,0.07,edge*edge*v.opacity);}
