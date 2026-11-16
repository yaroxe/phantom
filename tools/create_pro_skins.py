import os
from PIL import Image

def new_skin():
    return Image.new('RGBA', (64, 64), (0, 0, 0, 0))

def px(img, x, y, c):
    if 0 <= x < 64 and 0 <= y < 64:
        img.putpixel((x, y), c)

def rect(img, x0, y0, w, h, c):
    for y in range(y0, y0 + h):
        for x in range(x0, x0 + w):
            px(img, x, y, c)

def grid(img, x0, y0, pattern, palette):
    for dy, row in enumerate(pattern):
        for dx, ch in enumerate(row):
            if ch in palette and palette[ch] is not None:
                px(img, x0 + dx, y0 + dy, palette[ch])

# =========================================================================
# 1. ЗЕК БАТЯ (Типичный Батя из референса: тельняшка, берет, усы, тату 666, золотые зубы, часы)
# =========================================================================
def make_zek_batya():
    s = new_skin()

    # Colors
    SKIN = (226, 178, 142, 255)
    SKIN_LIGHT = (238, 192, 158, 255)
    SKIN_SHADE = (200, 150, 115, 255)
    SKIN_DARK = (175, 125, 90, 255)

    HAIR = (35, 25, 20, 255)
    STUBBLE = (165, 130, 105, 255)
    MUSTACHE = (40, 28, 20, 255)
    MUSTACHE_DARK = (25, 16, 12, 255)

    EYE_WHITE = (245, 245, 245, 255)
    EYE_PUPIL = (35, 45, 60, 255)
    EYEBROW = (30, 20, 15, 255)

    GOLD_TOOTH = (255, 215, 0, 255)
    GOLD_SHINE = (255, 248, 160, 255)
    LIP = (170, 100, 90, 255)

    TATTOO = (30, 60, 95, 255) # Синяя тюремная татуировка

    BERET = (25, 27, 32, 255)
    BERET_SHADE = (15, 16, 20, 255)
    BERET_LIGHT = (42, 45, 52, 255)

    WHITE_STRIPE = (240, 242, 248, 255)
    WHITE_STRIPE_S = (210, 214, 222, 255)
    BLUE_STRIPE = (22, 55, 130, 255)
    BLUE_STRIPE_S = (14, 38, 95, 255)

    PANTS = (20, 22, 28, 255)
    PANTS_SHADE = (12, 14, 18, 255)
    PANTS_STRIPE = (235, 235, 240, 255) # Белый лампас

    GOLD_WATCH = (245, 200, 35, 255)
    WATCH_FACE = (255, 255, 255, 255)
    WATCH_SHADE = (190, 150, 20, 255)

    BOOTS = (16, 16, 18, 255)
    BOOTS_LACE = (60, 62, 70, 255)

    # --- 1. HEAD BASE ---
    # Top of head (hair/scalp)
    rect(s, 8, 0, 8, 8, HAIR)
    # Bottom of head (neck)
    rect(s, 16, 0, 8, 8, SKIN_SHADE)
    # Right side head
    rect(s, 0, 8, 8, 8, SKIN)
    rect(s, 0, 8, 8, 3, HAIR)
    # Left side head
    rect(s, 16, 8, 8, 8, SKIN)
    rect(s, 16, 8, 8, 3, HAIR)
    # Back head
    rect(s, 24, 8, 8, 8, HAIR)
    rect(s, 24, 14, 8, 2, SKIN_SHADE)

    # Front Face (8..16, 8..16)
    rect(s, 8, 8, 8, 8, SKIN)
    # Forehead hair
    rect(s, 8, 8, 8, 2, HAIR)
    px(s, 8, 10, HAIR); px(s, 15, 10, HAIR)

    # Tattoo '666' on right forehead / temple (x=9..11, y=9..10)
    px(s, 9, 9, TATTOO); px(s, 10, 9, TATTOO); px(s, 11, 9, TATTOO)
    px(s, 9, 10, TATTOO); px(s, 10, 10, TATTOO); px(s, 11, 10, TATTOO)

    # Eyebrows (y=11)
    rect(s, 9, 11, 2, 1, EYEBROW)
    rect(s, 13, 11, 2, 1, EYEBROW)

    # Eyes (y=12): 2-pixel wide eyes
    px(s, 9, 12, EYE_WHITE); px(s, 10, 12, EYE_PUPIL)
    px(s, 13, 12, EYE_PUPIL); px(s, 14, 12, EYE_WHITE)

    # Nose (y=13)
    px(s, 11, 13, SKIN_LIGHT); px(s, 12, 13, SKIN_SHADE)

    # Mustache (y=14): пышные гусарские/батины усы
    rect(s, 9, 14, 6, 1, MUSTACHE)
    px(s, 9, 15, MUSTACHE_DARK); px(s, 14, 15, MUSTACHE_DARK)

    # Mouth with Gold Tooth (y=15): хитрый оскал / ухмылка
    px(s, 10, 15, LIP)
    px(s, 11, 15, GOLD_TOOTH); px(s, 12, 15, GOLD_SHINE) # Золотые зубы!
    px(s, 13, 15, LIP)

    # Щетина по бокам лица (stubble)
    px(s, 8, 13, STUBBLE); px(s, 8, 14, STUBBLE); px(s, 8, 15, STUBBLE)
    px(s, 15, 13, STUBBLE); px(s, 15, 14, STUBBLE); px(s, 15, 15, STUBBLE)

    # --- 2. HEAD 2nd LAYER (Чёрный берет набекрень - Hat layer 40..48, 8..16) ---
    # Top of beret (40, 0, 8, 8)
    rect(s, 40, 0, 8, 8, BERET)
    px(s, 43, 3, BERET_LIGHT); px(s, 44, 3, BERET_LIGHT); px(s, 43, 4, BERET_LIGHT)

    # Front of beret (40, 8, 8, 4) - берет сдвинут набекрень
    rect(s, 40, 8, 8, 3, BERET)
    rect(s, 40, 11, 4, 1, BERET_LIGHT) # Козырёк/залом берета слева
    # Right of beret
    rect(s, 32, 8, 8, 4, BERET_SHADE)
    # Left of beret
    rect(s, 48, 8, 8, 3, BERET)
    # Back of beret
    rect(s, 56, 8, 8, 4, BERET)

    # --- 3. TORSO (Тельняшка без рукавов) ---
    # Torso Front (20, 20, 8, 12)
    # Neck / Collar opening (v-neck showing chest)
    rect(s, 20, 20, 8, 12, WHITE_STRIPE)
    # Horizontal blue stripes
    for y in [22, 23, 25, 26, 28, 29, 31]:
        rect(s, 20, y, 8, 1, BLUE_STRIPE)
    # Open neck / chest (y=20, 21)
    rect(s, 22, 20, 4, 1, SKIN_LIGHT)
    rect(s, 23, 21, 2, 1, SKIN_SHADE)

    # Torso Back (32, 20, 8, 12)
    rect(s, 32, 20, 8, 12, WHITE_STRIPE_S)
    for y in [22, 23, 25, 26, 28, 29, 31]:
        rect(s, 32, y, 8, 1, BLUE_STRIPE_S)

    # Torso Sides (Right: 16, 20, 4, 12; Left: 28, 20, 4, 12)
    # Sleeveless cutout: armpit skin visible at top (y=20..22)
    rect(s, 16, 20, 4, 3, SKIN_SHADE)
    rect(s, 16, 23, 4, 9, WHITE_STRIPE)
    for y in [24, 26, 28, 30]:
        rect(s, 16, y, 4, 1, BLUE_STRIPE)

    rect(s, 28, 20, 4, 3, SKIN_SHADE)
    rect(s, 28, 23, 4, 9, WHITE_STRIPE)
    for y in [24, 26, 28, 30]:
        rect(s, 28, y, 4, 1, BLUE_STRIPE)

    # Torso Top (20, 16, 8, 4) - shoulder straps of telnyashka + neck
    rect(s, 20, 16, 8, 4, WHITE_STRIPE)
    rect(s, 20, 17, 8, 1, BLUE_STRIPE)
    rect(s, 22, 17, 4, 2, SKIN_SHADE)

    # Torso Bottom (28, 16, 8, 4) - belt line
    rect(s, 28, 16, 8, 4, PANTS_SHADE)

    # --- 4. RIGHT ARM (Открытая мускулистая рука) ---
    # Right Arm (40..56, 16..32)
    rect(s, 40, 20, 16, 12, SKIN)
    rect(s, 44, 16, 8, 4, SKIN) # Top/Bottom
    # Muscular shading on right arm
    for y in range(20, 24):
        px(s, 44, y, SKIN_LIGHT); px(s, 47, y, SKIN_SHADE)
    # Tattoo on right bicep
    px(s, 41, 23, TATTOO); px(s, 42, 23, TATTOO); px(s, 41, 24, TATTOO)

    # --- 5. LEFT ARM (С золотыми командирскими часами на запястье!) ---
    # Left Arm (32..48, 48..64)
    rect(s, 32, 52, 16, 12, SKIN)
    rect(s, 36, 48, 8, 4, SKIN)
    # Shading
    for y in range(52, 56):
        px(s, 36, y, SKIN_LIGHT); px(s, 39, y, SKIN_SHADE)

    # Golden Watch on Left Wrist (y=59, 60):
    # Front of wrist (36..40, 59..60)
    rect(s, 36, 59, 4, 2, GOLD_WATCH)
    px(s, 37, 59, WATCH_FACE); px(s, 38, 59, WATCH_FACE)
    px(s, 37, 60, WATCH_SHADE)
    # Sides & Back of strap
    rect(s, 32, 59, 4, 2, GOLD_WATCH)
    rect(s, 40, 59, 4, 2, GOLD_WATCH)
    rect(s, 44, 59, 4, 2, GOLD_WATCH)

    # --- 6. LEGS (Спортивные штаны с белыми лампасами) ---
    # Right Leg (0..16, 16..32)
    rect(s, 0, 20, 16, 12, PANTS)
    rect(s, 4, 16, 8, 4, PANTS_SHADE)
    # White stripe on outer side of right leg (x=0, y=20..30)
    for y in range(20, 30):
        px(s, 0, y, PANTS_STRIPE)
    # Black boots (y=30..32)
    rect(s, 0, 30, 16, 2, BOOTS)
    px(s, 5, 30, BOOTS_LACE); px(s, 6, 30, BOOTS_LACE)

    # Left Leg (16..32, 48..64)
    rect(s, 16, 52, 16, 12, PANTS)
    rect(s, 20, 48, 8, 4, PANTS_SHADE)
    # White stripe on outer side of left leg (x=27, y=52..62)
    for y in range(52, 62):
        px(s, 27, y, PANTS_STRIPE)
    # Black boots (y=62..64)
    rect(s, 16, 62, 16, 2, BOOTS)
    px(s, 21, 62, BOOTS_LACE); px(s, 22, 62, BOOTS_LACE)

    return s

