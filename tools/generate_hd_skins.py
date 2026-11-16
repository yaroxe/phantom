"""Generate release-quality 64x64 Minecraft skins with rich pixel shading and details."""
import os
from PIL import Image

def create_zek_batya():
    im = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
    p = im.putpixel

    # Skin tones
    SKIN_LIGHT = (215, 155, 125, 255)
    SKIN_MID   = (198, 138, 110, 255)
    SKIN_DARK  = (175, 118, 92, 255)
    SKIN_SHADOW= (150, 95, 75, 255)

    # Hair / Beret
    BERET_MAIN = (32, 35, 42, 255)
    BERET_DARK = (20, 22, 28, 255)
    BERET_EDGE = (45, 48, 56, 255)
    HAIR_GRAY  = (75, 75, 80, 255)
    HAIR_DARK  = (50, 50, 55, 255)

    # Teeth / Mustache
    GOLD_TOOTH = (235, 195, 40, 255)
    WHITE_TOOTH= (240, 240, 240, 255)
    STACHE_DARK= (40, 35, 32, 255)
    STACHE_MID = (65, 55, 48, 255)
    TATTOO_BLUE= (45, 70, 95, 255)

    # Telnyashka (striped sailor shirt)
    STRIPE_WHITE= (238, 240, 245, 255)
    STRIPE_SHADE= (205, 210, 220, 255)
    STRIPE_BLUE = (35, 85, 155, 255)
    STRIPE_BDARK= (25, 65, 125, 255)

    # Pants (black sweatpants / трико)
    PANTS_MID   = (38, 40, 48, 255)
    PANTS_DARK  = (26, 28, 34, 255)
    PANTS_LIGHT = (52, 55, 65, 255)
    PANTS_KNEE  = (48, 50, 60, 255)

    # Boots (black leather / slippers)
    BOOT_DARK   = (18, 18, 22, 255)
    BOOT_MID    = (28, 28, 34, 255)
    BOOT_SOLE   = (12, 12, 14, 255)

    # -------------------------------------------------------------
    # 1. HEAD (0..32, 0..16)
    # -------------------------------------------------------------
    # Top of head (8..16, 0..8): Black Beret
    for y in range(0, 8):
        for x in range(8, 16):
            c = BERET_MAIN if (x + y) % 3 != 0 else BERET_DARK
            p((x, y), c)
    p((11, 4), (55, 60, 70, 255)) # center knot/crease

    # Bottom of head / neck (16..24, 0..8): Neck skin
    for y in range(0, 8):
        for x in range(16, 24):
            c = SKIN_DARK if y > 4 else SKIN_MID
            p((x, y), c)

    # Head Sides:
    # Right side of head (0..8, 8..16)
    for y in range(8, 16):
        for x in range(0, 8):
            if y < 10:
                p((x, y), BERET_DARK if x > 4 else BERET_MAIN)
            elif y == 10:
                p((x, y), HAIR_GRAY if x % 2 == 0 else HAIR_DARK)
            else:
                p((x, y), SKIN_MID if (x + y) % 2 == 0 else SKIN_LIGHT)
    # Ear on right side
    p((4, 12), SKIN_SHADOW)
    p((4, 13), SKIN_DARK)

    # Front of head / Face (8..16, 8..16)
    for y in range(8, 16):
        for x in range(8, 16):
            if y == 8:
                # Beret brim
                p((x, y), BERET_EDGE)
            elif y == 9:
                # Forehead with wrinkles and "666" or prison tattoo!
                if x == 11 or x == 12 or x == 13:
                    p((x, y), TATTOO_BLUE) # Prison tattoo marks
                else:
                    p((x, y), SKIN_DARK if (x == 8 or x == 15) else SKIN_MID)
            elif y == 10:
                # Eyebrows & Eyes
                if x in (9, 10, 13, 14):
                    p((x, y), STACHE_DARK) # Bushy eyebrows
                else:
                    p((x, y), SKIN_MID)
            elif y == 11:
                # Eyes: White sclera + Brown/gray tired eyes
                if x == 9:  p((x, y), (230, 230, 235, 255))
                elif x == 10: p((x, y), (55, 45, 40, 255))
                elif x == 13: p((x, y), (55, 45, 40, 255))
                elif x == 14: p((x, y), (230, 230, 235, 255))
                else: p((x, y), SKIN_LIGHT) # Nose bridge
            elif y == 12:
                # Nose & cheeks
                if x in (11, 12):
                    p((x, y), (210, 135, 115, 255)) # Reddish nose
                elif x in (8, 15):
                    p((x, y), HAIR_GRAY) # Sideburns
                else:
                    p((x, y), SKIN_MID)
            elif y == 13:
                # Thick Mustache (Типичный батя!)
                if x in (9, 10, 11, 12, 13, 14):
                    p((x, y), STACHE_DARK if x in (11, 12) else STACHE_MID)
                else:
                    p((x, y), HAIR_GRAY)
            elif y == 14:
                # Mouth: Grin with GOLD TOOTH!
                if x == 10: p((x, y), GOLD_TOOTH)   # GOLD TOOTH!
                elif x == 11: p((x, y), WHITE_TOOTH)
                elif x in (12, 13): p((x, y), (40, 20, 20, 255)) # smile shadow
                elif x in (9, 14): p((x, y), STACHE_DARK) # mustache drooping edges
                else: p((x, y), SKIN_SHADOW)
            elif y == 15:
                # Chin with stubble / beard
                for cx in range(8, 16):
                    p((cx, y), HAIR_GRAY if cx % 2 == 0 else SKIN_DARK)

    # Left side of head (16..24, 8..16)
    for y in range(8, 16):
        for x in range(16, 24):
            if y < 10:
                p((x, y), BERET_MAIN)
            elif y == 10:
                p((x, y), HAIR_GRAY)
            else:
                p((x, y), SKIN_MID if (x + y) % 2 == 0 else SKIN_LIGHT)
    p((20, 12), SKIN_SHADOW)
    p((20, 13), SKIN_DARK)

    # Back of head (24..32, 8..16)
    for y in range(8, 16):
        for x in range(24, 32):
            if y < 11:
                p((x, y), BERET_DARK if y < 10 else BERET_EDGE)
            else:
                p((x, y), HAIR_DARK if y == 11 else (HAIR_GRAY if (x + y) % 2 == 0 else SKIN_DARK))

    # Outer Hat layer: 3D Beret overhang! (32..64, 0..16)
    for y in range(0, 8):
        for x in range(40, 48): # Top of 3D beret
            p((x, y), BERET_MAIN)
    # Beret edge puffing out on sides/front
    for x in range(40, 48):
        p((x, 8), BERET_DARK) # Front beret overhang
        p((x, 9), BERET_MAIN)
    for y in range(8, 12):
        p((34, y), BERET_MAIN) # Right beret puff
        p((35, y), BERET_DARK)
        p((48, y), BERET_MAIN) # Left beret puff
        p((49, y), BERET_DARK)

    # -------------------------------------------------------------
    # 2. TORSO (16..40, 16..32): TELNYASHKA (Striped tank top)
    # -------------------------------------------------------------
    # Top of shoulders (20..28, 16..20)
    for y in range(16, 20):
        for x in range(20, 28):
            if x in (20, 21, 26, 27): # Tank top straps
                p((x, y), STRIPE_BLUE if y % 2 == 0 else STRIPE_WHITE)
            else: # Bare neck / chest
                p((x, y), SKIN_MID if (x + y) % 2 == 0 else SKIN_LIGHT)

    # Front of Torso (20..28, 20..32)
    for y in range(20, 32):
        row_type = (y - 20) % 4
        is_blue = (row_type < 2)
        for x in range(20, 28):
            # Bare chest V-neck at top (y=20, 21)
            if y == 20 and x in (22, 23, 24, 25):
                p((x, y), SKIN_MID)
                continue
            if y == 21 and x in (23, 24):
                # Chest hair / tattoo crucifix chain!
                p((x, y), (200, 180, 50, 255) if x == 23 else TATTOO_BLUE)
                continue

            if is_blue:
                c = STRIPE_BLUE if (x + y) % 3 != 0 else STRIPE_BDARK
            else:
                c = STRIPE_WHITE if (x + y) % 3 != 0 else STRIPE_SHADE
            # Shade sides
            if x == 20 or x == 27:
                c = tuple(max(0, val - 25) for val in c[:3]) + (255,)
            p((x, y), c)

    # Back of Torso (32..40, 20..32)
    for y in range(20, 32):
        row_type = (y - 20) % 4
        is_blue = (row_type < 2)
        for x in range(32, 40):
            # Tattoos on back (Domed church / stars!)
            if y in (23, 24) and x in (35, 36):
                p((x, y), TATTOO_BLUE)
                continue
            c = STRIPE_BLUE if is_blue else STRIPE_WHITE
            if (x + y) % 4 == 0:
                c = STRIPE_BDARK if is_blue else STRIPE_SHADE
            p((x, y), c)

    # Torso Sides (16..20 and 28..32, 20..32)
    for y in range(20, 32):
        row_type = (y - 20) % 4
        is_blue = (row_type < 2)
        for x in list(range(16, 20)) + list(range(28, 32)):
            # Armpit / side is bare skin at top
            if y < 22:
                p((x, y), SKIN_DARK if (x + y) % 2 == 0 else SKIN_MID)
            else:
                p((x, y), STRIPE_BDARK if is_blue else STRIPE_SHADE)

    # Bottom of Torso / Belt (28..36, 16..20)
    for y in range(16, 20):
        for x in range(28, 36):
            p((x, y), (45, 40, 35, 255) if (x + y) % 2 == 0 else (30, 25, 20, 255))

    # -------------------------------------------------------------
    # 3. RIGHT ARM (40..56, 16..32): Muscular, bare with tattoos!
    # -------------------------------------------------------------
    for y in range(16, 32):
        for x in range(40, 56):
            if y < 20: # Shoulder
                p((x, y), SKIN_MID if (x + y) % 2 == 0 else SKIN_LIGHT)
            elif y in (21, 22) and x in (45, 46): # Shoulder tattoo ("СЕВЕР" / Stars)
                p((x, y), TATTOO_BLUE)
            elif y in (26, 27) and x in (44, 45): # Watch on wrist!
                p((x, y), (200, 190, 50, 255) if x == 44 else (40, 40, 45, 255))
            elif y >= 28: # Hand / fingers
                p((x, y), SKIN_MID if (x + y) % 2 == 0 else SKIN_LIGHT)
            else:
                p((x, y), SKIN_LIGHT if (x + y) % 3 != 0 else SKIN_MID)

    # -------------------------------------------------------------
    # 4. LEFT ARM (32..48, 48..64): Muscular, bare with forearm tattoo
    # -------------------------------------------------------------
    for y in range(48, 64):
        for x in range(32, 48):
            if y in (54, 55) and x in (37, 38): # Forearm tattoo ("666" or barbed wire)
                p((x, y), TATTOO_BLUE)
            elif y >= 60: # Hand
                p((x, y), SKIN_MID if (x + y) % 2 == 0 else SKIN_LIGHT)
            else:
                p((x, y), SKIN_LIGHT if (x + y) % 3 != 0 else SKIN_MID)

    # -------------------------------------------------------------
    # 5. RIGHT LEG (0..16, 16..32): Black sweatpants + Boot
    # -------------------------------------------------------------
    for y in range(16, 32):
        for x in range(0, 16):
            if y >= 28: # Black Boot / slipper
                p((x, y), BOOT_SOLE if y == 31 else (BOOT_MID if (x + y) % 2 == 0 else BOOT_DARK))
            else: # Sweatpants with side stripe (лампасы!)
                if x in (0, 8): # White stripe on side of sweatpants!
                    p((x, y), (220, 225, 235, 255))
                elif y in (22, 23): # Knee folds
                    p((x, y), PANTS_KNEE)
                else:
                    p((x, y), PANTS_MID if (x + y) % 2 == 0 else PANTS_DARK)

    # -------------------------------------------------------------
    # 6. LEFT LEG (16..32, 48..64): Black sweatpants + Boot
    # -------------------------------------------------------------
    for y in range(48, 64):
        for x in range(16, 32):
            if y >= 60: # Boot
                p((x, y), BOOT_SOLE if y == 63 else (BOOT_MID if (x + y) % 2 == 0 else BOOT_DARK))
            else: # Sweatpants with side stripe
                if x in (24, 31): # White stripe
                    p((x, y), (220, 225, 235, 255))
                elif y in (54, 55): # Knee folds
                    p((x, y), PANTS_KNEE)
                else:
                    p((x, y), PANTS_MID if (x + y) % 2 == 0 else PANTS_DARK)

    return im

