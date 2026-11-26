#import bevy_sprite::mesh2d_vertex_output::VertexOutput
struct LightParams {
 tint:vec4<f32>, uv:vec4<f32>, geometry:vec4<f32>, rotation:vec4<f32>, light:array<vec4<f32>,3>, light_color:array<vec4<f32>,3>, blockers:array<vec4<f32>,12>,
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
 let world=p.geometry.xy+rotate(local);
 var shade=vec3<f32>(p.rotation.z);
 for(var li:u32=0u;li<3u;li=li+1u){let light=p.light[li];if light.z<=0.0{continue;}let delta=light.xy-world;
  let perpendicular=vec2<f32>(-delta.y,delta.x)/max(length(delta),0.001);var visible=0.0;
  let samples=select(1u,3u,p.rotation.w>0.0);
  for(var s:u32=0u;s<samples;s=s+1u){let offset=select(0.0,f32(s)-1.0,samples>1u);let endpoint=light.xy+perpendicular*offset*p.rotation.w;var hit=false;for(var i:u32=0u;i<4u;i=i+1u){if occluded(world,endpoint,p.blockers[li*4u+i]){hit=true;}}visible+=select(1.0,0.0,hit);}
  visible/=f32(samples);let diffuse=max(dot(n,normalize(vec3<f32>(delta,light.z*0.25))),0.0);
  let attenuation=max(0.0,1.0-length(delta)/light.z);shade+=p.light_color[li].xyz*(diffuse*attenuation*visible*light.w);
 }
 return vec4<f32>(base.rgb*clamp(shade,vec3<f32>(0.0),vec3<f32>(1.0)),base.a);
}
