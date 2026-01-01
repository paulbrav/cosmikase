#!/bin/bash
# Script to download all missing theme backgrounds

set -e

THEMES_DIR="/home/paulbrav/github_software/cosmikase/themes"
THEME_REPO="https://raw.githubusercontent.com/basecamp/omarchy/master/themes"

cd "$THEMES_DIR"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🎨 Downloading Theme Backgrounds..."
echo ""

# Function to download backgrounds for a theme
download_theme_backgrounds() {
    local theme_name="$1"
    local theme_dir="$THEMES_DIR/$theme_name"
    
    if [ ! -d "$theme_dir" ]; then
        echo -e "${RED}✗${NC} Theme directory not found: $theme_name"
        return 1
    fi
    
    mkdir -p "$theme_dir/backgrounds"
    
    echo -e "${YELLOW}Checking $theme_name...${NC}"
    
    # Try to download backgrounds from the official repo
    # First, check if backgrounds exist
    local downloaded=0
    
    for img_num in {1..5}; do
        for ext in jpg png jpeg; do
            # Try different naming patterns
            for pattern in "$img_num.$ext" "$img_num-$theme_name.$ext" "${img_num}-${theme_name}.$ext"; do
                local url="$THEME_REPO/$theme_name/backgrounds/$pattern"
                local output="$theme_dir/backgrounds/$pattern"
                
                if [ ! -f "$output" ]; then
                    if curl -f -s -L "$url" -o "$output" 2>/dev/null; then
                        echo -e "  ${GREEN}✓${NC} Downloaded: $pattern"
                        downloaded=$((downloaded + 1))
                        break 2  # Break out of both ext and pattern loops
                    fi
                fi
            done
        done
    done
    
    if [ $downloaded -eq 0 ]; then
        # Check if theme already has images
        local existing=$(find "$theme_dir/backgrounds" -type f \( -name "*.jpg" -o -name "*.png" -o -name "*.jpeg" \) 2>/dev/null | wc -l)
        if [ $existing -gt 0 ]; then
            echo -e "  ${GREEN}✓${NC} Already has $existing image(s)"
        else
            echo -e "  ${YELLOW}⚠${NC}  No backgrounds found in repo - keeping placeholder"
        fi
    fi
}

# Download backgrounds for all official themes
official_themes=(
    "tokyo-night"
    "nord"
    "gruvbox"
    "kanagawa"
    "everforest"
    "rose-pine"
    "catppuccin-latte"
    "matte-black"
    "ristretto"
    "ethereal"
    "flexoki-light"
    "hackerman"
    "catppuccin"
)

for theme in "${official_themes[@]}"; do
    download_theme_backgrounds "$theme"
done

echo ""
echo "🎨 Custom Themes (pop-default, osaka-jade):"
echo "   For these custom themes, you'll need to add your own wallpapers to:"
echo "   - themes/pop-default/backgrounds/"
echo "   - themes/osaka-jade/backgrounds/"
echo ""
echo "   Suggested sources:"
echo "   - Pop!_OS wallpapers: https://github.com/pop-os/wallpapers"
echo "   - Unsplash (search: teal, orange, tech)"
echo "   - For osaka-jade: search for jade/cyan themed wallpapers"
echo ""
echo "✅ Background download complete!"
echo ""
echo "📊 Summary:"
for theme in */; do
    theme_name="${theme%/}"
    img_count=$(find "$theme/backgrounds" -type f \( -name "*.jpg" -o -name "*.png" -o -name "*.jpeg" \) 2>/dev/null | wc -l)
    if [ $img_count -gt 0 ]; then
        echo -e "  ${GREEN}✓${NC} $theme_name: $img_count image(s)"
    else
        echo -e "  ${YELLOW}⚠${NC}  $theme_name: 0 images (needs wallpapers)"
    fi
done

