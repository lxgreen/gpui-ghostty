//! Embedded terminal themes.
//!
//! This module provides a curated set of popular terminal themes embedded directly
//! in the binary. These themes are available without requiring Ghostty.app to be
//! installed or any external theme files.
//!
//! Theme format follows Ghostty's key-value syntax.

use std::collections::HashMap;
use std::sync::LazyLock;

/// A map of theme name (lowercase, normalized) to theme contents.
static EMBEDDED_THEMES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // Catppuccin family
    m.insert("catppuccin-mocha", CATPPUCCIN_MOCHA);
    m.insert("catppuccin mocha", CATPPUCCIN_MOCHA);
    m.insert("catppuccin-latte", CATPPUCCIN_LATTE);
    m.insert("catppuccin latte", CATPPUCCIN_LATTE);
    m.insert("catppuccin-frappe", CATPPUCCIN_FRAPPE);
    m.insert("catppuccin frappe", CATPPUCCIN_FRAPPE);
    m.insert("catppuccin-macchiato", CATPPUCCIN_MACCHIATO);
    m.insert("catppuccin macchiato", CATPPUCCIN_MACCHIATO);

    // Dracula
    m.insert("dracula", DRACULA);

    // Nord family
    m.insert("nord", NORD);
    m.insert("nord-light", NORD_LIGHT);
    m.insert("nord light", NORD_LIGHT);

    // Gruvbox family
    m.insert("gruvbox-dark", GRUVBOX_DARK);
    m.insert("gruvbox dark", GRUVBOX_DARK);
    m.insert("gruvbox-light", GRUVBOX_LIGHT);
    m.insert("gruvbox light", GRUVBOX_LIGHT);

    // Tokyo Night family
    m.insert("tokyonight", TOKYONIGHT);
    m.insert("tokyo-night", TOKYONIGHT);
    m.insert("tokyo night", TOKYONIGHT);
    m.insert("tokyonight-day", TOKYONIGHT_DAY);
    m.insert("tokyonight day", TOKYONIGHT_DAY);
    m.insert("tokyo-night-day", TOKYONIGHT_DAY);
    m.insert("tokyo night day", TOKYONIGHT_DAY);

    // Rose Pine family
    m.insert("rose-pine", ROSE_PINE);
    m.insert("rose pine", ROSE_PINE);
    m.insert("rosepine", ROSE_PINE);
    m.insert("rose-pine-dawn", ROSE_PINE_DAWN);
    m.insert("rose pine dawn", ROSE_PINE_DAWN);
    m.insert("rosepine-dawn", ROSE_PINE_DAWN);
    m.insert("rose-pine-moon", ROSE_PINE_MOON);
    m.insert("rose pine moon", ROSE_PINE_MOON);
    m.insert("rosepine-moon", ROSE_PINE_MOON);

    // Kanagawa family
    m.insert("kanagawa", KANAGAWA_WAVE);
    m.insert("kanagawa-wave", KANAGAWA_WAVE);
    m.insert("kanagawa wave", KANAGAWA_WAVE);
    m.insert("kanagawa-dragon", KANAGAWA_DRAGON);
    m.insert("kanagawa dragon", KANAGAWA_DRAGON);

    // Everforest family
    m.insert("everforest", EVERFOREST_DARK);
    m.insert("everforest-dark", EVERFOREST_DARK);
    m.insert("everforest dark", EVERFOREST_DARK);
    m.insert("everforest-dark-hard", EVERFOREST_DARK);
    m.insert("everforest dark hard", EVERFOREST_DARK);
    m.insert("everforest-light", EVERFOREST_LIGHT);
    m.insert("everforest light", EVERFOREST_LIGHT);
    m.insert("everforest-light-med", EVERFOREST_LIGHT);
    m.insert("everforest light med", EVERFOREST_LIGHT);

    // Ayu family
    m.insert("ayu", AYU);
    m.insert("ayu-dark", AYU);
    m.insert("ayu dark", AYU);
    m.insert("ayu-light", AYU_LIGHT);
    m.insert("ayu light", AYU_LIGHT);
    m.insert("ayu-mirage", AYU_MIRAGE);
    m.insert("ayu mirage", AYU_MIRAGE);

    m
});

