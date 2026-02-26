from PIL import Image, ImageDraw, ImageFont
import os

size = 1024
img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

bg_color = (74, 144, 217, 255)
margin = 40
draw.rounded_rectangle([margin, margin, size - margin, size - margin], radius=120, fill=bg_color)

try:
    font = ImageFont.truetype('C:/Windows/Fonts/arial.ttf', 400)
except Exception as e:
    print(f'Font fallback: {e}')
    font = ImageFont.load_default()

text = 'IM'
bbox = draw.textbbox((0, 0), text, font=font)
text_w = bbox[2] - bbox[0]
text_h = bbox[3] - bbox[1]
x = (size - text_w) / 2
y = (size - text_h) / 2 - bbox[1]
draw.text((x, y), text, fill=(255, 255, 255, 255), font=font)

output = 'E:/Projects/InfraMap/app-icon.png'
img.save(output, 'PNG')
print(f'Created source icon: {output}')
print(f'Size: {os.path.getsize(output)} bytes')
