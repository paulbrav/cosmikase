import json
import shutil
import tempfile
import unittest
from pathlib import Path

import yaml

from cosmikase.themes import load_manifest
from cosmikase.validate import validate_ron


class TestThemeLogic(unittest.TestCase):
    def setUp(self):
        self.test_dir = Path(tempfile.mkdtemp())
        self.theme_path = self.test_dir / "test-theme"
        self.theme_path.mkdir()

    def tearDown(self):
        shutil.rmtree(self.test_dir)

    def test_load_manifest_yaml(self):
        yaml_content = {
            "name": "Test Theme",
            "variant": "dark",
            "colors": {"bg": "#000000"},
            "cursor": {"theme": "TestCursor", "extension": "test.ext"},
            "wallpaper": "bg.png"
        }
        with open(self.theme_path / "theme.yaml", "w") as f:
            yaml.dump(yaml_content, f)
            
        manifest = load_manifest(self.theme_path)
        self.assertEqual(manifest.name, "Test Theme")
        self.assertEqual(manifest.variant, "dark")
        self.assertEqual(manifest.colors["bg"], "#000000")
        self.assertEqual(manifest.cursor_theme, "TestCursor")
        self.assertEqual(manifest.wallpaper, "bg.png")

    def test_load_manifest_legacy(self):
        # Create legacy files
        (self.theme_path / "light.mode").touch()
        with open(self.theme_path / "cursor.json", "w") as f:
            json.dump({"colorTheme": "LegacyCursor", "extension": "legacy.ext"}, f)
            
        manifest = load_manifest(self.theme_path)
        self.assertEqual(manifest.variant, "light")
        self.assertEqual(manifest.cursor_theme, "LegacyCursor")

    def test_validate_ron_valid(self):
        ron_content = '(key: "value", list: [1, 2, 3], nested: (a: 1))'
        ron_path = self.test_dir / "valid.ron"
        ron_path.write_text(ron_content)
        self.assertTrue(validate_ron(ron_path))

    def test_validate_ron_invalid(self):
        ron_content = '(key: "value", list: [1, 2, 3]'  # Missing closing paren
        ron_path = self.test_dir / "invalid.ron"
        ron_path.write_text(ron_content)
        self.assertFalse(validate_ron(ron_path))

    def test_validate_ron_with_quotes(self):
        ron_content = '(key: "value (with paren)", nested: (a: 1))'
        ron_path = self.test_dir / "quotes.ron"
        ron_path.write_text(ron_content)
        self.assertTrue(validate_ron(ron_path))


if __name__ == "__main__":
    unittest.main()