def create_steve_hd():
    im = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
    p = im.putpixel

    # Steve Palette
    HAIR_D = (42, 28, 16, 255)
    HAIR_M = (62, 42, 24, 255)
    HAIR_L = (82, 56, 34, 255)
    SKIN_L = (218, 160, 130, 255)
    SKIN_M = (195, 138, 110, 255)
    SKIN_D = (168, 112, 85, 255)
    EYE_W  = (240, 242, 245, 255)
    EYE_B  = (45, 75, 170, 255)
    EYE_P  = (20, 35, 90, 255)
    BEARD  = (108, 68, 44, 255)

    SHIRT_L= (0, 192, 192, 255)
    SHIRT_M= (0, 162, 162, 255)
    SHIRT_D= (0, 132, 135, 255)
    SHIRT_S= (0, 105, 110, 255)

    JEANS_L= (68, 85, 165, 255)
    JEANS_M= (48, 62, 138, 255)
    JEANS_D= (34, 44, 105, 255)
    BELT   = (52, 38, 28, 255)
    BUCKLE = (205, 185, 70, 255)
    SHOE_M = (88, 88, 95, 255)
    SHOE_D = (55, 55, 62, 255)

    # HEAD (0..32, 0..16)
    # Top hair
    for y in range(0, 8):
        for x in range(8, 16):
            p((x, y), HAIR_M if (x + y) % 3 != 0 else (HAIR_D if (x*y)%2==0 else HAIR_L))
    # Bottom neck
    for y in range(0, 8):
        for x in range(16, 24):
            p((x, y), SKIN_D if y > 4 else SKIN_M)

    # Right head
    for y in range(8, 16):
        for x in range(0, 8):
            if y < 11 or (y == 11 and x < 4):
                p((x, y), HAIR_M if (x + y) % 2 == 0 else HAIR_D)
            else:
                p((x, y), SKIN_M if (x + y) % 2 == 0 else SKIN_L)

    # Front Face
    for y in range(8, 16):
        for x in range(8, 16):
            if y < 10 or (y == 10 and (x in (8, 15) or x in (11, 12))):
                p((x, y), HAIR_D if (x + y) % 2 == 0 else HAIR_M)
            elif y == 11: # Eyebrows & Eyes
                if x in (9, 14): p((x, y), EYE_W)
                elif x in (10, 13): p((x, y), EYE_B)
                else: p((x, y), SKIN_L)
            elif y == 12: # Nose
                p((x, y), SKIN_D if x in (11, 12) else SKIN_M)
            elif y == 13: # Beard / Mouth
                p((x, y), BEARD if x in (10, 11, 12, 13) else SKIN_M)
            elif y == 14: # Goatee
                p((x, y), BEARD if x in (9, 10, 11, 12, 13, 14) else SKIN_D)
            else: # Chin
                p((x, y), BEARD if x in (11, 12) else SKIN_D)

    # Left head
    for y in range(8, 16):
        for x in range(16, 24):
            if y < 11 or (y == 11 and x > 19):
                p((x, y), HAIR_M if (x + y) % 2 == 0 else HAIR_D)
            else:
                p((x, y), SKIN_M if (x + y) % 2 == 0 else SKIN_L)

    # Back head
    for y in range(8, 16):
        for x in range(24, 32):
            p((x, y), HAIR_D if (x + y) % 2 == 0 else HAIR_M)

    # 3D Hair layer (32..64, 0..16)
    for y in range(0, 8):
        for x in range(40, 48):
            p((x, y), HAIR_L if (x + y) % 3 == 0 else (HAIR_M if (x+y)%2==0 else HAIR_D))
    # Hair strands overhang
    for x in range(40, 48):
        if x in (40, 41, 46, 47): p((x, 8), HAIR_M)
    for y in range(8, 12):
        p((32, y), HAIR_D)
        p((49, y), HAIR_D)

    # TORSO (16..40, 16..32)
    # Front Torso (20..28, 20..32)
    for y in range(20, 32):
        for x in range(20, 28):
            if y < 22 and x in (22, 23, 24, 25): # V-neck skin
                p((x, y), SKIN_M if y == 21 else SKIN_L)
            elif y == 31: # Belt
                p((x, y), BUCKLE if x in (23, 24) else BELT)
            else: # Shirt with folds
                c = SHIRT_M
                if y in (22, 25, 28) and (x + y) % 2 == 0: c = SHIRT_L
                elif x in (20, 27) or y == 30: c = SHIRT_D
                p((x, y), c)

    # Back Torso (32..40, 20..32)
    for y in range(20, 32):
        for x in range(32, 40):
            p((x, y), BELT if y == 31 else (SHIRT_M if (x + y) % 2 == 0 else SHIRT_D))

    # Sides Torso (16..20 and 28..32)
    for y in range(20, 32):
        for x in list(range(16, 20)) + list(range(28, 32)):
            p((x, y), SHIRT_D if (x + y) % 2 == 0 else SHIRT_S)

    # ARMS (Right: 40..56, 16..32; Left: 32..48, 48..64)
    for y in range(16, 32):
        for x in range(40, 56):
            if y < 20: # Sleeve
                p((x, y), SHIRT_M if (x + y) % 2 == 0 else SHIRT_D)
            elif y == 20: # Sleeve cuff
                p((x, y), SHIRT_S)
            else: # Bare arm & hand
                p((x, y), SKIN_L if (x + y) % 3 != 0 else SKIN_M)

    for y in range(48, 64):
        for x in range(32, 48):
            if y < 52: # Sleeve
                p((x, y), SHIRT_M if (x + y) % 2 == 0 else SHIRT_D)
            elif y == 52:
                p((x, y), SHIRT_S)
            else:
                p((x, y), SKIN_L if (x + y) % 3 != 0 else SKIN_M)

    # LEGS (Right: 0..16, 16..32; Left: 16..32, 48..64)
    for y in range(16, 32):
        for x in range(0, 16):
            if y >= 29: # Shoes
                p((x, y), SHOE_D if y == 31 else (SHOE_M if (x + y) % 2 == 0 else SHOE_D))
            else: # Jeans
                c = JEANS_M
                if y in (21, 25): c = JEANS_L
                elif x in (0, 7, 8, 15) or y == 28: c = JEANS_D
                p((x, y), c)

    for y in range(48, 64):
        for x in range(16, 32):
            if y >= 61: # Shoes
                p((x, y), SHOE_D if y == 63 else (SHOE_M if (x + y) % 2 == 0 else SHOE_D))
            else:
                c = JEANS_M
                if y in (53, 57): c = JEANS_L
                elif x in (16, 23, 24, 31) or y == 60: c = JEANS_D
                p((x, y), c)

    return im

