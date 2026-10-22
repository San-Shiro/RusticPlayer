import os
import glob

for f in glob.glob("assets/icons/*.svg"):
    with open(f, 'r') as file:
        content = file.read()
    if '<svg width' not in content:
        content = content.replace('<svg ', '<svg width="24" height="24" ')
        with open(f, 'w') as file:
            file.write(content)