# =========================================================================
# 2. СТИВ (Классический мастерски детализированный Steve)
# =========================================================================
def make_steve():
    s = new_skin()

    SKIN = (185, 126, 98, 255)
    SKIN_L = (205, 145, 115, 255)
    SKIN_S = (155, 98, 72, 255)
    HAIR = (45, 32, 18, 255)
    HAIR_L = (62, 44, 25, 255)
    EYE_W = (255, 255, 255, 255)
    EYE_B = (45, 75, 175, 255)
    BEARD = (105, 68, 46, 255)

    TEAL = (0, 168, 168, 255)
    TEAL_L = (20, 195, 195, 255)
    TEAL_S = (0, 135, 135, 255)
    TEAL_D = (0, 110, 110, 255)

    JEANS = (45, 55, 135, 255)
    JEANS_L = (60, 75, 165, 255)
    JEANS_S = (32, 40, 105, 255)
    BELT = (30, 22, 15, 255)
    BUCKLE = (200, 190, 80, 255)
    SHOES = (60, 60, 65, 255)

    # Head
    rect(s, 8, 0, 8, 8, HAIR)
    rect(s, 16, 0, 8, 8, SKIN_S)
    rect(s, 0, 8, 8, 8, SKIN)
    rect(s, 0, 8, 8, 4, HAIR)
    rect(s, 16, 8, 8, 8, SKIN)
    rect(s, 16, 8, 8, 4, HAIR)
    rect(s, 24, 8, 8, 8, HAIR)

    # Face (8..16, 8..16)
    rect(s, 8, 8, 8, 8, SKIN)
    rect(s, 8, 8, 8, 2, HAIR)
    px(s, 8, 10, HAIR); px(s, 15, 10, HAIR)
    # Eyes
    px(s, 9, 12, EYE_W); px(s, 10, 12, EYE_B)
    px(s, 13, 12, EYE_B); px(s, 14, 12, EYE_W)
    # Nose
    px(s, 11, 13, SKIN_L); px(s, 12, 13, SKIN_S)
    # Beard
    rect(s, 10, 14, 4, 1, BEARD)
    rect(s, 9, 15, 6, 1, BEARD)
    px(s, 11, 14, (135, 78, 55, 255)); px(s, 12, 14, (135, 78, 55, 255))

    # Torso
    rect(s, 20, 20, 8, 12, TEAL)
    rect(s, 32, 20, 8, 12, TEAL_S)
    rect(s, 16, 20, 4, 12, TEAL_S)
    rect(s, 28, 20, 4, 12, TEAL_S)
    rect(s, 20, 16, 8, 4, TEAL_L)
    rect(s, 28, 16, 8, 4, BELT)

    # V-Neck
    rect(s, 22, 20, 4, 2, SKIN)
    px(s, 23, 21, SKIN_S); px(s, 24, 21, SKIN_S)
    # Belt & Buckle
    rect(s, 20, 31, 8, 1, BELT)
    px(s, 23, 31, BUCKLE); px(s, 24, 31, BUCKLE)
    # Folds on shirt
    rect(s, 20, 26, 8, 1, TEAL_S)
    rect(s, 21, 28, 6, 1, TEAL_L)

    # Right Arm
    rect(s, 40, 20, 16, 4, TEAL)
    rect(s, 40, 24, 16, 8, SKIN)
    rect(s, 44, 16, 8, 4, TEAL_L)
    rect(s, 44, 24, 4, 1, TEAL_S) # Sleeve edge

    # Left Arm
    rect(s, 32, 52, 16, 4, TEAL)
    rect(s, 32, 56, 16, 8, SKIN)
    rect(s, 36, 48, 8, 4, TEAL_L)
    rect(s, 36, 56, 4, 1, TEAL_S)

    # Right Leg
    rect(s, 0, 20, 16, 10, JEANS)
    rect(s, 4, 16, 8, 4, JEANS_S)
    rect(s, 0, 30, 16, 2, SHOES)
    # Pocket seams
    px(s, 4, 21, JEANS_L); px(s, 5, 21, JEANS_L)
    # Knee fold
    rect(s, 4, 25, 4, 1, JEANS_S)

    # Left Leg
    rect(s, 16, 52, 16, 10, JEANS)
    rect(s, 20, 48, 8, 4, JEANS_S)
    rect(s, 16, 62, 16, 2, SHOES)
    px(s, 22, 53, JEANS_L); px(s, 23, 53, JEANS_L)
    rect(s, 20, 57, 4, 1, JEANS_S)

    return s

