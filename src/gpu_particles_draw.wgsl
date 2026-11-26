#import bevy_sprite::mesh2d_vertex_output::VertexOutput
@group(2) @binding(0) var<storage,read> particles:array<vec4<f32>>;
@group(2) @binding(1) var tex:texture_2d<f32>;
@group(2) @binding(2) var tex_sampler:sampler;
@fragment fn fragment(mesh:VertexOutput)->@location(0) vec4<f32>{
 let config=particles[0];let color=particles[1];let shape=particles[2];var rgb=vec3<f32>(0.0);var alpha=0.0;
 for(var i:u32=3u;i<arrayLength(&particles);i=i+1u){let p=particles[i];let uv=(mesh.uv-p.xy)/shape.xy+vec2<f32>(0.5);if all(uv>=vec2<f32>(0.0))&&all(uv<=vec2<f32>(1.0)){let t=textureSampleLevel(tex,tex_sampler,uv,0.0);let edge=max(0.0,1.0-length((uv-0.5)*2.0));let a=t.a*edge*color.a*clamp(1.0-p.z/config.z,0.0,1.0);rgb=rgb*(1.0-a)+t.rgb*color.rgb*a;alpha=alpha+(1.0-alpha)*a;}}
 return vec4<f32>(rgb/max(alpha,0.00001),alpha);
}
