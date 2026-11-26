from pathlib import Path
import json, unittest
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]/"examples/studio_pro"
class StudioFixture(unittest.TestCase):
 def test_clips(self):
  p=json.loads((ROOT/"project.forge.json").read_text());o=next(o for o in p["objects"] if o["player"])
  with Image.open(ROOT/o["sprite"]) as im:
   for name,a in o["clips"].items():
    self.assertEqual(im.width%a["columns"],0);self.assertEqual(im.height%a["rows"],0);self.assertLessEqual(a["start"]+a["frames"],a["columns"]*a["rows"]);self.assertEqual(len(a["durations"]),a["frames"]);self.assertTrue(all(x>0 for x in a["durations"]))
  for r in o["motion"]["transitions"]:self.assertIn(r["from"],o["clips"]);self.assertIn(r["to"],o["clips"])
 def test_widgets(self):
  p=json.loads((ROOT/"project.forge.json").read_text());o=next(o for o in p["objects"] if o["player"]);s=p["settings"];self.assertTrue(s["hud_canvas_groups"]);self.assertTrue(any(w["action"]=="pause" for w in s["widgets"]))
  for w in s["widgets"]:
   if w["action"]=="progress":self.assertTrue(0<=o["pro"]["variables"][w["text"]]<=1)
if __name__=="__main__":unittest.main()
