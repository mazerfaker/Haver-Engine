import tempfile, pathlib, json, zipfile, unittest
from pack_assets import pack
class PackTests(unittest.TestCase):
 def test_prune_and_script_guard(self):
  with tempfile.TemporaryDirectory() as d:
   r=pathlib.Path(d)/'game';r.mkdir();(r/'assets').mkdir()
   for n in ['used.png','unused.png','win.wav']:(r/'assets'/n).write_bytes(b'example')
   p=r/'project.forge.json';p.write_text(json.dumps({'objects':[{'sprite':'assets/used.png'}]}))
   m=pack(r,pathlib.Path(d)/'one.zip',True);self.assertEqual(m['excluded'],['assets/unused.png'])
   p.write_text(json.dumps({'objects':[{'script':'load()'}]}));m=pack(r,pathlib.Path(d)/'two.zip',True);self.assertFalse(m['prune_applied']);self.assertFalse(m['excluded'])
 def test_missing(self):
  with tempfile.TemporaryDirectory() as d:
   r=pathlib.Path(d);(r/'project.forge.json').write_text(json.dumps({'sprite':'assets/missing.png'}))
   with self.assertRaises(ValueError):pack(r,r/'out.zip')
if __name__=='__main__':unittest.main()