def create_street_hoodie():
    """Modern teen streetwear boy with white/black windbreaker and neon cyan accents."""
    im = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
    p = im.putpixel

    # Palette
    HAIR_DARK  = (30, 26, 32, 255)
    HAIR_MID   = (55, 48, 60, 255)
    HAIR_LIGHT = (85, 75, 92, 255)
    SKIN_L     = (235, 185, 160, 255)
    SKIN_M     = (210, 160, 135, 255)
    NEON_CYAN  = (0, 230, 240, 255)
    NEON_BLUE  = (0, 160, 220, 255)
    WHITE_JACK = (240, 242, 245, 255)
    WHITE_SHADE= (205, 210, 220, 255)
    BLACK_JACK = (32, 34, 40, 255)
    BLACK_DARK = (20, 22, 26, 255)
    PANTS_BLACK= (28, 30, 36, 255)
    PANTS_GRAY = (44, 46, 54, 255)
    SNEAKER_W  = (245, 245, 250, 255)

    # HEAD (0..32, 0..16) - Messy stylish dark anime hair with blue eyes
    for y in range(0, 8):
        for x in range(8, 16):
            p((x, y), HAIR_MID if (x + y) % 2 == 0 else HAIR_DARK)

    # Face
    for y in range(8, 16):
        for x in range(8, 16):
            if y < 10 or (y == 10 and (x in (8, 9, 14, 15))):
                p((x, y), HAIR_MID if (x + y) % 2 == 0 else HAIR_DARK)
            elif y == 11: # Blue expressive eyes
                if x in (9, 14): p((x, y), (240, 245, 255, 255))
                elif x in (10, 13): p((x, y), NEON_CYAN)
                else: p((x, y), SKIN_L)
            elif y == 12:
                p((x, y), SKIN_M if x in (11, 12) else SKIN_L)
            elif y == 13:
                p((x, y), (180, 110, 95, 255) if x in (11, 12) else SKIN_L)
            else:
                p((x, y), SKIN_M)

    # Sides & back of head
    for y in range(8, 16):
        for x in list(range(0, 8)) + list(range(16, 32)):
            p((x, y), HAIR_MID if (x + y) % 2 == 0 else HAIR_DARK)

    # Outer Hat layer: 3D hair spikes and headphones!
    for y in range(8, 14):
        # Neon headphones around neck/ears!
        p((33, y), NEON_CYAN) # Right earphone
        p((34, y), BLACK_JACK)
        p((49, y), BLACK_JACK) # Left earphone
        p((50, y), NEON_CYAN)

    # TORSO (16..40, 16..32): Color-block Windbreaker (White top, black bottom, neon stripe)
    for y in range(20, 32):
        for x in range(20, 28):
            if y < 24: # Upper white block
                p((x, y), WHITE_JACK if (x + y) % 2 == 0 else WHITE_SHADE)
            elif y == 24: # Neon cyan stripe across chest!
                p((x, y), NEON_CYAN if (x + y) % 3 != 0 else NEON_BLUE)
            else: # Lower black block with kangaroo pocket
                if y in (27, 28) and x in (22, 23, 24, 25):
                    p((x, y), BLACK_JACK) # Pocket
                else:
                    p((x, y), BLACK_DARK if (x + y) % 2 == 0 else BLACK_JACK)

    for y in range(20, 32):
        for x in range(32, 40): # Back
            if y < 24: p((x, y), WHITE_JACK)
            elif y == 24: p((x, y), NEON_CYAN)
            else: p((x, y), BLACK_JACK)

    for y in range(20, 32):
        for x in list(range(16, 20)) + list(range(28, 32)):
            p((x, y), BLACK_JACK if y >= 24 else WHITE_SHADE)

    # 3D Jacket layer (16..40, 32..48) - Windbreaker collar & hood overhang!
    for y in range(32, 40):
        for x in range(20, 28):
            if y < 35:
                p((x, y), WHITE_JACK)

    # ARMS
    for y in range(16, 32):
        for x in range(40, 56):
            if y < 22:
                p((x, y), WHITE_JACK if (x + y) % 2 == 0 else WHITE_SHADE)
            elif y == 22:
                p((x, y), NEON_CYAN)
            elif y < 27:
                p((x, y), BLACK_JACK if (x + y) % 2 == 0 else BLACK_DARK)
            elif y == 27 and x in (44, 45): # Smart watch!
                p((x, y), NEON_CYAN)
            else: # Hands
                p((x, y), SKIN_L if (x + y) % 2 == 0 else SKIN_M)

    for y in range(48, 64):
        for x in range(32, 48):
            if y < 54:
                p((x, y), WHITE_JACK)
            elif y == 54:
                p((x, y), NEON_CYAN)
            elif y < 59:
                p((x, y), BLACK_JACK)
            else:
                p((x, y), SKIN_L)

    # LEGS (Cargo sweatpants + Neon Sneaker)
    for y in range(16, 32):
        for x in range(0, 16):
            if y >= 29: # Sneakers
                p((x, y), SNEAKER_W if y == 31 else (NEON_CYAN if (x + y) % 2 == 0 else BLACK_JACK))
            else: # Cargo pants with pockets
                if y in (22, 23) and x in (2, 3, 4, 5):
                    p((x, y), PANTS_GRAY) # Pocket flap
                else:
                    p((x, y), PANTS_BLACK if (x + y) % 2 == 0 else PANTS_GRAY)

    for y in range(48, 64):
        for x in range(16, 32):
            if y >= 61:
                p((x, y), SNEAKER_W if y == 63 else (NEON_CYAN if (x + y) % 2 == 0 else BLACK_JACK))
            else:
                p((x, y), PANTS_BLACK if (x + y) % 2 == 0 else PANTS_GRAY)

    return im

