"""Offline, journalled asset relocation for saved Forge projects (Python 3.10+).
Close all editors/players before applying or rolling back. Dry-run is default.
"""
import argparse, hashlib, json, os, re, tempfile, uuid
from pathlib import Path

MAX_FILE = 128 * 1024 * 1024
MAX_DOC = 32 * 1024 * 1024
MAX_TOTAL = 256 * 1024 * 1024
FIELDS = {'sprite', 'normal_map', 'particle_texture', 'path', 'image'}
CACHE = {'.forge-assets.json', '.forge-import-cache-v1.json'}

def digest(data): return hashlib.sha256(data).hexdigest()
def read(path, limit=MAX_FILE):
    with path.open('rb') as f: data = f.read(limit + 1)
    if len(data) > limit: raise ValueError(f'File too large: {path}')
    return data

def checked(root, rel):
    if not rel or '\\' in rel or any(p in ('', '.', '..') or p.endswith(('.', ' ')) or re.search(r'[<>:"|?*\x00-\x1f]', p) or p.split('.')[0].upper() in {'CON','PRN','AUX','NUL',*[f'COM{i}' for i in range(1,10)],*[f'LPT{i}' for i in range(1,10)]} for p in rel.split('/')):
        raise ValueError(f'Unsafe path: {rel}')
    path = root.joinpath(*rel.split('/'))
    # Reject symlinks and junctions, including aliases inside the root.
    if path.resolve() != path: raise ValueError(f'Linked path: {rel}')
    if not path.is_relative_to(root): raise ValueError('Path outside project')
    return path

def json_read(raw):
    def pairs(items):
        out = {}
        for k,v in items:
            if k in out: raise ValueError(f'Duplicate JSON key: {k}')
            out[k] = v
        return out
    return json.loads(raw, object_pairs_hook=pairs, parse_constant=lambda x: (_ for _ in ()).throw(ValueError(f'Invalid JSON value: {x}')))

def encoded(obj): return (json.dumps(obj, ensure_ascii=False, indent=2, allow_nan=False)+'\n').encode('utf-8')

def rewrite(obj, old, new, key=None):
    if isinstance(obj, dict):
        if any(old in k for k in obj): raise ValueError('Asset path in JSON key; manual review required')
        return {k:rewrite(v,old,new,k) for k,v in obj.items()}
    if isinstance(obj, list): return [rewrite(v,old,new,key) for v in obj]
    if isinstance(obj, str):
        if obj == old and key in FIELDS: return new
        if old.casefold() in obj.replace('\\','/').casefold(): raise ValueError('Unrecognized or embedded asset reference; manual review required')
    return obj