# =========================================================================
# 3. АЛЕКС (Alex Slim с косой на 2 слое, изумрудными глазами и сапогами)
# =========================================================================
def make_alex():
    s = new_skin()

    SKIN = (235, 185, 150, 255)
    SKIN_L = (248, 198, 165, 255)
    SKIN_S = (210, 160, 125, 255)

    GINGER = (175, 85, 30, 255)
    GINGER_L = (205, 110, 45, 255)
    GINGER_S = (145, 65, 20, 255)

    EYE_W = (255, 255, 255, 255)
    EYE_G = (40, 145, 75, 255) # Изумрудные глаза

    SHIRT = (100, 130, 75, 255)
    SHIRT_L = (120, 155, 90, 255)
    SHIRT_S = (80, 105, 60, 255)

    PANTS = (120, 75, 50, 255)
    PANTS_S = (95, 58, 38, 255)
    BOOTS = (75, 75, 80, 255)
    BOOTS_L = (95, 95, 100, 255)

    # Head
    rect(s, 8, 0, 8, 8, GINGER)
    rect(s, 16, 0, 8, 8, SKIN_S)
    rect(s, 0, 8, 8, 8, SKIN)
    rect(s, 0, 8, 8, 4, GINGER)
    rect(s, 16, 8, 8, 8, SKIN)
    rect(s, 16, 8, 8, 4, GINGER)
    rect(s, 24, 8, 8, 8, GINGER)

    # Face
    rect(s, 8, 8, 8, 8, SKIN)
    rect(s, 8, 8, 8, 2, GINGER)
    px(s, 8, 10, GINGER); px(s, 9, 10, GINGER); px(s, 15, 10, GINGER)
    # Eyes
    px(s, 9, 12, EYE_W); px(s, 10, 12, EYE_G)
    px(s, 13, 12, EYE_G); px(s, 14, 12, EYE_W)
    # Cheeks
    px(s, 8, 13, (245, 160, 145, 255)); px(s, 15, 13, (245, 160, 145, 255))
    # Lips
    px(s, 11, 14, (200, 120, 110, 255)); px(s, 12, 14, (200, 120, 110, 255))

    # Hat 2nd layer: Объемные волосы и косичка
    rect(s, 40, 0, 8, 8, GINGER)
    rect(s, 40, 8, 8, 3, GINGER_L)
    rect(s, 32, 8, 8, 4, GINGER)
    rect(s, 48, 8, 8, 5, GINGER) # Свисающая коса слева!
    rect(s, 56, 8, 8, 6, GINGER_S)

    # Torso
    rect(s, 20, 20, 8, 12, SHIRT)
    rect(s, 32, 20, 8, 12, SHIRT_S)
    rect(s, 16, 20, 4, 12, SHIRT_S)
    rect(s, 28, 20, 4, 12, SHIRT_S)
    rect(s, 20, 16, 8, 4, SHIRT_L)
    rect(s, 28, 16, 8, 4, PANTS_S)
    # Belt
    rect(s, 20, 31, 8, 1, (75, 45, 30, 255))

    # Right Arm (Slim 3px wide: x=44..47)
    rect(s, 40, 20, 16, 4, SHIRT)
    rect(s, 40, 24, 16, 8, SKIN)

    # Left Arm (Slim 3px wide)
    rect(s, 32, 52, 16, 4, SHIRT)
    rect(s, 32, 56, 16, 8, SKIN)

    # Legs & Boots
    rect(s, 0, 20, 16, 8, PANTS)
    rect(s, 0, 28, 16, 4, BOOTS) # Высокие сапоги
    rect(s, 16, 52, 16, 8, PANTS)
    rect(s, 16, 60, 16, 4, BOOTS)

    return s

