import os

app_name = "Crazyflie Mini Client"
app_path = f"target/release/bundle/osx/{app_name}.app"

files = [app_path]
symlinks = {"Applications": "/Applications"}

icon_size = 80
text_size = 12

window_rect = ((200, 120), (640, 480))
background_color = "#ffffff"

icon_locations = {
    f"{app_name}.app": (140, 200),
    "Applications": (500, 200),
}
