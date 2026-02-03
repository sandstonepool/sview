# VHS tape for generating sview screenshots
# 
# Prerequisites:
#   - Install VHS: https://github.com/charmbracelet/vhs
#   - Install ffmpeg: apt install ffmpeg (or brew install ffmpeg)
#   - Have sview built: cargo build --release
#   - Configure sview to connect to a real Cardano node
#
# Run with: vhs docs/screenshot.tape
#
# This will generate:
#   - screenshots/sview-main.gif (animated demo)
#   - screenshots/sview-main.png (static screenshot)

Output screenshots/sview-main.gif
Output screenshots/sview-main.png

Set Shell "bash"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Monokai"
Set Padding 20

# Start sview (adjust path and arguments as needed)
Type "./target/release/sview"
Enter
Sleep 5s

# Capture the main view
Screenshot screenshots/sview-main.png

# Show help overlay
Type "?"
Sleep 2s
Screenshot screenshots/sview-help.png
Type "?"

# Show peer list
Sleep 1s
Type "p"
Sleep 2s
Screenshot screenshots/sview-peers.png

# Cycle through themes
Type "t"
Sleep 1s
Type "t"
Sleep 1s
Type "t"
Sleep 1s

# Return to main view and quit
Type "p"
Sleep 1s
Type "q"
