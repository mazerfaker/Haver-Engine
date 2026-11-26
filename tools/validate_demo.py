"""Static demo asset and geometry checks; not a Rust compilation test."""
import json, pathlib, uuid, wave, struct
root=pathlib.Path(__file__).resolve().parents[1]/'examples'/'signal_lost'
def validate(file):
 p=json.loads(file.read_text()); objs=p['objects']; ids=set(); names=set()
 for o in objs:
     uid=uuid.UUID(o['id']); assert uid not in ids; ids.add(uid)
     assert o['name'] not in names; names.add(o['name'])
     assert all(s>0 for s in o['size'])
     animations = ([o['animation']] if 'animation' in o else []) + list(o.get('clips', {}).values())
     for anim in animations:
         assert {'columns','rows','frames','fps','looping'} <= anim.keys(), (file.name, o['name'], 'missing animation fields')
         assert isinstance(anim['looping'], bool), (file.name, o['name'], 'invalid looping')
     for rel in [o.get('sprite'),o.get('production',{}).get('particle_texture'),o.get('production',{}).get('normal_map'),(o.get('pro',{}).get('sound') or {}).get('path')]:
         if not rel: continue
         path=root/rel; assert path.is_file() and path.resolve().is_relative_to(root.resolve())
         if path.suffix=='.png':
             raw=path.read_bytes(); assert raw[:8]==b'\x89PNG\r\n\x1a\n'; w,h=struct.unpack('>II',raw[16:24]); assert w and h
             if rel==o.get('sprite'):
                 for anim in [o.get('animation',{}),*o.get('clips',{}).values()]:
                     c,r=anim.get('columns',1),anim.get('rows',1); assert w%c==0 and h%r==0; assert anim.get('start',0)+anim.get('frames',1)<=c*r
         elif path.suffix=='.wav':
             with wave.open(str(path),'rb') as f: assert f.getnframes()>0
 assert sum(bool(o['player']) for o in objs)==1
 assert sum(o['name'].startswith('Core:') for o in objs)==3
 assert 'Exit' in names and any(o.get('camera') for o in objs)
 print(f'STATIC PASS [{file.name}]: {len(objs)} objects, unique IDs, 3 cores, assets and atlas ranges valid')
for file in sorted(root.glob("*.forge.json")):
 validate(file)

for cue in ("collect", "hurt", "win"):
 with wave.open(str(root/"assets"/(cue+".wav")), "rb") as f: assert f.getnframes()>0
