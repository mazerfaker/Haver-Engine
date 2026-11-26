#import bevy_sprite::mesh2d_vertex_output::VertexOutput
@group(2) @binding(0) var<uniform> light: vec4<f32>;
@group(2) @binding(1) var normals: texture_2d<f32>;
@group(2) @binding(2) var normal_sampler: sampler;
// Segment vs fixed circular occluder. Dynamic light, analytic hard shadow.
fn visibility(a:vec2<f32>,b:vec2<f32>)->f32 {
 let center=vec2<f32>(0.52,0.48); let d=b-a;
 let t=clamp(dot(center-a,d)/max(dot(d,d),0.00001),0.0,1.0);
 return select(1.0,0.0,length(a+t*d-center)<0.075);
}
@fragment fn fragment(mesh:VertexOutput)->@location(0) vec4<f32>{
 let uv=mesh.uv;
 let normal=normalize(textureSample(normals,normal_sampler,uv).xyz*2.0-1.0);
 let delta=light.xy-uv;
 let diffuse=max(dot(normal,normalize(vec3<f32>(delta,0.25))),0.0);
 let attenuation=max(0.0,1.0-length(delta)/max(light.z,0.001));
 let lit=light.w+diffuse*attenuation*visibility(uv,light.xy);
 let grid=0.85+0.15*cos(uv.x*110.0)*cos(uv.y*110.0);
 if length(uv-vec2<f32>(0.52,0.48))<0.075 {return vec4<f32>(0.045,0.055,0.07,1.0);}
 return vec4<f32>(vec3<f32>(0.42,0.65,0.92)*lit*grid,1.0);
}
