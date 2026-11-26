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
 pub fn query(&self,center:[f32;2],half:[f32;2],cell:f32)->Option<Vec<usize>>{let(lo,hi)=cells(center,half,cell)?;let mut hits:BTreeSet<_>=self.large.iter().copied().collect();for x in lo[0]..=hi[0]{for y in lo[1]..=hi[1]{if let Some(v)=self.cells.get(&(x,y)){hits.extend(v.iter().copied());}}}Some(hits.into_iter().collect())}
}
#[cfg(test)] mod tests{use super::*;#[test]fn negative_and_large(){let mut s=SpatialIndex::default();s.insert(1,[-32.,0.],[4.,4.],32.);s.insert(2,[0.,0.],[99999.,1.],32.);assert_eq!(s.query([-32.,0.],[1.,1.],32.).unwrap(),vec![1,2]);}}