def plan(root, old, new, allow_dynamic=False):
    root = root.resolve(strict=True)
    if not old.startswith('assets/') or not new.startswith('assets/'): raise ValueError('Paths must begin with assets/')
    src,dst=checked(root,old),checked(root,new)
    if old.casefold()==new.casefold(): raise ValueError('Case-only or identical relocation is not supported')
    if src.suffix.lower()!=dst.suffix.lower() or src.suffix.lower() not in {'.png','.jpg','.jpeg','.wav','.ogg'}: raise ValueError('Keep supported original extension')
    if not src.is_file() or dst.exists() or not dst.parent.is_dir(): raise ValueError('Need regular source, unused destination and existing destination folder')
    if not (root/'assets').is_dir(): raise ValueError('Missing assets directory')
    asset=read(src); changes=[]; warnings=[]; total=len(asset); seen=0
    def scan_error(exc): raise exc
    for directory,dirs,files in os.walk(root, followlinks=False, onerror=scan_error):
        d=Path(directory)
        dirs[:]=[name for name in dirs if not name.startswith('.') and name not in {'target','__pycache__'}]
        for name in dirs:
            p=d/name
            if p.resolve()!=p: raise ValueError(f'Linked directory: {p}')
        for name in sorted(files):
            seen+=1
            if seen>32768: raise ValueError('Scan exceeds 32768 files')
            p=d/name; rel=p.relative_to(root).as_posix()
            if name.startswith('.') or name.endswith('.bak'): continue
            if p.suffix.lower()=='.lua':
                warnings.append(rel); raw=read(checked(root,rel),MAX_DOC); total+=len(raw)
                if total>MAX_TOTAL: raise ValueError("Scan exceeds 256 MiB")
                if old.casefold() in raw.decode('utf-8',errors='replace').replace('\\','/').casefold(): raise ValueError(f'Literal Lua reference in {rel}; update manually first')
                continue
            if not name.lower().endswith('.json'): continue
            raw=read(checked(root,rel),MAX_DOC); total+=len(raw)
            if total>MAX_TOTAL: raise ValueError('Scan exceeds 256 MiB')
            obj=json_read(raw)
            # Only documents owned by this root, plus generated atlas sidecars.
            known=(name.endswith('.forge.json') and p.parent==root) or (name.endswith('.prefab.json') and rel.startswith('prefabs/')) or ((name.endswith('.atlas.json') or re.fullmatch(r'atlas-state-[0-9a-fA-F-]{36}\.json',name)) and rel.startswith('assets/'))
            if not known:
                if old.casefold() in json.dumps(obj,ensure_ascii=False).replace('\\\\','/').casefold(): raise ValueError(f'Unrecognized JSON reference: {rel}')
                continue
            if 'script' in raw.decode('utf-8',errors='replace') and name.endswith(('.forge.json','.prefab.json')):
                def has_script(x):
                    if isinstance(x,dict): return bool(x.get('script')) or any(has_script(v) for v in x.values())
                    if isinstance(x,list): return any(has_script(v) for v in x)
                    return False
                if has_script(obj): warnings.append(rel)
            new_obj=rewrite(obj,old,new)
            if new_obj!=obj: changes.append((rel,raw,encoded(new_obj)))
    # Packed atlas sidecars named after the image must move together.
    side=src.with_suffix('.atlas.json'); dest_side=dst.with_suffix('.atlas.json')
    if side.exists():
        side_rel=side.relative_to(root).as_posix(); dest_rel=dest_side.relative_to(root).as_posix()
        if dest_side.exists(): raise ValueError('Destination atlas sidecar exists')
        data=read(checked(root,side_rel),MAX_DOC); obj=json_read(data); result=encoded(rewrite(obj,old,new))
        changes=[c for c in changes if c[0]!=side_rel]
        side_move=(side_rel,dest_rel,data,result)
    else: side_move=None
    if warnings and not allow_dynamic: raise ValueError('Dynamic script references cannot be proven safe. Review scripts, then explicitly use --allow-dynamic. Examples: '+', '.join(warnings[:6]))
    return root,asset,changes,side_move,warnings

def sync_write(path,data,exclusive=False):
    fd,tmp=tempfile.mkstemp(prefix='.forge-relocate-',dir=path.parent)
    try:
        with os.fdopen(fd,'wb') as f: f.write(data); f.flush(); os.fsync(f.fileno())
        if exclusive: os.link(tmp,path)  # atomic publication without overwriting; NTFS/ext4
        else: os.replace(tmp,path)
    finally:
        if os.path.exists(tmp): os.unlink(tmp)

def plan_digest(old,new,asset,changes,side,warnings):
    # Deterministic approval fingerprint; a changed asset or rewritten document invalidates approval.
    summary=[old,new,digest(asset),sorted((rel,digest(a),digest(b)) for rel,a,b in changes),None if side is None else (side[0],side[1],digest(side[2]),digest(side[3])),sorted(warnings)]
    return digest(encoded(summary))

def apply(root,old,new,allow_dynamic=False,fail_after=None,expected_plan=None):
    root,asset,changes,side,warnings=plan(root,old,new,allow_dynamic)
    if expected_plan is not None and expected_plan!=plan_digest(old,new,asset,changes,side,warnings): raise RuntimeError('Relocation plan changed after approval; run dry run again')
    ops=[(new,None,asset)]+[(rel,a,b) for rel,a,b in changes]
    if side: ops.extend([(side[1],None,side[3]),(side[0],side[2],None)])
    for cache in sorted(CACHE):
        p=checked(root,cache)
        if p.exists(): ops.append((cache,read(p,MAX_DOC),None))
    ops.append((old,asset,None))  # remove original last
    return commit_ops(root,ops,fail_after)

