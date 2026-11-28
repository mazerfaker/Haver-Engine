//! Conservative eligibility planning for future scene instancing. Does not change rendering.
use crate::Project;
use std::collections::BTreeMap;
#[derive(Debug,Default,serde::Serialize)]
pub struct InstancePlan { pub candidate_sprites: usize, pub candidate_groups: usize, pub fallback_objects: usize }
pub fn plan(project:&Project)->InstancePlan {
    let mut groups=BTreeMap::<(i32,&str),usize>::new();
    let mut result=InstancePlan::default();
    for o in &project.objects {
        if project.isometric || project.settings.gpu_lighting || o.camera || o.color[3]!=255 || o.pro.emitter.is_some() || o.pro.light.is_some() || o.sprite.is_none() {
            result.fallback_objects+=1; continue;
        }
        *groups.entry((o.layer,o.sprite.as_deref().unwrap())).or_default()+=1;
    }
    for n in groups.values().copied(){if n>=2{result.candidate_groups+=1;result.candidate_sprites+=n;}else{result.fallback_objects+=n;}}
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn default_scene_fallback(){let p=Project::default();assert_eq!(plan(&p).fallback_objects,p.objects.len());}
    #[test] fn isometric_fallback(){let mut p=Project::default();p.isometric=true;assert_eq!(plan(&p).candidate_sprites,0);}
}
