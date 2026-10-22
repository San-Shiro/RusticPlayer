$ErrorActionPreference = "Stop"

$icons = @(
    "play", "pause", "skip-back", "skip-forward", "volume-2", "volume-x",
    "list", "settings", "music", "search", "library", "disc", "repeat", "shuffle",
    "more-horizontal", "heart"
)

Write-Host "Downloading Icons..."
foreach ($icon in $icons) {
    $url = "https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/$icon.svg"
    $dest = "crates\ui-kit\assets\icons\$icon.svg"
    try {
        Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing
        Write-Host "  Downloaded $icon.svg"
    } catch {
        Write-Host "  Failed to download $icon.svg"
    }
}

Write-Host "Downloading Fonts..."
$fonts = @(
    @{ Name = "Rubik-Regular.ttf"; Url = "https://raw.githubusercontent.com/google/fonts/main/ofl/rubik/Rubik%5Bwght%5D.ttf" },
    @{ Name = "SpaceMono-Regular.ttf"; Url = "https://raw.githubusercontent.com/google/fonts/main/ofl/spacemono/SpaceMono-Regular.ttf" },
    @{ Name = "SpaceMono-Bold.ttf"; Url = "https://raw.githubusercontent.com/google/fonts/main/ofl/spacemono/SpaceMono-Bold.ttf" }
)

foreach ($font in $fonts) {
    $dest = "crates\ui-kit\assets\fonts\$($font.Name)"
    try {
        Invoke-WebRequest -Uri $font.Url -OutFile $dest -UseBasicParsing
        Write-Host "  Downloaded $($font.Name)"
    } catch {
        Write-Host "  Failed to download $($font.Name)"
    }
}

Write-Host "Done!"