# =========================================================================
# 4. ВЕТРОВКА СТРИТ (Современный стритвир: черно-красная ветровка, капюшон, джоггеры)
# =========================================================================
def make_streetwear():
    s = new_skin()

    SKIN = (230, 185, 150, 255)
    HAIR = (40, 30, 25, 255)
    HAIR_TINT = (80, 50, 40, 255)
    EYE_B = (50, 80, 120, 255)

    JACKET_BLACK = (28, 28, 34, 255)
    JACKET_SHADE = (18, 18, 22, 255)
    ACCENT_RED = (230, 45, 55, 255)
    ACCENT_WHITE = (245, 245, 250, 255)
    ZIPPER = (190, 195, 205, 255)

    JOGGERS = (22, 24, 30, 255)
    SNEAKER_WHITE = (250, 250, 255, 255)
    SNEAKER_RED = (225, 40, 50, 255)

    # Head & Stylish undercut hair
    rect(s, 8, 0, 8, 8, HAIR)
    rect(s, 16, 0, 8, 8, SKIN)
    rect(s, 0, 8, 8, 8, SKIN)
    rect(s, 0, 8, 8, 3, HAIR)
    rect(s, 16, 8, 8, 8, SKIN)
    rect(s, 16, 8, 8, 3, HAIR)
    rect(s, 24, 8, 8, 8, HAIR)

    # Face
    rect(s, 8, 8, 8, 8, SKIN)
    rect(s, 8, 8, 8, 2, HAIR)
    px(s, 8, 10, HAIR); px(s, 9, 10, HAIR_TINT)
    # Eyes
    px(s, 9, 12, (255, 255, 255, 255)); px(s, 10, 12, EYE_B)
    px(s, 13, 12, EYE_B); px(s, 14, 12, (255, 255, 255, 255))
    # Mouth
    px(s, 11, 14, (180, 110, 100, 255)); px(s, 12, 14, (180, 110, 100, 255))

    # Hat layer: объемные волосы челка + наушники на шее
    rect(s, 40, 0, 8, 8, HAIR)
    rect(s, 40, 8, 8, 3, HAIR_TINT)
    px(s, 40, 11, ACCENT_RED); px(s, 47, 11, ACCENT_RED) # Наушники!

    # Torso: Стильная ветровка
    rect(s, 20, 20, 8, 12, JACKET_BLACK)
    rect(s, 32, 20, 8, 12, JACKET_SHADE)
    rect(s, 16, 20, 4, 12, JACKET_BLACK)
    rect(s, 28, 20, 4, 12, JACKET_BLACK)
    rect(s, 20, 16, 8, 4, JACKET_BLACK)

    # Красная диагональная полоса и молния
    rect(s, 20, 23, 8, 2, ACCENT_RED)
    rect(s, 20, 25, 8, 1, ACCENT_WHITE)
    # Молния (Zipper)
    for y in range(21, 32):
        px(s, 23, y, ZIPPER); px(s, 24, y, ZIPPER)
    # Белая футболка под ветровкой
    rect(s, 22, 20, 4, 2, (245, 245, 245, 255))

    # Jacket 2nd Layer (объемные карманы и воротник)
    rect(s, 20, 36, 8, 12, (0, 0, 0, 0))
    rect(s, 20, 36, 8, 2, JACKET_SHADE) # Воротник
    rect(s, 21, 44, 2, 3, ACCENT_RED) # Карман правый
    rect(s, 25, 44, 2, 3, ACCENT_RED) # Карман левый

    # Arms
    rect(s, 40, 20, 16, 12, JACKET_BLACK)
    rect(s, 40, 24, 16, 2, ACCENT_RED) # Полоска на рукаве
    rect(s, 44, 30, 4, 2, SKIN) # Кисти рук

    rect(s, 32, 52, 16, 12, JACKET_BLACK)
    rect(s, 32, 56, 16, 2, ACCENT_RED)
    rect(s, 36, 62, 4, 2, SKIN)

    # Legs & Sneaker
    rect(s, 0, 20, 16, 9, JOGGERS)
    rect(s, 0, 29, 16, 3, SNEAKER_WHITE)
    rect(s, 4, 31, 4, 1, SNEAKER_RED)

    rect(s, 16, 52, 16, 9, JOGGERS)
    rect(s, 16, 61, 16, 3, SNEAKER_WHITE)
    rect(s, 20, 63, 4, 1, SNEAKER_RED)

    return s

