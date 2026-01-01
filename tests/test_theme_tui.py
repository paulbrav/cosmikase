import pytest
from pathlib import Path
from omarchy_pop.themes import _unique_dirs, list_themes

def test_unique_dirs(tmp_path):
    dir1 = tmp_path / "dir1"
    dir1.mkdir()
    dir2 = tmp_path / "dir2"
    dir2.mkdir()
    
    # Duplicate path
    candidates = [dir1, dir2, dir1]
    unique = _unique_dirs(candidates)
    assert len(unique) == 2
    assert dir1 in unique
    assert dir2 in unique

def test_list_themes(tmp_path):
    themes_dir = tmp_path / "themes"
    themes_dir.mkdir()
    (themes_dir / "nord").mkdir()
    (themes_dir / "tokyo-night").mkdir()
    (themes_dir / "random-file.txt").touch()
    
    names = list_themes(themes_dir)
    assert names == ["nord", "tokyo-night"]

def test_list_themes_none():
    assert list_themes(None) == []
    assert list_themes(Path("/non/existent/path")) == []

