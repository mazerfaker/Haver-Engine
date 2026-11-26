#import bevy_sprite::mesh2d_vertex_output::VertexOutput
struct LightParams {
 tint:vec4<f32>, uv:vec4<f32>, geometry:vec4<f32>, rotation:vec4<f32>, light:vec4<f32>, light_color:vec4<f32>, blockers:array<vec4<f32>,4>,
};
@group(2) @binding(0) var<uniform> p:LightParams;
@group(2) @binding(1) var albedo:texture_2d<f32>;
@group(2) @binding(2) var albedo_sampler:sampler;
@group(2) @binding(3) var normals:texture_2d<f32>;
@group(2) @binding(4) var normal_sampler:sampler;
fn rotate(v:vec2<f32>)->vec2<f32>{return vec2<f32>(p.rotation.x*v.x-p.rotation.y*v.y,p.rotation.y*v.x+p.rotation.x*v.y);}
fn occluded(a:vec2<f32>,b:vec2<f32>,bounds:vec4<f32>)->bool{
 if bounds.z<=bounds.x || bounds.w<=bounds.y {return false;}
 let d=b-a;
 if (abs(d.x)<0.00001 && (a.x<bounds.x||a.x>bounds.z)) || (abs(d.y)<0.00001 && (a.y<bounds.y||a.y>bounds.w)){return false;}
 let inv=vec2<f32>(1.0)/select(vec2<f32>(0.00001),d,abs(d)>vec2<f32>(0.00001));
 let t0=(bounds.xy-a)*inv;let t1=(bounds.zw-a)*inv;let lo=min(t0,t1);let hi=max(t0,t1);
 return max(max(lo.x,lo.y),0.001)<min(min(hi.x,hi.y),0.999);
}
@fragment fn fragment(mesh:VertexOutput)->@location(0) vec4<f32>{
 let uv=mix(p.uv.xy,p.uv.zw,mesh.uv);
 let base=textureSample(albedo,albedo_sampler,uv)*p.tint;
 let raw=textureSample(normals,normal_sampler,uv).xyz*2.0-1.0;
 let n=normalize(vec3<f32>(rotate(vec2<f32>(raw.x,-raw.y)),max(raw.z,0.001)));
 let local=vec2<f32>(mesh.uv.x-0.5,0.5-mesh.uv.y)*p.geometry.zw;
 let world=p.geometry.xy+rotate(local);let delta=p.light.xy-world;
 var shade=vec3<f32>(p.rotation.z);
 if p.light.z>0.0 {
  var visible=1.0;for(var i:u32=0u;i<4u;i=i+1u){if occluded(world,p.light.xy,p.blockers[i]){visible=0.0;}}
  let diffuse=max(dot(n,normalize(vec3<f32>(delta,p.light.z*0.25))),0.0);
  let attenuation=max(0.0,1.0-length(delta)/p.light.z);
  shade+=p.light_color.xyz*(diffuse*attenuation*visible*p.light.w);
 }
 return vec4<f32>(base.rgb*clamp(shade,vec3<f32>(0.0),vec3<f32>(1.0)),base.a);
}