# =========================================================================
# 5. КИБЕР-ФАНТОМ (Cyberpunk кибер-ассасин с фирменным Cyber-Yellow #FFE600)
# =========================================================================
def make_cyber_phantom():
    s = new_skin()

    CYBER_YELLOW = (255, 230, 0, 255)
    CYBER_YELLOW_L = (255, 245, 100, 255)
    CYBER_YELLOW_D = (195, 170, 0, 255)

    CARBON_BLACK = (20, 22, 26, 255)
    CARBON_DARK = (12, 14, 17, 255)
    TECH_GREY = (55, 60, 72, 255)
    CHROME = (180, 195, 210, 255)

    SKIN = (215, 175, 145, 255)

    # Head
    rect(s, 8, 0, 8, 8, CARBON_BLACK)
    rect(s, 16, 0, 8, 8, CARBON_DARK)
    rect(s, 0, 8, 8, 8, CARBON_BLACK)
    rect(s, 16, 8, 8, 8, CARBON_BLACK)
    rect(s, 24, 8, 8, 8, CARBON_BLACK)

    # Face: Кибернетический визор / маска
    rect(s, 8, 8, 8, 8, CARBON_DARK)
    # Glowing Cyber-Yellow Visor
    rect(s, 8, 11, 8, 2, CYBER_YELLOW)
    px(s, 11, 11, CYBER_YELLOW_L); px(s, 12, 11, CYBER_YELLOW_L)
    # Respirator / cyber jaw
    rect(s, 9, 14, 6, 2, TECH_GREY)
    px(s, 11, 14, CYBER_YELLOW_D); px(s, 12, 14, CYBER_YELLOW_D)

    # Hat 2nd layer: Объемные кибер-линзы и капюшон
    rect(s, 40, 0, 8, 8, CARBON_BLACK)
    rect(s, 40, 8, 8, 4, CARBON_BLACK)
    rect(s, 40, 11, 8, 2, CYBER_YELLOW_L) # Светящийся оверлей

    # Torso: Techwear тактическая броня Phantom Engine
    rect(s, 20, 20, 8, 12, CARBON_BLACK)
    rect(s, 32, 20, 8, 12, CARBON_DARK)
    rect(s, 16, 20, 4, 12, CARBON_BLACK)
    rect(s, 28, 20, 4, 12, CARBON_BLACK)

    # Логотип Phantom 'P' и неоновые проводники на груди
    rect(s, 22, 22, 4, 1, CYBER_YELLOW)
    px(s, 22, 23, CYBER_YELLOW); px(s, 25, 23, CYBER_YELLOW)
    rect(s, 22, 24, 4, 1, CYBER_YELLOW)
    px(s, 22, 25, CYBER_YELLOW) # Логотип P!
    rect(s, 20, 28, 8, 1, CYBER_YELLOW_D) # Неоновый пояс

    # Torso 2nd Layer (куртка-плащ)
    rect(s, 20, 36, 8, 12, CARBON_DARK)
    rect(s, 22, 36, 4, 12, (0, 0, 0, 0)) # Распахнутый плащ спереди
    for y in range(36, 48):
        px(s, 21, y, CYBER_YELLOW); px(s, 26, y, CYBER_YELLOW) # Неоновый кант плаща!

    # Right Arm: Кибернетический хромированный протез
    rect(s, 40, 20, 16, 12, CHROME)
    rect(s, 40, 24, 16, 2, CYBER_YELLOW) # Светящаяся шина данных

    # Left Arm: Карбоновый рукав
    rect(s, 32, 52, 16, 12, CARBON_BLACK)
    rect(s, 32, 56, 16, 2, TECH_GREY)

    # Legs: Бронированные экзоскелетные штаны
    rect(s, 0, 20, 16, 12, CARBON_BLACK)
    for y in range(21, 31):
        px(s, 4, y, CYBER_YELLOW_D) # Неоновая подсветка ног
    rect(s, 16, 52, 16, 12, CARBON_BLACK)
    for y in range(53, 63):
        px(s, 23, y, CYBER_YELLOW_D)

    return s

