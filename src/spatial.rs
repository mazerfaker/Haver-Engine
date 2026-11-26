//! Conservative 2D bounds and chunk indexing for future batched rendering.
use std::collections::{BTreeMap,BTreeSet};
pub fn cells(center:[f32;2],half:[f32;2],cell:f32)->Option<([i32;2],[i32;2])>{
 if !cell.is_finite()||cell<=0.||center.iter().chain(half.iter()).any(|v|!v.is_finite())||half.iter().any(|v|*v<0.){return None;}
 let lo=std::array::from_fn(|i|((center[i]-half[i])/cell).floor() as i32);
 let hi=std::array::from_fn(|i|((center[i]+half[i])/cell).floor() as i32);
 if (0..2).any(|i|i64::from(hi[i])-i64::from(lo[i])>128){return None;} Some((lo,hi))
}
#[derive(Default)] pub struct SpatialIndex { cells:BTreeMap<(i32,i32),Vec<usize>>, large:Vec<usize> }
impl SpatialIndex {
 pub fn insert(&mut self,id:usize,center:[f32;2],half:[f32;2],cell:f32){if let Some((lo,hi))=cells(center,half,cell){for x in lo[0]..=hi[0]{for y in lo[1]..=hi[1]{self.cells.entry((x,y)).or_default().push(id);}}}else{self.large.push(id);}}
 /// Bounded, deterministic query; callers must fall back to a full scan on false.
 pub fn query_into(&self,center:[f32;2],half:[f32;2],cell:f32,out:&mut Vec<usize>)->bool{out.clear();let Some((lo,hi))=cells(center,half,cell)else{return false;};if (0..2).any(|i|i64::from(hi[i])-i64::from(lo[i])>8){return false;}out.extend_from_slice(&self.large);for x in lo[0]..=hi[0]{for y in lo[1]..=hi[1]{if let Some(v)=self.cells.get(&(x,y)){out.extend_from_slice(v);}}}out.sort_unstable();out.dedup();true}
 pub fn insert_bounded(&mut self,id:usize,center:[f32;2],half:[f32;2],cell:f32){if let Some((lo,hi))=cells(center,half,cell){if (0..2).all(|i|i64::from(hi[i])-i64::from(lo[i])<=8){self.insert(id,center,half,cell);return;}}self.large.push(id);}
 pub fn query(&self,center:[f32;2],half:[f32;2],cell:f32)->Option<Vec<usize>>{let(lo,hi)=cells(center,half,cell)?;let mut hits:BTreeSet<_>=self.large.iter().copied().collect();for x in lo[0]..=hi[0]{for y in lo[1]..=hi[1]{if let Some(v)=self.cells.get(&(x,y)){hits.extend(v.iter().copied());}}}Some(hits.into_iter().collect())}
}
#[cfg(test)] mod tests{use super::*;#[test]fn negative_and_large(){let mut s=SpatialIndex::default();s.insert(1,[-32.,0.],[4.,4.],32.);s.insert(2,[0.,0.],[99999.,1.],32.);assert_eq!(s.query([-32.,0.],[1.,1.],32.).unwrap(),vec![1,2]);}}

/// Conservative rotated sprite bounds for a 2D camera; supports negative coordinates.
pub fn visible(center:[f32;2],size:[f32;2],rotation:f32,view_center:[f32;2],view_half:[f32;2])->bool {
 let (s,c)=rotation.to_radians().sin_cos();
 let half=[(c.abs()*size[0]+s.abs()*size[1])*0.5,(s.abs()*size[0]+c.abs()*size[1])*0.5];
 (0..2).all(|i|(center[i]-view_center[i]).abs()<=half[i]+view_half[i])
}
#[cfg(test)] mod visibility_tests {use super::*;#[test]fn bounds(){assert!(visible([12.,0.],[4.,20.],90.,[0.,0.],[3.,3.]));assert!(!visible([30.,0.],[4.,4.],0.,[0.,0.],[3.,3.]));}}

#[cfg(test)]mod bounded_tests{use super::*;#[test]fn deterministic_and_conservative(){let mut s=SpatialIndex::default();s.insert_bounded(1,[0.,0.],[300.,100.],256.);s.insert_bounded(0,[10.,0.],[1.,1.],256.);s.insert_bounded(2,[0.,0.],[100000.,100.],256.);let mut hits=vec![];assert!(s.query_into([0.,0.],[300.,300.],256.,&mut hits));assert_eq!(hits,vec![0,1,2]);assert!(!s.query_into([0.,0.],[100000.,100000.],256.,&mut hits));assert!(hits.is_empty());}}