def create_cyber_phantom():
    """Cyberpunk Phantom with Cyber Yellow glowing visor and carbon armor."""
    im = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
    p = im.putpixel

    CARBON_D = (18, 20, 24, 255)
    CARBON_M = (30, 32, 38, 255)
    CARBON_L = (48, 52, 62, 255)
    YELLOW_GLOW = (255, 225, 0, 255)
    YELLOW_HOT  = (255, 250, 120, 255)
    YELLOW_DARK = (185, 150, 0, 255)
    CHROME_L    = (210, 215, 225, 255)
    CHROME_M    = (145, 152, 165, 255)

    # HEAD: Tactical carbon helm + glowing visor
    for y in range(0, 16):
        for x in range(0, 32):
            p((x, y), CARBON_M if (x + y) % 2 == 0 else CARBON_D)

    # Face visor (8..16, 8..16)
    for y in range(8, 16):
        for x in range(8, 16):
            if y in (10, 11): # Glowing cyber visor!
                p((x, y), YELLOW_HOT if x in (11, 12) else YELLOW_GLOW)
            elif y == 12:
                p((x, y), YELLOW_DARK if x in (10, 11, 12, 13) else CARBON_D)
            elif y in (13, 14): # Respirator / rebreather grill
                p((x, y), CHROME_M if (x + y) % 2 == 0 else CARBON_D)
            else:
                p((x, y), CARBON_L if (x + y) % 2 == 0 else CARBON_M)

    # 3D Head layer: Cyber horn / sensors
    for y in range(0, 8):
        for x in range(40, 48):
            p((x, y), CARBON_D)
    p((40, 8), YELLOW_GLOW)
    p((47, 8), YELLOW_GLOW)

    # TORSO: Cyber armor with yellow conduits
    for y in range(20, 32):
        for x in range(20, 28):
            if (x in (21, 26) and y in (22, 23, 24, 25)) or (y == 25 and x in (22, 23, 24, 25)):
                # Power circuit line
                p((x, y), YELLOW_GLOW)
            elif y in (22, 23) and x in (23, 24):
                # Reactor core!
                p((x, y), YELLOW_HOT)
            else:
                p((x, y), CARBON_M if (x + y) % 2 == 0 else CARBON_D)

    for y in range(20, 32):
        for x in range(32, 40): # Back thruster
            if y in (23, 24) and x in (35, 36):
                p((x, y), YELLOW_GLOW)
            else:
                p((x, y), CARBON_D if (x + y) % 2 == 0 else CARBON_M)

    for y in range(20, 32):
        for x in list(range(16, 20)) + list(range(28, 32)):
            p((x, y), CARBON_D)

    # ARMS: Right arm = Carbon jacket with yellow stripe; Left arm = Full cybernetic chrome!
    for y in range(16, 32):
        for x in range(40, 56):
            if x in (45, 46) and y in range(20, 28):
                p((x, y), YELLOW_GLOW)
            else:
                p((x, y), CARBON_M if (x + y) % 2 == 0 else CARBON_D)

    # Chrome Cyber Arm (Left)
    for y in range(48, 64):
        for x in range(32, 48):
            if (x + y) % 4 == 0:
                p((x, y), YELLOW_GLOW)
            else:
                p((x, y), CHROME_L if (x + y) % 2 == 0 else CHROME_M)

    # LEGS: Cyber plated legs
    for y in range(16, 32):
        for x in range(0, 16):
            if y in (22, 23): p((x, y), CHROME_M) # Knee armor
            elif y in (24, 25) and x in (2, 3, 10, 11): p((x, y), YELLOW_GLOW)
            elif y >= 29: p((x, y), CARBON_D if y == 31 else CARBON_L)
            else: p((x, y), CARBON_M if (x + y) % 2 == 0 else CARBON_D)

    for y in range(48, 64):
        for x in range(16, 32):
            if y in (54, 55): p((x, y), CHROME_M)
            elif y in (56, 57) and x in (18, 19, 26, 27): p((x, y), YELLOW_GLOW)
            elif y >= 61: p((x, y), CARBON_D if y == 63 else CARBON_L)
            else: p((x, y), CARBON_M if (x + y) % 2 == 0 else CARBON_D)

    return im

