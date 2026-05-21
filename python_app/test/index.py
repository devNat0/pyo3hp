import os
shell = os.getenv("SHELL")

print("""<!DOCTYPE html>
<html>
<body>
""")

print("Hello from test/index.py:", shell)
print("<br>")

color = "blue"
print(f"My car is {color} <br>")


print("""
</body>
</html>
""")