#import bevy_sprite::mesh2d_vertex_output::VertexOutput
@group(2) @binding(0) var<storage,read> particles:array<vec4<f32>>;
@fragment fn fragment(mesh:VertexOutput)->@location(0) vec4<f32>{
 var rgb=vec3<f32>(0.02,0.03,0.05);
 for(var i:u32=0u;i<arrayLength(&particles);i=i+1u){let p=particles[i];let d=length((mesh.uv-p.xy)*vec2<f32>(1.77778,1.0));let a=max(0.0,1.0-d/0.022)*(1.0-p.z);rgb+=vec3<f32>(1.0,0.38,0.07)*a*a;}
 return vec4<f32>(rgb,1.0);
}