def create_specops():
    """Tactical specops soldier with balaclava, comms and tactical vest."""
    im = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
    p = im.putpixel

    CAMO_D = (32, 38, 30, 255)
    CAMO_M = (48, 56, 42, 255)
    CAMO_L = (68, 78, 58, 255)
    BLACK_V = (22, 24, 26, 255)
    GRAY_V  = (38, 42, 46, 255)
    SKIN_M  = (195, 140, 115, 255)
    GOGGLE  = (40, 180, 120, 255)

    for y in range(0, 16):
        for x in range(0, 32):
            p((x, y), BLACK_V if (x + y) % 2 == 0 else GRAY_V)

    # Eye slit in balaclava (8..16, 8..16)
    for y in range(8, 16):
        for x in range(8, 16):
            if y == 11: # Eyes in slit
                if x in (10, 13): p((x, y), (40, 35, 30, 255))
                elif x in (9, 14): p((x, y), (230, 230, 230, 255))
                else: p((x, y), SKIN_M)
            else:
                p((x, y), BLACK_V if (x + y) % 2 == 0 else GRAY_V)

    # 3D Tactical Night Vision / Helmet
    for y in range(0, 8):
        for x in range(40, 48):
            p((x, y), CAMO_M)
    for x in range(40, 48):
        if x in (41, 42, 45, 46): p((x, 8), GOGGLE)

    # TORSO: Tactical Plate Carrier
    for y in range(20, 32):
        for x in range(20, 28):
            if y in (23, 24, 25) and x in (21, 22, 23, 24, 25, 26):
                p((x, y), GRAY_V if (x + y) % 2 == 0 else BLACK_V) # Mag pouches
            elif y == 31: # Duty belt
                p((x, y), (15, 16, 18, 255))
            else:
                p((x, y), CAMO_M if (x + y) % 2 == 0 else CAMO_D)

    for y in range(20, 32):
        for x in range(32, 40):
            p((x, y), GRAY_V if (x + y) % 2 == 0 else BLACK_V)

    for y in range(20, 32):
        for x in list(range(16, 20)) + list(range(28, 32)):
            p((x, y), CAMO_D)

    # ARMS: Camo sleeves + Tactical gloves
    for y in range(16, 32):
        for x in range(40, 56):
            if y >= 26: p((x, y), BLACK_V if (x + y) % 2 == 0 else GRAY_V) # Gloves
            else: p((x, y), CAMO_L if (x + y) % 3 == 0 else CAMO_M)

    for y in range(48, 64):
        for x in range(32, 48):
            if y >= 58: p((x, y), BLACK_V if (x + y) % 2 == 0 else GRAY_V)
            else: p((x, y), CAMO_L if (x + y) % 3 == 0 else CAMO_M)

    # LEGS: Camo pants + Knee pads + Combat boots
    for y in range(16, 32):
        for x in range(0, 16):
            if y in (22, 23): p((x, y), BLACK_V) # Knee pad
            elif y >= 28: p((x, y), (18, 18, 20, 255) if y == 31 else BLACK_V)
            else: p((x, y), CAMO_M if (x + y) % 2 == 0 else CAMO_D)

    for y in range(48, 64):
        for x in range(16, 32):
            if y in (54, 55): p((x, y), BLACK_V)
            elif y >= 60: p((x, y), (18, 18, 20, 255) if y == 63 else BLACK_V)
            else: p((x, y), CAMO_M if (x + y) % 2 == 0 else CAMO_D)

    return im