/// Look up an embedded theme by name.
///
/// Theme names are case-insensitive and support multiple formats:
/// - `catppuccin-mocha` (kebab-case)
/// - `Catppuccin Mocha` (title case with spaces)
/// - `catppuccin mocha` (lowercase with spaces)
///
/// Returns the theme contents as a string if found.
pub fn get_embedded_theme(name: &str) -> Option<&'static str> {
    let normalized = name.to_lowercase();
    EMBEDDED_THEMES.get(normalized.as_str()).copied()
}

/// Get a list of all available embedded theme names.
///
/// Returns canonical names (kebab-case) for each theme family.
pub fn list_embedded_themes() -> Vec<&'static str> {
    vec![
        // Catppuccin
        "catppuccin-mocha",
        "catppuccin-latte",
        "catppuccin-frappe",
        "catppuccin-macchiato",
        // Dracula
        "dracula",
        // Nord
        "nord",
        "nord-light",
        // Gruvbox
        "gruvbox-dark",
        "gruvbox-light",
        // Tokyo Night
        "tokyonight",
        "tokyonight-day",
        // Rose Pine
        "rose-pine",
        "rose-pine-dawn",
        "rose-pine-moon",
        // Kanagawa
        "kanagawa-wave",
        "kanagawa-dragon",
        // Everforest
        "everforest-dark",
        "everforest-light",
        // Ayu
        "ayu",
        "ayu-light",
        "ayu-mirage",
    ]
}

// =============================================================================
// Embedded Theme Contents
// =============================================================================

const CATPPUCCIN_MOCHA: &str = r#"palette = 0=#45475a
palette = 1=#f38ba8
palette = 2=#a6e3a1
palette = 3=#f9e2af
palette = 4=#89b4fa
palette = 5=#f5c2e7
palette = 6=#94e2d5
palette = 7=#a6adc8
palette = 8=#585b70
palette = 9=#f37799
palette = 10=#89d88b
palette = 11=#ebd391
palette = 12=#74a8fc
palette = 13=#f2aede
palette = 14=#6bd7ca
palette = 15=#bac2de
background = #1e1e2e
foreground = #cdd6f4
cursor-color = #f5e0dc
cursor-text = #1e1e2e
selection-background = #585b70
selection-foreground = #cdd6f4
"#;

const CATPPUCCIN_LATTE: &str = r#"palette = 0=#5c5f77
palette = 1=#d20f39
palette = 2=#40a02b
palette = 3=#df8e1d
palette = 4=#1e66f5
palette = 5=#ea76cb
palette = 6=#179299
palette = 7=#acb0be
palette = 8=#6c6f85
palette = 9=#de293e
palette = 10=#49af3d
palette = 11=#eea02d
palette = 12=#456eff
palette = 13=#fe85d8
palette = 14=#2d9fa8
palette = 15=#bcc0cc
background = #eff1f5
foreground = #4c4f69
cursor-color = #dc8a78
cursor-text = #eff1f5
selection-background = #acb0be
selection-foreground = #4c4f69
"#;

const CATPPUCCIN_FRAPPE: &str = r#"palette = 0=#51576d
palette = 1=#e78284
palette = 2=#a6d189
palette = 3=#e5c890
palette = 4=#8caaee
palette = 5=#f4b8e4
palette = 6=#81c8be
palette = 7=#a5adce
palette = 8=#626880
palette = 9=#e67172
palette = 10=#8ec772
palette = 11=#d9ba73
palette = 12=#7b9ef0
palette = 13=#f2a4db
palette = 14=#5abfb5
palette = 15=#b5bfe2
background = #303446
foreground = #c6d0f5
cursor-color = #f2d5cf
cursor-text = #303446
selection-background = #626880
selection-foreground = #c6d0f5
"#;

const CATPPUCCIN_MACCHIATO: &str = r#"palette = 0=#494d64
palette = 1=#ed8796
palette = 2=#a6da95
palette = 3=#eed49f
palette = 4=#8aadf4
palette = 5=#f5bde6
palette = 6=#8bd5ca
palette = 7=#a5adcb
palette = 8=#5b6078
palette = 9=#ec7486
palette = 10=#8ccf7f
palette = 11=#e1c682
palette = 12=#78a1f6
palette = 13=#f2a9dd
palette = 14=#63cbc0
palette = 15=#b8c0e0
background = #24273a
foreground = #cad3f5
cursor-color = #f4dbd6
cursor-text = #24273a
selection-background = #5b6078
selection-foreground = #cad3f5
"#;

