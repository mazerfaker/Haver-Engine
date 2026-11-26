"""Journalled transfer of a local subtree between unused library prefabs.
Dry run by default. Close editors/players. Refuses referenced or scripted prefabs.
"""
import argparse, os, re, uuid
from pathlib import Path
import relocate_asset as tx

def uid(s):
    try: return str(uuid.UUID(s))
    except (ValueError,TypeError,AttributeError): raise ValueError('Invalid object UUID')
def file_name(s):
    if s.startswith('.') or not re.fullmatch(r'[A-Za-z0-9_.-]+\.prefab\.json',s): raise ValueError('Use a simple library prefab filename')
    return 'prefabs/'+s

def unpack(obj):
    if isinstance(obj,list): return obj,[]
    if isinstance(obj,dict) and isinstance(obj.get('objects'),list) and isinstance(obj.get('nested',[]),list): return obj['objects'],obj.get('nested',[])
    raise ValueError('Invalid prefab document')
def validate(objects,links):
    if not objects or len(objects)>10000 or len(links)>128: raise ValueError('Prefab object/link limit or empty prefab')
    parents={uid(o['id']):uid(o['parent']) if o.get('parent') else None for o in objects}
    if len(parents)!=len(objects): raise ValueError('Duplicate object UUID')
    for i in parents:
        seen={i}; p=parents[i]
        while p is not None:
            if p in seen or p not in parents: raise ValueError('Invalid hierarchy')
            seen.add(p);p=parents[p]
    ids=set()
    for l in links:
        i=uid(l['id']);file_name(l['file'])
        if i in ids or i in parents: raise ValueError('Duplicate link UUID')
        ids.add(i)
        if l.get('parent') and uid(l['parent']) not in parents: raise ValueError('Missing attachment parent')
    return parents

def strings(x):
    if isinstance(x,str): yield x
    elif isinstance(x,list):
        for v in x: yield from strings(v)
    elif isinstance(x,dict):
        for k,v in x.items():
            yield k;yield from strings(v)

def scripted(x):
    if isinstance(x,dict): return bool(x.get('script')) or any(scripted(v) for v in x.values())
    if isinstance(x,list): return any(scripted(v) for v in x)
    return False

def plan(root,src,dst,object_id,parent=None):
    root=root.resolve(strict=True);a,b=file_name(src),file_name(dst)
    if a.casefold()==b.casefold(): raise ValueError('Different prefabs required')
    object_id=uid(object_id);parent=uid(parent) if parent else None
    raw_a=tx.read(tx.checked(root,a),8*1024*1024);raw_b=tx.read(tx.checked(root,b),8*1024*1024)
    doc_a,doc_b=tx.json_read(raw_a),tx.json_read(raw_b);ao,al=unpack(doc_a);bo,bl=unpack(doc_b)
    ap,bp=validate(ao,al),validate(bo,bl)
    if object_id not in ap or (parent and parent not in bp): raise ValueError('Unknown source object or destination parent')
    moved={object_id}
    while True:
        n=len(moved);moved.update(i for i,p in ap.items() if p in moved)
        if len(moved)==n: break
    if moved.intersection(bp): raise ValueError('UUID collision')
    # Strict conservative guard: linked materialized instances and external consumers cannot be migrated safely yet.
    count=total=0
    def scan_error(exc):raise exc
    for directory,dirs,files in os.walk(root,followlinks=False,onerror=scan_error):
        d=Path(directory);dirs[:]=[n for n in dirs if not n.startswith('.') and n not in {'target','__pycache__'}]
        for n in dirs:
            if (d/n).resolve()!=d/n: raise ValueError('Linked directory')
        for n in files:
            count+=1
            if count>32768: raise ValueError('Scan limit')
            if n.startswith('.') or n.endswith('.bak'): continue
            p=d/n; rel=p.relative_to(root).as_posix()
            if p.suffix.lower() not in {'.json','.lua'}: continue
            raw=tx.read(tx.checked(root,rel),tx.MAX_DOC);total+=len(raw)
            if total>tx.MAX_TOTAL: raise ValueError('Scan byte limit')
            if p.suffix.lower()=='.lua': raise ValueError('Lua present: cannot prove transfer safe')
            obj=tx.json_read(raw)
            if scripted(obj): raise ValueError('Embedded scripts present: cannot prove transfer safe')
            for v in strings(obj):
                q=v.casefold().replace('\\','/')
                if src.casefold() in q or dst.casefold() in q: raise ValueError(f'External prefab reference in {rel}')
                if rel not in {a,b} and any(i in q for i in moved):
                    # Per-object atlas metadata stays valid because the object ID and assets do not change.
                    if not (re.fullmatch(r'assets/atlas-state-[0-9a-fA-F-]{36}\.json',rel) and isinstance(obj,dict) and obj.get('schema')==2 and obj.get('object') in moved):
                        raise ValueError(f'External UUID reference in {rel}')
    take=[o for o in ao if uid(o['id']) in moved];keep=[o for o in ao if uid(o['id']) not in moved]
    # Do not transfer attached external references: scoped nested IDs would change.
    if any(l.get('parent') and uid(l['parent']) in moved for l in al): raise ValueError('Subtree has external attachments; detach first')
    if any(o.get('prefab_link') for o in take): raise ValueError('Linked instance transfer unsupported')
    # Parent relationships use world positions; preserve transforms and animation coordinates.
    for o in take:
        if uid(o['id'])==object_id:o['parent']=parent
    # Do not silently leave cross-file references in custom graph/metadata fields.
    if any(i in s.casefold() for s in strings([keep,al,bo,bl]) for i in moved): raise ValueError('Remaining document references moved UUID')
    remaining=set(ap)-moved
    if any(i in s.casefold() for s in strings(take) for i in remaining): raise ValueError('Transferred subtree references a remaining source UUID')
    validate(keep,al);validate(bo+take,bl)
    def repack(doc,objects,links):
        if isinstance(doc,list):return objects
        return {**doc,'objects':objects,'nested':links}
    ops=[(a,raw_a,tx.encoded(repack(doc_a,keep,al))),(b,raw_b,tx.encoded(repack(doc_b,bo+take,bl)))]
    for cache in sorted(tx.CACHE):
        p=tx.checked(root,cache)
        if p.exists():ops.append((cache,tx.read(p,tx.MAX_DOC),None))
    return root,ops,len(take)

def apply(root,src,dst,object_id,parent=None,fail_after=None):
    root,ops,n=plan(root,src,dst,object_id,parent)
    return tx.commit_ops(root,ops,fail_after),n

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('root',type=Path);p.add_argument('source');p.add_argument('destination');p.add_argument('object');p.add_argument('--parent');p.add_argument('--apply',action='store_true');args=p.parse_args()
    _,_,n=plan(args.root,args.source,args.destination,args.object,args.parent);print(f'{n} objects in subtree; source and destination validated')
    if args.apply: print('Committed. Recovery journal:',apply(args.root,args.source,args.destination,args.object,args.parent)[0])
    else:print('Dry run only. Close editors and players before --apply.')
if __name__=='__main__':
    try:main()
    except (ValueError,RuntimeError,OSError,KeyError) as e:raise SystemExit(str(e))
