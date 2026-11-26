// xy = position, z = phase/age, w = respawn counter. Fixed step per rendered frame (lab).
@group(0) @binding(0) var<storage,read_write> particles:array<vec4<f32>>;
@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) id:vec3<u32>){
 if id.x>=arrayLength(&particles){return;}
 var p=particles[id.x];let seed=f32(id.x)*0.61803399+p.w*0.12345;
 p.z+=0.006+0.008*fract(seed);p.x+=0.002*sin(seed*43.0+p.z*9.0);p.y-=0.005+0.002*fract(seed*17.0);
 if p.z>=1.0||p.y<0.03 {p=vec4<f32>(0.35+0.3*fract(seed*7.0),0.9,0.0,p.w+1.0);}
 particles[id.x]=p;
}