const DRACULA: &str = r#"palette = 0=#21222c
palette = 1=#ff5555
palette = 2=#50fa7b
palette = 3=#f1fa8c
palette = 4=#bd93f9
palette = 5=#ff79c6
palette = 6=#8be9fd
palette = 7=#f8f8f2
palette = 8=#6272a4
palette = 9=#ff6e6e
palette = 10=#69ff94
palette = 11=#ffffa5
palette = 12=#d6acff
palette = 13=#ff92df
palette = 14=#a4ffff
palette = 15=#ffffff
background = #282a36
foreground = #f8f8f2
cursor-color = #f8f8f2
cursor-text = #282a36
selection-background = #44475a
selection-foreground = #ffffff
"#;

const NORD: &str = r#"palette = 0=#3b4252
palette = 1=#bf616a
palette = 2=#a3be8c
palette = 3=#ebcb8b
palette = 4=#81a1c1
palette = 5=#b48ead
palette = 6=#88c0d0
palette = 7=#e5e9f0
palette = 8=#596377
palette = 9=#bf616a
palette = 10=#a3be8c
palette = 11=#ebcb8b
palette = 12=#81a1c1
palette = 13=#b48ead
palette = 14=#8fbcbb
palette = 15=#eceff4
background = #2e3440
foreground = #d8dee9
cursor-color = #eceff4
cursor-text = #282828
selection-background = #eceff4
selection-foreground = #4c566a
"#;

const NORD_LIGHT: &str = r#"palette = 0=#3b4252
palette = 1=#bf616a
palette = 2=#96b17f
palette = 3=#c5a565
palette = 4=#81a1c1
palette = 5=#b48ead
palette = 6=#7bb3c3
palette = 7=#a5abb6
palette = 8=#4c566a
palette = 9=#bf616a
palette = 10=#96b17f
palette = 11=#c5a565
palette = 12=#81a1c1
palette = 13=#b48ead
palette = 14=#82afae
palette = 15=#eceff4
background = #e5e9f0
foreground = #414858
cursor-color = #7bb3c3
cursor-text = #3b4252
selection-background = #d8dee9
selection-foreground = #4c556a
"#;

const GRUVBOX_DARK: &str = r#"palette = 0=#282828
palette = 1=#cc241d
palette = 2=#98971a
palette = 3=#d79921
palette = 4=#458588
palette = 5=#b16286
palette = 6=#689d6a
palette = 7=#a89984
palette = 8=#928374
palette = 9=#fb4934
palette = 10=#b8bb26
palette = 11=#fabd2f
palette = 12=#83a598
palette = 13=#d3869b
palette = 14=#8ec07c
palette = 15=#ebdbb2
background = #282828
foreground = #ebdbb2
cursor-color = #ebdbb2
cursor-text = #282828
selection-background = #665c54
selection-foreground = #ebdbb2
"#;

const GRUVBOX_LIGHT: &str = r#"palette = 0=#fbf1c7
palette = 1=#cc241d
palette = 2=#98971a
palette = 3=#d79921
palette = 4=#458588
palette = 5=#b16286
palette = 6=#689d6a
palette = 7=#7c6f64
palette = 8=#928374
palette = 9=#9d0006
palette = 10=#79740e
palette = 11=#b57614
palette = 12=#076678
palette = 13=#8f3f71
palette = 14=#427b58
palette = 15=#3c3836
background = #fbf1c7
foreground = #3c3836
cursor-color = #3c3836
cursor-text = #3c3836
selection-background = #3c3836
selection-foreground = #fbf1c7
"#;

const TOKYONIGHT: &str = r#"palette = 0=#15161e
palette = 1=#f7768e
palette = 2=#9ece6a
palette = 3=#e0af68
palette = 4=#7aa2f7
palette = 5=#bb9af7
palette = 6=#7dcfff
palette = 7=#a9b1d6
palette = 8=#414868
palette = 9=#f7768e
palette = 10=#9ece6a
palette = 11=#e0af68
palette = 12=#7aa2f7
palette = 13=#bb9af7
palette = 14=#7dcfff
palette = 15=#c0caf5
background = #1a1b26
foreground = #c0caf5
cursor-color = #c0caf5
cursor-text = #15161e
selection-background = #33467c
selection-foreground = #c0caf5
"#;