# =========================================================================
# 6. СПЕЦНАЗ (Tactical Operator в балаклаве, разгрузке и цифровом камуфляже)
# =========================================================================
def make_specops():
    s = new_skin()

    CAMO_DARK = (32, 38, 30, 255)
    CAMO_MID = (48, 58, 44, 255)
    CAMO_LIGHT = (70, 82, 62, 255)
    CAMO_EARTH = (44, 38, 30, 255)

    VEST_BLACK = (22, 24, 24, 255)
    VEST_SHADE = (14, 15, 15, 255)
    POUCH = (38, 42, 36, 255)

    GOGGLE_LENS = (35, 175, 215, 255) # Тактические очки
    GOGGLE_FRAME = (18, 18, 18, 255)

    BOOTS = (18, 18, 18, 255)
    SKIN = (215, 175, 140, 255)

    # Head: Балаклава
    rect(s, 8, 0, 8, 8, CAMO_DARK)
    rect(s, 16, 0, 8, 8, CAMO_DARK)
    rect(s, 0, 8, 8, 8, CAMO_DARK)
    rect(s, 16, 8, 8, 8, CAMO_DARK)
    rect(s, 24, 8, 8, 8, CAMO_DARK)

    # Face: прорезь для глаз в балаклаве
    rect(s, 8, 8, 8, 8, CAMO_DARK)
    rect(s, 9, 11, 6, 2, SKIN)
    px(s, 10, 11, (30, 30, 35, 255)); px(s, 13, 11, (30, 30, 35, 255)) # Глаза

    # Hat 2nd layer: Тактические очки с синими линзами и шлем
    rect(s, 40, 0, 8, 8, CAMO_MID)
    rect(s, 40, 8, 8, 3, CAMO_MID) # Каска
    # Goggles
    rect(s, 41, 10, 3, 2, GOGGLE_LENS)
    rect(s, 44, 10, 3, 2, GOGGLE_LENS)
    rect(s, 32, 10, 8, 1, GOGGLE_FRAME) # Ремешок очков
    rect(s, 48, 10, 8, 1, GOGGLE_FRAME)
    rect(s, 56, 10, 8, 1, GOGGLE_FRAME)

    # Torso: Бронежилет / разгрузка с подсумками и рацией
    rect(s, 20, 20, 8, 12, VEST_BLACK)
    rect(s, 32, 20, 8, 12, VEST_SHADE)
    rect(s, 16, 20, 4, 12, CAMO_MID)
    rect(s, 28, 20, 4, 12, CAMO_MID)

    # Карманы и магазины на разгрузке
    rect(s, 21, 24, 2, 4, POUCH)
    rect(s, 25, 24, 2, 4, POUCH)
    px(s, 21, 21, (80, 80, 80, 255)) # Рация
    px(s, 21, 20, (150, 150, 150, 255)) # Антенна

    # Torso 2nd Layer (объемный бронежилет)
    rect(s, 20, 36, 8, 12, VEST_SHADE)
    px(s, 23, 40, (220, 220, 230, 255)); px(s, 24, 40, (220, 220, 230, 255)) # Шеврон

    # Arms (Камуфляж + перчатки)
    rect(s, 40, 20, 16, 9, CAMO_MID)
    rect(s, 40, 29, 16, 3, VEST_BLACK) # Тактические перчатки

    rect(s, 32, 52, 16, 9, CAMO_MID)
    rect(s, 32, 61, 16, 3, VEST_BLACK)

    # Legs (Камуфляжные штаны + берцы)
    rect(s, 0, 20, 16, 9, CAMO_MID)
    rect(s, 0, 29, 16, 3, BOOTS) # Берцы

    rect(s, 16, 52, 16, 9, CAMO_MID)
    rect(s, 16, 61, 16, 3, BOOTS)

    return s

