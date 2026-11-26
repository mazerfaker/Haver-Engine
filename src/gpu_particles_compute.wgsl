@group(0) @binding(0) var<storage,read_write> particles:array<vec4<f32>>;
@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) id:vec3<u32>){
 let i=id.x+3u;if i>=arrayLength(&particles){return;}
 let config=particles[0];let shape=particles[2];var p=particles[i];
 p.z+=1.0/60.0;p=vec4<f32>(p.xy+config.xy/60.0,p.z,p.w);
 if p.z>=config.z {let seed=fract(sin(f32(i)*12.9898+p.w*78.233)*43758.5453);let spread=vec2<f32>(shape.x,shape.y)*config.w;p=vec4<f32>(shape.zw+(vec2<f32>(seed,fract(seed*17.3))-0.5)*2.0*min(spread,vec2<f32>(0.45)),0.0,p.w+1.0);}
 particles[i]=p;
}