def create_alex_hd():
    """Authentic high detail Alex with orange hair braid and green tunic."""
    im = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
    p = im.putpixel

    HAIR_D = (155, 68, 24, 255)
    HAIR_M = (185, 88, 32, 255)
    HAIR_L = (215, 115, 45, 255)
    SKIN_L = (235, 175, 145, 255)
    SKIN_M = (210, 150, 122, 255)
    EYE_G  = (55, 155, 75, 255)
    EYE_W  = (240, 245, 240, 255)
    TUNIC_M= (88, 115, 62, 255)
    TUNIC_D= (65, 88, 45, 255)
    TUNIC_L= (108, 138, 78, 255)
    BELT   = (52, 38, 28, 255)
    PANTS  = (115, 85, 52, 255)
    BOOTS  = (68, 48, 30, 255)

    # HEAD
    for y in range(0, 8):
        for x in range(8, 16):
            p((x, y), HAIR_M if (x + y) % 2 == 0 else HAIR_D)

    # Face
    for y in range(8, 16):
        for x in range(8, 16):
            if y < 10:
                p((x, y), HAIR_M if (x + y) % 2 == 0 else HAIR_D)
            elif y == 11:
                if x in (9, 14): p((x, y), EYE_W)
                elif x in (10, 13): p((x, y), EYE_G)
                else: p((x, y), SKIN_L)
            elif y == 12:
                p((x, y), (225, 145, 125, 255) if x in (11, 12) else SKIN_L)
            elif y == 13:
                p((x, y), (195, 105, 95, 255) if x in (11, 12) else SKIN_M)
            else:
                p((x, y), SKIN_M)

    for y in range(8, 16):
        for x in list(range(0, 8)) + list(range(16, 32)):
            p((x, y), HAIR_M if (x + y) % 2 == 0 else HAIR_D)

    # TORSO: Green tunic with belt
    for y in range(20, 32):
        for x in range(20, 28):
            if y < 22 and x in (22, 23, 24, 25): p((x, y), SKIN_L)
            elif y == 30: p((x, y), (180, 150, 50, 255) if x in (23, 24) else BELT)
            else: p((x, y), TUNIC_M if (x + y) % 2 == 0 else TUNIC_D)

    for y in range(20, 32):
        for x in range(32, 40):
            p((x, y), BELT if y == 30 else (TUNIC_D if (x + y) % 2 == 0 else TUNIC_M))

    for y in range(20, 32):
        for x in list(range(16, 20)) + list(range(28, 32)):
            p((x, y), TUNIC_D)

    # ARMS (Slim 3px)
    for y in range(16, 32):
        for x in range(40, 56):
            if y < 21: p((x, y), TUNIC_M if (x + y) % 2 == 0 else TUNIC_D)
            else: p((x, y), SKIN_L if (x + y) % 2 == 0 else SKIN_M)

    for y in range(48, 64):
        for x in range(32, 48):
            if y < 53: p((x, y), TUNIC_M if (x + y) % 2 == 0 else TUNIC_D)
            else: p((x, y), SKIN_L if (x + y) % 2 == 0 else SKIN_M)

    # LEGS
    for y in range(16, 32):
        for x in range(0, 16):
            if y >= 28: p((x, y), BOOTS)
            else: p((x, y), PANTS if (x + y) % 2 == 0 else (95, 70, 42, 255))

    for y in range(48, 64):
        for x in range(16, 32):
            if y >= 60: p((x, y), BOOTS)
            else: p((x, y), PANTS if (x + y) % 2 == 0 else (95, 70, 42, 255))

    return im