const TOKYONIGHT_DAY: &str = r#"palette = 0=#e9e9ed
palette = 1=#f52a65
palette = 2=#587539
palette = 3=#8c6c3e
palette = 4=#2e7de9
palette = 5=#9854f1
palette = 6=#007197
palette = 7=#6172b0
palette = 8=#a1a6c5
palette = 9=#f52a65
palette = 10=#587539
palette = 11=#8c6c3e
palette = 12=#2e7de9
palette = 13=#9854f1
palette = 14=#007197
palette = 15=#3760bf
background = #e1e2e7
foreground = #3760bf
cursor-color = #3760bf
cursor-text = #e1e2e7
selection-background = #99a7df
selection-foreground = #3760bf
"#;

const ROSE_PINE: &str = r#"palette = 0=#26233a
palette = 1=#eb6f92
palette = 2=#31748f
palette = 3=#f6c177
palette = 4=#9ccfd8
palette = 5=#c4a7e7
palette = 6=#ebbcba
palette = 7=#e0def4
palette = 8=#6e6a86
palette = 9=#eb6f92
palette = 10=#31748f
palette = 11=#f6c177
palette = 12=#9ccfd8
palette = 13=#c4a7e7
palette = 14=#ebbcba
palette = 15=#e0def4
background = #191724
foreground = #e0def4
cursor-color = #e0def4
cursor-text = #191724
selection-background = #403d52
selection-foreground = #e0def4
"#;

const ROSE_PINE_DAWN: &str = r#"palette = 0=#f2e9e1
palette = 1=#b4637a
palette = 2=#286983
palette = 3=#ea9d34
palette = 4=#56949f
palette = 5=#907aa9
palette = 6=#d7827e
palette = 7=#575279
palette = 8=#9893a5
palette = 9=#b4637a
palette = 10=#286983
palette = 11=#ea9d34
palette = 12=#56949f
palette = 13=#907aa9
palette = 14=#d7827e
palette = 15=#575279
background = #faf4ed
foreground = #575279
cursor-color = #575279
cursor-text = #faf4ed
selection-background = #dfdad9
selection-foreground = #575279
"#;

const ROSE_PINE_MOON: &str = r#"palette = 0=#393552
palette = 1=#eb6f92
palette = 2=#3e8fb0
palette = 3=#f6c177
palette = 4=#9ccfd8
palette = 5=#c4a7e7
palette = 6=#ea9a97
palette = 7=#e0def4
palette = 8=#6e6a86
palette = 9=#eb6f92
palette = 10=#3e8fb0
palette = 11=#f6c177
palette = 12=#9ccfd8
palette = 13=#c4a7e7
palette = 14=#ea9a97
palette = 15=#e0def4
background = #232136
foreground = #e0def4
cursor-color = #e0def4
cursor-text = #232136
selection-background = #44415a
selection-foreground = #e0def4
"#;

const KANAGAWA_WAVE: &str = r#"palette = 0=#090618
palette = 1=#c34043
palette = 2=#76946a
palette = 3=#c0a36e
palette = 4=#7e9cd8
palette = 5=#957fb8
palette = 6=#6a9589
palette = 7=#c8c093
palette = 8=#727169
palette = 9=#e82424
palette = 10=#98bb6c
palette = 11=#e6c384
palette = 12=#7fb4ca
palette = 13=#938aa9
palette = 14=#7aa89f
palette = 15=#dcd7ba
background = #1f1f28
foreground = #dcd7ba
cursor-color = #c8c093
cursor-text = #1d202f
selection-background = #2d4f67
selection-foreground = #c8c093
"#;

const KANAGAWA_DRAGON: &str = r#"palette = 0=#0d0c0c
palette = 1=#c4746e
palette = 2=#8a9a7b
palette = 3=#c4b28a
palette = 4=#8ba4b0
palette = 5=#a292a3
palette = 6=#8ea4a2
palette = 7=#c8c093
palette = 8=#a6a69c
palette = 9=#e46876
palette = 10=#87a987
palette = 11=#e6c384
palette = 12=#7fb4ca
palette = 13=#938aa9
palette = 14=#7aa89f
palette = 15=#c5c9c5
background = #181616
foreground = #c8c093
cursor-color = #c5c9c5
cursor-text = #1d202f
selection-background = #223249
selection-foreground = #c5c9c5
"#;

