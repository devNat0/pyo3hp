import os
shell = os.getenv("SHELL")

print("""<!DOCTYPE html>
<html>
<body>
""")

print("Hello from index.py:", shell)
print("<br>")

color = "red"
print(f"My car is {color} <br>")


print("""
</body>
</html>
""")