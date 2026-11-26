"""Export tracked source and overlay patch from Git, preserving local Cargo.lock/target."""
from pathlib import Path, PurePosixPath
import argparse, hashlib, json, re, subprocess, tomllib, zipfile
ROOT=Path(__file__).resolve().parents[1]
def git(*args):
    return subprocess.check_output(['git','-C',str(ROOT),*args])
def version_tuple(tag):
    return tuple(map(int,tag.removeprefix('source-v').split('.')))
def files(ref):
    return set(git('ls-tree','-r','--name-only','-z',ref).decode('utf-8').rstrip('\0').split('\0'))
def blob(ref,path):
    return git('show',f'{ref}:{path}')
def eligible(path):
    p=PurePosixPath(path)
    return bool(path) and not p.is_absolute() and '..' not in p.parts and not set(p.parts)&{'.git','target','__pycache__','dist','ci-packages'}
def main():
    p=argparse.ArgumentParser();p.add_argument('--base');p.add_argument('--output',type=Path,default=ROOT/'ci-packages');a=p.parse_args()
    head=git('rev-parse','HEAD').decode().strip()
    version=tomllib.loads(blob(head,'Cargo.toml').decode())['package']['version']
    tags=[t for t in git('tag','--merged',head,'--list','source-v*').decode().splitlines() if re.fullmatch(r'source-v\d+(?:\.\d+)+',t) and version_tuple(t)<version_tuple('source-v'+version)]
    old=a.base or max(tags,key=version_tuple,default=None)
    out=a.output.resolve();out.mkdir(parents=True,exist_ok=True)
    now={f for f in files(head) if eligible(f)}
    full=out/f'ForgeEngine-v{version}-source.zip'
    with zipfile.ZipFile(full,'w',zipfile.ZIP_DEFLATED) as z:
        for f in sorted(now):z.writestr(f'ForgeEngine-v{version}/{f}',blob(head,f))
    with zipfile.ZipFile(full) as z:assert z.testzip() is None
    if old:
        prev={f for f in files(old) if eligible(f)}
        changed=[f for f in sorted(now-{'Cargo.lock'}) if f not in prev or blob(head,f)!=blob(old,f)]
        removed=sorted((prev-now)-{'Cargo.lock'})
        base_version=tomllib.loads(blob(old,'Cargo.toml').decode())['package']['version']
        patch=out/f'ForgeEngine-v{version}-update-from-{base_version}.zip'
        manifest={'base_ref':old,'base_commit':git('rev-parse',old).decode().strip(),'head_commit':head,'version':version,'preserve':['Cargo.lock','target/'],'remove_manually_after_backup':removed,'sha256':{f:hashlib.sha256(blob(head,f)).hexdigest() for f in changed}}
        with zipfile.ZipFile(patch,'w',zipfile.ZIP_DEFLATED) as z:
            for f in changed:z.writestr(f,blob(head,f))
            z.writestr('FORGE_PATCH_MANIFEST.json',json.dumps(manifest,indent=2)+'\n')
        with zipfile.ZipFile(patch) as z:
            assert z.testzip() is None
            assert all(hashlib.sha256(z.read(f)).hexdigest()==h for f,h in manifest['sha256'].items())
        print(patch, 'changed:',len(changed),'removed:',len(removed))
    print(full,'tracked files:',len(now))
if __name__=='__main__':main()