def main():
    skins = [
        ("zek_batya.png", create_zek_batya()),
        ("steve.png", create_steve_hd()),
        ("neon.png", create_street_hoodie()),
        ("cyber_phantom.png", create_cyber_phantom()),
        ("specops.png", create_specops()),
        ("alex.png", create_alex_hd()),
    ]

    out_dirs = [
        "phantom-launcher/assets/skins",
        os.path.expandvars(r"%APPDATA%\phantom\skins_library"),
    ]

    for fname, img in skins:
        for d in out_dirs:
            os.makedirs(d, exist_ok=True)
            target = os.path.join(d, fname)
            img.save(target)
            print(f"Saved {fname} to {d}")

        # Also save localized names in skins_library
        loc_map = {
            "zek_batya.png": "Зек Батя.png",
            "steve.png": "Стив.png",
            "neon.png": "Ветровка Стрит.png",
            "cyber_phantom.png": "Кибер-фантом.png",
            "specops.png": "Спецназ.png",
            "alex.png": "Алекс.png",
        }
        if fname in loc_map:
            loc_target = os.path.join(os.path.expandvars(r"%APPDATA%\phantom\skins_library"), loc_map[fname])
            img.save(loc_target)
            print(f"Saved localized {loc_map[fname]}")

if __name__ == "__main__":
    main()