const EVERFOREST_DARK: &str = r#"palette = 0=#7a8478
palette = 1=#e67e80
palette = 2=#a7c080
palette = 3=#dbbc7f
palette = 4=#7fbbb3
palette = 5=#d699b6
palette = 6=#83c092
palette = 7=#f2efdf
palette = 8=#a6b0a0
palette = 9=#f85552
palette = 10=#8da101
palette = 11=#dfa000
palette = 12=#3a94c5
palette = 13=#df69ba
palette = 14=#35a77c
palette = 15=#fffbef
background = #1e2326
foreground = #d3c6aa
cursor-color = #e69875
cursor-text = #4c3743
selection-background = #4c3743
selection-foreground = #d3c6aa
"#;

const EVERFOREST_LIGHT: &str = r#"palette = 0=#7a8478
palette = 1=#e67e80
palette = 2=#9ab373
palette = 3=#c1a266
palette = 4=#7fbbb3
palette = 5=#d699b6
palette = 6=#83c092
palette = 7=#b2af9f
palette = 8=#a6b0a0
palette = 9=#f85552
palette = 10=#8da101
palette = 11=#dfa000
palette = 12=#3a94c5
palette = 13=#df69ba
palette = 14=#35a77c
palette = 15=#fffbef
background = #efebd4
foreground = #5c6a72
cursor-color = #f57d26
cursor-text = #eaedc8
selection-background = #eaedc8
selection-foreground = #5c6a72
"#;

const AYU: &str = r#"palette = 0=#11151c
palette = 1=#ea6c73
palette = 2=#7fd962
palette = 3=#f9af4f
palette = 4=#53bdfa
palette = 5=#cda1fa
palette = 6=#90e1c6
palette = 7=#c7c7c7
palette = 8=#686868
palette = 9=#f07178
palette = 10=#aad94c
palette = 11=#ffb454
palette = 12=#59c2ff
palette = 13=#d2a6ff
palette = 14=#95e6cb
palette = 15=#ffffff
background = #0b0e14
foreground = #bfbdb6
cursor-color = #e6b450
cursor-text = #0b0e14
selection-background = #409fff
selection-foreground = #0b0e14
"#;

const AYU_LIGHT: &str = r#"palette = 0=#000000
palette = 1=#ea6c6d
palette = 2=#6cbf43
palette = 3=#eca944
palette = 4=#3199e1
palette = 5=#9e75c7
palette = 6=#46ba94
palette = 7=#bababa
palette = 8=#686868
palette = 9=#f07171
palette = 10=#86b300
palette = 11=#f2ae49
palette = 12=#399ee6
palette = 13=#a37acc
palette = 14=#4cbf99
palette = 15=#d1d1d1
background = #f8f9fa
foreground = #5c6166
cursor-color = #ffaa33
cursor-text = #f8f9fa
selection-background = #035bd6
selection-foreground = #f8f9fa
"#;

const AYU_MIRAGE: &str = r#"palette = 0=#171b24
palette = 1=#ed8274
palette = 2=#87d96c
palette = 3=#facc6e
palette = 4=#6dcbfa
palette = 5=#dabafa
palette = 6=#90e1c6
palette = 7=#c7c7c7
palette = 8=#686868
palette = 9=#f28779
palette = 10=#d5ff80
palette = 11=#ffd173
palette = 12=#73d0ff
palette = 13=#dfbfff
palette = 14=#95e6cb
palette = 15=#ffffff
background = #1f2430
foreground = #cccac2
cursor-color = #ffcc66
cursor-text = #1f2430
selection-background = #409fff
selection-foreground = #1f2430
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_embedded_theme_case_insensitive() {
        assert!(get_embedded_theme("catppuccin-mocha").is_some());
        assert!(get_embedded_theme("Catppuccin-Mocha").is_some());
        assert!(get_embedded_theme("CATPPUCCIN-MOCHA").is_some());
        assert!(get_embedded_theme("Catppuccin Mocha").is_some());
    }

    #[test]
    fn test_get_embedded_theme_not_found() {
        assert!(get_embedded_theme("nonexistent-theme").is_none());
    }

    #[test]
    fn test_list_embedded_themes() {
        let themes = list_embedded_themes();
        assert!(themes.contains(&"catppuccin-mocha"));
        assert!(themes.contains(&"dracula"));
        assert!(themes.contains(&"nord"));
        assert!(themes.len() >= 20);
    }

    #[test]
    fn test_theme_contents_valid() {
        // Verify themes contain expected keys
        let theme = get_embedded_theme("dracula").unwrap();
        assert!(theme.contains("background"));
        assert!(theme.contains("foreground"));
        assert!(theme.contains("palette = 0="));
        assert!(theme.contains("palette = 15="));
    }
}
