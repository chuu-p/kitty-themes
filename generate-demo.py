import os
import subprocess
import time

THEMES_DIR = "themes"
SCREENSHOTS_DIR = "screenshots"
DEMO_FILE = "DEMO.md"
EXAMPLE_FILE = "example/lib.rs"

os.makedirs(SCREENSHOTS_DIR, exist_ok=True)

with open(DEMO_FILE, "w") as f:
    f.write("")

for filename in sorted(os.listdir(THEMES_DIR)):
    if filename.endswith(".conf"):
        conf_path = os.path.join(THEMES_DIR, filename)
        screenshot_filename = f"{filename}.png"
        screenshot_path = os.path.join(SCREENSHOTS_DIR, screenshot_filename)

        proc = subprocess.Popen([
            "kitty", "-c", conf_path, "hx", EXAMPLE_FILE
        ])

        time.sleep(0.5)

        subprocess.run([
            "flameshot", "screen", "--region", "2560x1390+0+50", "--path", screenshot_path
        ])

        proc.terminate()

        with open(DEMO_FILE, "a") as f:
            f.write(f"## {filename}\n")
            f.write(f"![](./{SCREENSHOTS_DIR}/{screenshot_filename})\n\n")

