"""Portable, non-destructive project packaging. Python 3.10+. Pillow optional."""
import argparse, hashlib, io, json, pathlib, shutil, tempfile, zipfile

def pack(root, output, prune=False, optimize=False):
    root=pathlib.Path(root).resolve(); output=pathlib.Path(output).resolve()
    if output.exists(): raise FileExistsError(output)
    if not (root/'project.forge.json').is_file(): raise ValueError('Missing project.forge.json')
    files={p.relative_to(root).as_posix():p for p in root.rglob('*') if p.is_file() and not p.is_symlink() and p.resolve().is_relative_to(root) and (p.relative_to(root).parts[0]=='assets' or p.name.endswith(('.forge.json','.prefab.json')))}
    refs=set(); dynamic=False
    def scan(v):
        nonlocal dynamic
        if isinstance(v,dict):
            if isinstance(v.get('script'),str) and v['script'].strip(): dynamic=True
            for x in v.values(): scan(x)
        elif isinstance(v,list):
            for x in v: scan(x)
        elif isinstance(v,str) and v.startswith('assets/'):
            if v not in files: raise ValueError('Missing or unsafe asset: '+v)
            refs.add(v)
    for name,p in files.items():
        if name.endswith(('.forge.json','.prefab.json')): scan(json.loads(p.read_text(encoding='utf-8-sig')))
    # Runtime demo cues are implicit, not JSON references.
    refs.update(n for n in files if n in {'assets/collect.wav','assets/hurt.wav','assets/win.wav'})
    # Scripts can use arbitrary paths. Do not prune scripted projects.
    keep_all=not prune or dynamic
    selected={n:p for n,p in files.items() if keep_all or not n.startswith('assets/') or n in refs}
    manifest={'prune_requested':prune,'prune_applied':not keep_all,'script_detected':dynamic,'files':[],'excluded':sorted(set(files)-set(selected))}
    if optimize:
        from PIL import Image
    output.parent.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(dir=output.parent) as tmp:
        zpath=pathlib.Path(tmp)/'bundle.zip'
        with zipfile.ZipFile(zpath,'w',zipfile.ZIP_DEFLATED,compresslevel=9) as z:
            for n,p in sorted(selected.items()):
                data=p.read_bytes(); original=len(data)
                if optimize and p.suffix.lower()=='.png':
                    with Image.open(io.BytesIO(data)) as im:
                        if im.mode in ('RGB','RGBA') and not im.info and not getattr(im,'is_animated',False):
                            buf=io.BytesIO(); im.save(buf,format='PNG',optimize=True)
                            candidate=buf.getvalue()
                            with Image.open(io.BytesIO(candidate)) as check:
                                if check.mode==im.mode and check.size==im.size and check.tobytes()==im.tobytes() and len(candidate)<len(data): data=candidate
                z.writestr(n,data)
                manifest['files'].append({'path':n,'original_bytes':original,'packed_bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
            z.writestr('forge-package-manifest.json',json.dumps(manifest,indent=2))
        with zipfile.ZipFile(zpath) as z:
            assert z.testzip() is None
        shutil.move(zpath,output)
    return manifest

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('project_folder');p.add_argument('output_zip');p.add_argument('--prune',action='store_true',help='Only for projects with no implicit asset references');p.add_argument('--optimize-png',action='store_true');a=p.parse_args();m=pack(a.project_folder,a.output_zip,a.prune,a.optimize_png);print(json.dumps({'files':len(m['files']),'excluded':m['excluded'],'prune_applied':m['prune_applied'],'source_bytes':sum(x['original_bytes'] for x in m['files']),'packed_bytes':sum(x['packed_bytes'] for x in m['files'])},indent=2))
