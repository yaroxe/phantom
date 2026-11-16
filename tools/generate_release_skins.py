"""Generate release-quality 64x64 Minecraft skins and test 3D renders."""
import os
import math
from PIL import Image, ImageDraw

def create_zek_batya():
    img = Image.new('RGBA', (64, 64), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    # Palette
    skin = (218, 168, 130, 255)
    skin_shadow = (185, 138, 104, 255)
    skin_high = (235, 185, 150, 255)
    skin_dark = (155, 112, 82, 255)

    hair_dark = (36, 28, 22, 255)
    hair_gray = (75, 70, 68, 255)
    stubble = (140, 110, 95, 255)
    mustache = (45, 34, 26, 255)
    gold_tooth = (255, 215, 0, 255)
    gold_tooth_high = (255, 245, 140, 255)
    tattoo_blue = (40, 65, 88, 255)

    tel_blue = (28, 55, 125, 255)
    tel_blue_dark = (20, 42, 98, 255)
    tel_white = (235, 240, 248, 255)
    tel_white_dark = (200, 206, 216, 255)

    pants_dark = (22, 24, 28, 255)
    pants_mid = (34, 37, 44, 255)
    pants_light = (48, 52, 62, 255)
    pants_stripe = (220, 225, 235, 255)

    leather_shoe = (30, 22, 18, 255)
    shoe_sole = (55, 45, 38, 255)

    gold_chain = (245, 195, 35, 255)

    # 1. HEAD BASE (x: 0..32, y: 0..16)
    # Head Front (8..16, 8..16)
    for y in range(8, 16):
        for x in range(8, 16):
            img.putpixel((x, y), skin)
    # Hair on top of forehead
    for x in range(8, 16):
        img.putpixel((x, 8), hair_dark)
    img.putpixel((8, 9), hair_dark)
    img.putpixel((15, 9), hair_dark)
    img.putpixel((9, 9), hair_gray)
    img.putpixel((14, 9), hair_gray)

    # Forehead "666" or prison mark
    # 6 6 6: x=10, 12, 14 at y=9, 10
    img.putpixel((10, 10), tattoo_blue)
    img.putpixel((12, 10), tattoo_blue)
    img.putpixel((14, 10), tattoo_blue)

    # Eyes at y=11
    img.putpixel((9, 11), (240, 240, 245, 255))
    img.putpixel((10, 11), (45, 55, 70, 255)) # tired hazel/blue-gray eyes
    img.putpixel((13, 11), (45, 55, 70, 255))
    img.putpixel((14, 11), (240, 240, 245, 255))
    # Eye bags at y=12
    img.putpixel((9, 12), skin_shadow)
    img.putpixel((10, 12), skin_shadow)
    img.putpixel((13, 12), skin_shadow)
    img.putpixel((14, 12), skin_shadow)
    # Nose at y=12..13
    img.putpixel((11, 12), skin_high)
    img.putpixel((12, 12), skin_shadow)

    # Thick Batya Mustache at y=13
    for x in range(9, 15):
        img.putpixel((x, 13), mustache)
    img.putpixel((9, 14), mustache)
    img.putpixel((14, 14), mustache)

    # Mouth with GOLD TOOTH at y=14
    img.putpixel((10, 14), (145, 60, 55, 255)) # mouth
    img.putpixel((11, 14), gold_tooth)         # GOLD TOOTH!
    img.putpixel((12, 14), (245, 245, 245, 255))# normal tooth
    img.putpixel((13, 14), (145, 60, 55, 255)) # mouth

    # Chin / Stubble at y=15
    for x in range(8, 16):
        img.putpixel((x, 15), stubble)
    img.putpixel((11, 15), skin_shadow)
    img.putpixel((12, 15), skin_shadow)

    # Head Right (0..8, 8..16) & Left (16..24, 8..16) & Back (24..32, 8..16)
    for x in range(0, 8):
        for y in range(8, 16):
            c = hair_dark if y < 11 else (stubble if y >= 14 else skin)
            img.putpixel((x, y), c)
    for x in range(16, 24):
        for y in range(8, 16):
            c = hair_dark if y < 11 else (stubble if y >= 14 else skin)
            img.putpixel((x, y), c)
    for x in range(24, 32):
        for y in range(8, 16):
            c = hair_dark if y < 12 else (hair_gray if y < 14 else stubble)
            img.putpixel((x, y), c)
    # Head Top (8..16, 0..8) - thinning hair on crown
    for x in range(8, 16):
        for y in range(0, 8):
            img.putpixel((x, y), hair_dark if (x+y)%3 != 0 else hair_gray)
    # Head Bottom (16..24, 0..8) - neck base
    for x in range(16, 24):
        for y in range(0, 8):
            img.putpixel((x, y), skin_shadow)

    # 1b. HEAD OVERLAY: BLACK BERET / КЕПКА (32..64, 0..16)
    # Beret tilted over right side
    beret_dark = (20, 22, 26, 255)
    beret_mid = (32, 35, 42, 255)
    beret_high = (45, 48, 58, 255)
    # Hat top (40..48, 0..8)
    for x in range(40, 48):
        for y in range(0, 8):
            img.putpixel((x, y), beret_mid if (x+y)%2 == 0 else beret_dark)
    # Hat front (40..48, 8..16)
    for x in range(40, 48):
        img.putpixel((x, 8), beret_high)
        img.putpixel((x, 9), beret_mid)
    img.putpixel((40, 10), beret_dark)
    img.putpixel((41, 10), beret_dark)
    # Hat right, left, back rim
    for x in range(32, 40):
        for y in range(8, 10):
            img.putpixel((x, y), beret_mid)
    for x in range(48, 64):
        for y in range(8, 10):
            img.putpixel((x, y), beret_dark)

    # 2. TORSO (Base: 16..40, 16..32)
    # Telnyashka: alternating blue & white stripes
    for y in range(20, 32):
        stripe_blue = (y % 2 == 0)
        c_main = tel_blue if stripe_blue else tel_white
        c_shade = tel_blue_dark if stripe_blue else tel_white_dark
        # Front (20..28, 20..32)
        for x in range(20, 28):
            # V-neck chest opening at y=20..22, x=22..25
            if y == 20 and 22 <= x <= 25:
                img.putpixel((x, y), skin)
            elif y == 21 and 23 <= x <= 24:
                # Gold cross chain on chest
                img.putpixel((x, y), gold_chain)
            elif y == 22 and x == 23:
                # Prison tattoo on chest (cross)
                img.putpixel((x, y), tattoo_blue)
            elif y == 31: # Black belt with brass buckle
                img.putpixel((x, y), (205, 160, 45, 255) if 23 <= x <= 24 else (25, 25, 30, 255))
            else:
                col = c_shade if (x == 20 or x == 27) else c_main
                img.putpixel((x, y), col)
        # Back (32..40, 20..32)
        for x in range(32, 40):
            if y == 31:
                img.putpixel((x, y), (25, 25, 30, 255))
            else:
                col = c_shade if (x == 32 or x == 39) else c_main
                img.putpixel((x, y), col)
        # Right & Left sides of torso
        for x in range(16, 20):
            img.putpixel((x, y), (25, 25, 30, 255) if y == 31 else c_shade)
        for x in range(28, 32):
            img.putpixel((x, y), (25, 25, 30, 255) if y == 31 else c_shade)
    # Top of shoulders (20..28, 16..20)
    for x in range(20, 28):
        for y in range(16, 20):
            # Sleeveless: outer edges are skin (bare shoulders), inner is telnyashka straps
            if x == 20 or x == 27:
                img.putpixel((x, y), skin)
            else:
                img.putpixel((x, y), tel_blue if y % 2 == 0 else tel_white)

    # 3. RIGHT ARM (40..56, 16..32) - Muscular bare arm with forearm tattoo & watch!
    for y in range(20, 32):
        for x in range(40, 56):
            if y == 20: # shoulder strap of telnyashka
                img.putpixel((x, y), tel_blue)
            elif y < 29: # bare skin with tattoos
                col = skin
                if y == 24 and (x == 44 or x == 45): # forearm tattoo star
                    col = tattoo_blue
                elif y == 25 and x == 45:
                    col = tattoo_blue
                elif x in (40, 43, 47, 51, 55):
                    col = skin_shadow
                img.putpixel((x, y), col)
            elif y == 29: # Golden watch on right wrist!
                img.putpixel((x, y), gold_chain)
            else: # hand
                img.putpixel((x, y), skin_shadow if y == 31 else skin)
    for x in range(40, 48):
        for y in range(16, 20):
            img.putpixel((x, y), skin)

    # 4. LEFT ARM (32..48, 48..64) - Muscular bare arm with elbow spiderweb tattoo
    for y in range(52, 64):
        for x in range(32, 48):
            if y == 52:
                img.putpixel((x, y), tel_blue)
            elif y < 30 + 32: # y=53..61
                col = skin
                if y == 56 and (x == 36 or x == 37): # spiderweb tattoo
                    col = tattoo_blue
                elif y == 57 and x == 36:
                    col = tattoo_blue
                elif x in (32, 35, 39, 43, 47):
                    col = skin_shadow
                img.putpixel((x, y), col)
            else: # hand
                img.putpixel((x, y), skin_shadow if y == 63 else skin)
    for x in range(32, 40):
        for y in range(48, 52):
            img.putpixel((x, y), skin)

    # 5. RIGHT LEG (0..16, 16..32) - Dark Adidas tracksuit pants with side stripe + slipper
    for y in range(20, 32):
        for x in range(0, 16):
            if y >= 30: # black leather slipper with brown sole
                col = shoe_sole if y == 31 else leather_shoe
            else: # track pants
                # Side stripe on outer edge (x=0, 1 on right face; x=12 on back)
                if x in (0, 1): # side double stripe
                    col = pants_stripe
                else:
                    col = pants_light if y == 25 else (pants_mid if 4 <= x < 8 else pants_dark)
            img.putpixel((x, y), col)
    for x in range(0, 8):
        for y in range(16, 20):
            img.putpixel((x, y), pants_dark)

    # 6. LEFT LEG (16..32, 48..64) - Dark Adidas tracksuit pants with side stripe + slipper
    for y in range(52, 64):
        for x in range(16, 32):
            if y >= 62:
                col = shoe_sole if y == 63 else leather_shoe
            else:
                if x in (24, 25): # side double stripe
                    col = pants_stripe
                else:
                    col = pants_light if y == 57 else (pants_mid if 20 <= x < 24 else pants_dark)
            img.putpixel((x, y), col)
    for x in range(16, 24):
        for y in range(48, 52):
            img.putpixel((x, y), pants_dark)

    return img

def create_cyber_phantom():
    img = Image.new('RGBA', (64, 64), (0, 0, 0, 0))
    # Palette
    c_black = (18, 18, 22, 255)
    c_dark = (28, 30, 36, 255)
    c_carbon = (42, 45, 54, 255)
    c_gold = (255, 230, 0, 255)       # Cyber yellow
    c_gold_glow = (255, 245, 120, 255)
    c_cyan = (0, 220, 255, 255)
    skin = (210, 175, 145, 255)
    hair = (30, 32, 38, 255)

    # Head
    for y in range(8, 16):
        for x in range(8, 16):
            img.putpixel((x, y), skin)
    # Hair & Cyber Undercut
    for x in range(8, 16):
        img.putpixel((x, 8), hair)
        img.putpixel((x, 9), hair)
    img.putpixel((8, 10), hair)
    img.putpixel((15, 10), hair)
    # Cyber Visor / Glowing Yellow Eye on left side (x=13..15, y=10..11)
    img.putpixel((9, 11), (230, 230, 240, 255))
    img.putpixel((10, 11), (40, 140, 200, 255)) # normal eye
    img.putpixel((12, 11), c_gold)              # cyber eye
    img.putpixel((13, 11), c_gold_glow)
    img.putpixel((14, 11), c_gold)
    img.putpixel((13, 10), c_gold)              # cyber optical line
    img.putpixel((11, 14), (160, 80, 70, 255))  # mouth
    # Sides & back of head
    for x in range(0, 8):
        for y in range(8, 16): img.putpixel((x, y), hair if y < 12 else c_carbon)
    for x in range(16, 24):
        for y in range(8, 16): img.putpixel((x, y), hair if y < 12 else c_carbon)
    for x in range(24, 32):
        for y in range(8, 16): img.putpixel((x, y), hair)
    for x in range(8, 16):
        for y in range(0, 8): img.putpixel((x, y), hair)

    # 3D Hat Layer: Cyber Visor hardware & Headset
    for x in range(40, 48):
        img.putpixel((x, 8), c_dark)
        img.putpixel((x, 9), c_gold if x in (44, 45) else c_dark)
    img.putpixel((48, 11), c_gold) # ear implant
    img.putpixel((32, 11), c_gold)

    # Torso: Tactical Cyber-Coat with Yellow Glowing Collar & Center Zipper
    for y in range(20, 32):
        for x in range(16, 40):
            col = c_dark
            if 20 <= x < 28: # Front
                if y in (20, 21): col = c_gold if x in (21, 26) else c_black
                elif x in (23, 24): col = c_gold if y % 3 == 0 else c_gold_glow # Glowing yellow neon zipper
                elif y in (24, 28): col = c_carbon
                elif y == 31: col = c_gold # Utility belt
                else: col = c_black
            elif 32 <= x < 40: # Back
                if y in (24, 25) and 34 <= x <= 37: col = c_gold # Phantom logo on back!
                else: col = c_black
            img.putpixel((x, y), col)
    for x in range(20, 28):
        for y in range(16, 20): img.putpixel((x, y), c_gold if x in (21, 26) else c_black)

    # Torso Overlay (Jacket Collar)
    for y in range(36, 40):
        for x in range(20, 28):
            if y == 36: img.putpixel((x, y), c_gold) # Raised collar

    # Right Arm: Chrome / Carbon with Glowing Yellow Conduit
    for y in range(20, 32):
        for x in range(40, 56):
            col = c_carbon if y < 30 else c_black
            if x == 45 and y in (22, 23, 26, 27): col = c_gold # yellow power line
            img.putpixel((x, y), col)
    for x in range(40, 48):
        for y in range(16, 20): img.putpixel((x, y), c_black)

    # Left Arm: Sleek Black Tactical Sleeve & Cybernetic Glove
    for y in range(52, 64):
        for x in range(32, 48):
            col = c_black if y < 60 else c_carbon
            if x == 37 and y in (54, 55, 58, 59): col = c_gold
            img.putpixel((x, y), col)
    for x in range(32, 40):
        for y in range(48, 52): img.putpixel((x, y), c_black)

    # Legs: Dark Cargo Joggers with Glowing Knee Accents & Cyber Boots
    for y in range(20, 32):
        for x in range(0, 16):
            col = c_dark
            if y == 25 and 4 <= x < 8: col = c_gold # Knee neon line
            elif y >= 30: col = c_carbon if y == 30 else c_gold # Yellow sole
            img.putpixel((x, y), col)
    for x in range(0, 8):
        for y in range(16, 20): img.putpixel((x, y), c_black)

    for y in range(52, 64):
        for x in range(16, 32):
            col = c_dark
            if y == 57 and 20 <= x < 24: col = c_gold # Knee neon line
            elif y >= 62: col = c_carbon if y == 62 else c_gold
            img.putpixel((x, y), col)
    for x in range(16, 24):
        for y in range(48, 52): img.putpixel((x, y), c_black)

    return img

def create_streetwear():
    # Trendy two-tone hypebeast windbreaker (White & Crimson Red / Black)
    img = Image.new('RGBA', (64, 64), (0, 0, 0, 0))
    c_white = (245, 248, 252, 255)
    c_white_shade = (210, 215, 225, 255)
    c_red = (225, 38, 48, 255)
    c_red_shade = (175, 25, 35, 255)
    c_black = (24, 25, 30, 255)
    skin = (225, 178, 142, 255)
    hair = (48, 36, 28, 255)
    headphone_cyan = (0, 215, 235, 255)

    # Head
    for y in range(8, 16):
        for x in range(8, 16): img.putpixel((x, y), skin)
    # Trendy messy hair
    for x in range(8, 16):
        img.putpixel((x, 8), hair)
        img.putpixel((x, 9), hair)
    img.putpixel((8, 10), hair)
    img.putpixel((9, 10), hair)
    img.putpixel((14, 10), hair)
    img.putpixel((15, 10), hair)
    # Eyes
    img.putpixel((9, 11), (240, 240, 245, 255))
    img.putpixel((10, 11), (30, 40, 60, 255))
    img.putpixel((13, 11), (30, 40, 60, 255))
    img.putpixel((14, 11), (240, 240, 245, 255))
    img.putpixel((11, 14), (170, 95, 80, 255))
    for x in range(0, 8):
        for y in range(8, 16): img.putpixel((x, y), hair)
    for x in range(16, 24):
        for y in range(8, 16): img.putpixel((x, y), hair)
    for x in range(24, 32):
        for y in range(8, 16): img.putpixel((x, y), hair)
    for x in range(8, 16):
        for y in range(0, 8): img.putpixel((x, y), hair)

    # Hat Layer: Gaming / Street Headphones (Cyan & Black)
    for x in range(40, 48): img.putpixel((x, 8), c_black) # headphone band
    img.putpixel((32, 11), headphone_cyan) # earcups
    img.putpixel((32, 12), headphone_cyan)
    img.putpixel((48, 11), headphone_cyan)
    img.putpixel((48, 12), headphone_cyan)

    # Torso: Modern Windbreaker - Top half clean white, bottom half bold red & black blocks
    for y in range(20, 32):
        for x in range(16, 40):
            if y < 25: # Upper jacket: crisp white with black zipper & hoodie strings
                col = c_black if (x in (23, 24) and 20 <= x < 28) else c_white
            elif y < 29: # Mid jacket: bold crimson red
                col = c_red_shade if x in (16, 28, 32) else c_red
            else: # Lower jacket: dark black waistband
                col = c_black
            img.putpixel((x, y), col)
    for x in range(20, 28):
        for y in range(16, 20): img.putpixel((x, y), c_white)

    # Right Arm: Windbreaker Sleeve (White upper, Red forearm, watch on wrist, skin hand)
    for y in range(20, 32):
        for x in range(40, 56):
            if y < 25: col = c_white
            elif y < 29: col = c_red
            elif y == 29: col = (40, 220, 240, 255) # cyan smartwatch
            else: col = skin
            img.putpixel((x, y), col)
    for x in range(40, 48):
        for y in range(16, 20): img.putpixel((x, y), c_white)

    # Left Arm: Windbreaker Sleeve (White upper, Red forearm, skin hand)
    for y in range(52, 64):
        for x in range(32, 48):
            if y < 57: col = c_white
            elif y < 61: col = c_red
            else: col = skin
            img.putpixel((x, y), col)
    for x in range(32, 40):
        for y in range(48, 52): img.putpixel((x, y), c_white)

    # Legs: Ripped black skinny cargo pants + White chunky Nike-style sneakers
    for y in range(20, 32):
        for x in range(0, 16):
            if y >= 29: col = c_white if y == 31 else c_red # sneaker
            elif y == 24 and 4 <= x < 8: col = skin # ripped knee
            else: col = c_black
            img.putpixel((x, y), col)
    for x in range(0, 8):
        for y in range(16, 20): img.putpixel((x, y), c_black)

    for y in range(52, 64):
        for x in range(16, 32):
            if y >= 61: col = c_white if y == 63 else c_red
            elif y == 56 and 20 <= x < 24: col = skin
            else: col = c_black
            img.putpixel((x, y), col)
    for x in range(16, 24):
        for y in range(48, 52): img.putpixel((x, y), c_black)

    return img

def create_specops():
    # Elite Tactical SpecOps Operator
    img = Image.new('RGBA', (64, 64), (0, 0, 0, 0))
    c_camo1 = (45, 52, 42, 255)
    c_camo2 = (32, 38, 30, 255)
    c_vest = (25, 28, 24, 255)
    c_vest_pouch = (55, 62, 50, 255)
    c_balaclava = (35, 38, 36, 255)
    c_goggle = (40, 230, 90, 255) # Night vision green lens

    # Head (Balaclava + focused eyes)
    for y in range(8, 16):
        for x in range(0, 32): img.putpixel((x, y), c_balaclava)
    for x in range(8, 16):
        for y in range(0, 8): img.putpixel((x, y), c_camo2)
    # Eye slit at y=11
    for x in range(9, 15): img.putpixel((x, 11), (215, 170, 135, 255)) # skin around eyes
    img.putpixel((10, 11), (30, 45, 55, 255))
    img.putpixel((13, 11), (30, 45, 55, 255))

    # Hat layer: Tactical Helmet with NVG mount and green goggles
    for x in range(40, 48):
        img.putpixel((x, 8), c_camo1)
        img.putpixel((x, 9), c_goggle if 42 <= x <= 45 else c_camo1)
    img.putpixel((48, 10), c_camo2) # comms mic
    img.putpixel((47, 12), c_camo2)

    # Torso: Plate carrier vest with mag pouches and radio
    for y in range(20, 32):
        for x in range(16, 40):
            col = c_camo1 if (x+y)%2 == 0 else c_camo2
            if 20 <= x < 28: # Front plate carrier
                if 22 <= y <= 27: col = c_vest_pouch if y in (24, 25) else c_vest
                elif y == 28: col = (180, 140, 40, 255) # Shotgun shells / mags
                elif y >= 29: col = c_vest
            img.putpixel((x, y), col)
    for x in range(20, 28):
        for y in range(16, 20): img.putpixel((x, y), c_vest)

    # Arms: Camo combat shirt + tactical gloves
    for y in range(20, 32):
        for x in range(40, 56):
            col = (c_camo1 if y < 28 else c_vest)
            img.putpixel((x, y), col)
    for x in range(40, 48):
        for y in range(16, 20): img.putpixel((x, y), c_camo1)

    for y in range(52, 64):
        for x in range(32, 48):
            col = (c_camo1 if y < 60 else c_vest)
            img.putpixel((x, y), col)
    for x in range(32, 40):
        for y in range(48, 52): img.putpixel((x, y), c_camo1)

    # Legs: Camo combat trousers with kneepads & combat boots
    for y in range(20, 32):
        for x in range(0, 16):
            if y >= 29: col = c_vest
            elif y in (25, 26) and 4 <= x < 8: col = c_vest # kneepads
            else: col = c_camo1 if (x+y)%2 == 0 else c_camo2
            img.putpixel((x, y), col)
    for x in range(0, 8):
        for y in range(16, 20): img.putpixel((x, y), c_camo1)

    for y in range(52, 64):
        for x in range(16, 32):
            if y >= 61: col = c_vest
            elif y in (57, 58) and 20 <= x < 24: col = c_vest
            else: col = c_camo1 if (x+y)%2 == 0 else c_camo2
            img.putpixel((x, y), col)
    for x in range(16, 24):
        for y in range(48, 52): img.putpixel((x, y), c_camo1)

    return img

def main():
    skins_dir = 'phantom-launcher/assets/skins'
    os.makedirs(skins_dir, exist_ok=True)

    # 1. Steve (Official HD Masterpiece)
    if os.path.exists('ref_steve.png'):
        im = Image.open('ref_steve.png').convert('RGBA')
        im.save(os.path.join(skins_dir, 'steve.png'))
        print('Saved steve.png')

    # 2. Alex (Official HD Slim Masterpiece)
    if os.path.exists('ref_alex.png'):
        im = Image.open('ref_alex.png').convert('RGBA')
        im.save(os.path.join(skins_dir, 'alex.png'))
        print('Saved alex.png')

    # 3. Zek Batya
    im_zek = create_zek_batya()
    im_zek.save(os.path.join(skins_dir, 'zek_batya.png'))
    print('Saved zek_batya.png')

    # 4. Cyber Phantom
    im_cyber = create_cyber_phantom()
    im_cyber.save(os.path.join(skins_dir, 'cyber_phantom.png'))
    print('Saved cyber_phantom.png')

    # 5. Neon Streetwear
    im_street = create_streetwear()
    im_street.save(os.path.join(skins_dir, 'neon.png'))
    print('Saved neon.png')

    # 6. SpecOps
    im_specops = create_specops()
    im_specops.save(os.path.join(skins_dir, 'specops.png'))
    print('Saved specops.png')

if __name__ == '__main__':
    main()