def commit_ops(root,ops,fail_after=None):
    # Shared transaction journal for asset moves and prefab subtree transfers.
    root=root.resolve(strict=True)
    if len({rel.casefold() for rel,_,_ in ops})!=len(ops): raise ValueError('Duplicate transaction path')
    for rel,before,after in ops:
        p=checked(root,rel); current=read(p) if p.exists() else None
        if current!=before: raise RuntimeError(f'Concurrent change: {rel}')
        if after is not None and len(after)>MAX_FILE: raise ValueError('Transaction output too large')
    journal=root/('.forge-relocation-'+uuid.uuid4().hex); journal.mkdir()
    entries=[]
    for i,(rel,before,after) in enumerate(ops):
        for tag,data in [('before',before),('after',after)]:
            if data is not None: sync_write(journal/f'{i}.{tag}',data,True)
        entries.append({'path':rel,'before':None if before is None else digest(before),'after':None if after is None else digest(after)})
    sync_write(journal/'journal.json',encoded({'state':'prepared','entries':entries}),True)
    try:
        for i,(rel,before,after) in enumerate(ops):
            p=checked(root,rel); current=read(p) if p.exists() else None
            if current!=before: raise RuntimeError(f'Concurrent change: {rel}')
            if after is None: p.unlink()
            else: sync_write(p,after,exclusive=before is None)
            if fail_after==i: raise RuntimeError('Injected test failure')
        sync_write(journal/'journal.json',encoded({'state':'committed','entries':entries}))
    except Exception:
        rollback(root,journal)
        raise
    return journal

def rollback(root,journal):
    root=root.resolve(strict=True); journal=journal.resolve(strict=True)
    if journal.parent!=root or not journal.name.startswith('.forge-relocation-'): raise ValueError('Invalid journal path')
    record=json_read(read(journal/'journal.json',MAX_DOC)); entries=record['entries']
    # Preflight entire rollback; refuse to overwrite unrelated edits.
    pending=[]
    for i,e in enumerate(entries):
        p=checked(root,e['path']); current=read(p) if p.exists() else None; h=None if current is None else digest(current)
        if h not in (e['before'],e['after']): raise RuntimeError(f'Rollback conflict: {p}; backups retained in {journal}')
        before=None if e['before'] is None else read(journal/f'{i}.before')
        if before is not None and digest(before)!=e['before']: raise RuntimeError('Backup hash mismatch')
        pending.append((p,h,e['before'],before))
    for p,h,expected,before in reversed(pending):
        if h==expected: continue
        if before is None: p.unlink()
        else: sync_write(p,before,exclusive=h is None)
    record['state']='rolled_back';sync_write(journal/'journal.json',encoded(record))

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('root',type=Path);p.add_argument('old',nargs='?');p.add_argument('new',nargs='?');p.add_argument('--apply',action='store_true');p.add_argument('--allow-dynamic',action='store_true');p.add_argument('--rollback',type=Path);p.add_argument('--expect-plan');a=p.parse_args()
    if a.rollback: rollback(a.root,a.rollback);print('Rollback complete');return
    if not a.old or not a.new: p.error('old and new asset paths are required')
    _,asset,changes,side,warnings=plan(a.root,a.old,a.new,a.allow_dynamic)
    print("Plan SHA256:",plan_digest(a.old,a.new,asset,changes,side,warnings))
    print('Documents:',*[c[0] for c in changes],sep='\n  ');print('Atlas sidecar:',bool(side));print('Dynamic references reviewed/acknowledged:',len(warnings))
    if a.apply: print('Committed. Recovery journal:',apply(a.root,a.old,a.new,a.allow_dynamic,expected_plan=a.expect_plan))
    else: print('Dry run only. Close the editor and repeat with --apply to commit.')
if __name__=='__main__':
    try: main()
    except (OSError,ValueError,RuntimeError) as exc: raise SystemExit(str(exc))