def main():
    dest_dir = 'phantom-launcher/assets/skins'
    os.makedirs(dest_dir, exist_ok=True)

    skins = {
        'zek_batya.png': make_zek_batya(),
        'steve.png': make_steve(),
        'alex.png': make_alex(),
        'neon.png': make_streetwear(),
        'cyber_phantom.png': make_cyber_phantom(),
        'specops.png': make_specops(),
    }

    for name, img in skins.items():
        path = os.path.join(dest_dir, name)
        img.save(path)
        print(f'Created {path} ({img.size})')

    # Also sync into APPDATA skins_library so existing installs immediately get the new skins
    appdata_lib = os.path.expandvars(r'%APPDATA%\phantom\skins_library')
    if os.path.exists(appdata_lib):
        name_map = {
            'zek_batya.png': 'Зек Батя.png',
            'steve.png': 'Стив.png',
            'alex.png': 'Алекс.png',
            'neon.png': 'Ветровка Стрит.png',
            'cyber_phantom.png': 'Кибер-фантом.png',
            'specops.png': 'Спецназ.png',
        }
        for src_name, dest_name in name_map.items():
            dp = os.path.join(appdata_lib, dest_name)
            skins[src_name].save(dp)
            print(f'Synced to {dp}')

if __name__ == '__main__':
    main()
