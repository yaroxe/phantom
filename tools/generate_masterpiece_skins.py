import os
from PIL import Image

def create_skin():
    return Image.new('RGBA', (64, 64), (0, 0, 0, 0))

def put_px(img, x, y, r, g, b, a=255):
    if 0 <= x < 64 and 0 <= y < 64:
        img.putpixel((x, y), (r, g, b, a))

def fill_rect(img, x0, y0, w, h, color):
    for y in range(y0, y0 + h):
        for x in range(x0, x0 + w):
            put_px(img, x, y, *color)

def draw_head_base(skin, skin_col, skin_shade, hair_col, hair_light, hair_shade):
    # Top of head
    fill_rect(skin, 8, 0, 8, 8, hair_col)
    # Bottom of head (neck/under chin)
    fill_rect(skin, 16, 0, 8, 8, skin_shade)

    # Right side of head (0..8, 8..16): Hair top, skin bottom
    fill_rect(skin, 0, 8, 8, 4, hair_col)
    fill_rect(skin, 0, 12, 8, 4, skin_col)
    for y in range(12, 16):
        put_px(skin, 0, y, *hair_col)
        put_px(skin, 1, y, *hair_shade)

    # Left side of head (16..24, 8..16)
    fill_rect(skin, 16, 8, 8, 4, hair_col)
    fill_rect(skin, 16, 12, 8, 4, skin_col)
    for y in range(12, 16):
        put_px(skin, 22, y, *hair_shade)
        put_px(skin, 23, y, *hair_col)

    # Back of head (24..32, 8..16): All hair
    fill_rect(skin, 24, 8, 8, 8, hair_col)
    for y in range(8, 16):
        put_px(skin, 27, y, *hair_light)
        put_px(skin, 28, y, *hair_shade)

    # Front of head (8..16, 8..16): Face
    fill_rect(skin, 8, 8, 8, 8, skin_col)
    # Hairline on forehead
    for x in range(8, 16):
        put_px(skin, x, 8, *hair_col)
    put_px(skin, 8, 9, *hair_col); put_px(skin, 15, 9, *hair_col)

# =========================================================================
# 1. ЗЕК БАТЯ (Типичный Батя в тельняшке, кепке, с тату 666, золотыми зубами и часами)
# =========================================================================
def make_zek_batya():
    skin = create_skin()

    SKIN_BASE = (222, 172, 134)
    SKIN_LIGHT = (235, 185, 148)
    SKIN_SHADE = (196, 146, 110)
    SKIN_DARK = (170, 122, 88)
    
    HAIR_DARK = (36, 24, 16)
    HAIR_MID = (55, 38, 26)
    STUBBLE = (145, 115, 92)
    
    EYE_WHITE = (240, 240, 245)
    EYE_PUPIL = (45, 55, 68)
    
    GOLD_TOOTH = (255, 215, 0)
    GOLD_SHINE = (255, 240, 140)
    WHITE_TOOTH = (245, 245, 240)
    LIP = (145, 80, 75)
    
    TATTOO_BLUE = (38, 62, 85)
    
    BERET_BASE = (24, 26, 32)
    BERET_SHADE = (16, 17, 22)
    BERET_LIGHT = (40, 44, 54)
    
    STRIPE_W = (235, 238, 245)
    STRIPE_W_SHADE = (205, 210, 220)
    STRIPE_B = (28, 55, 125)
    STRIPE_B_SHADE = (18, 38, 90)
    
    PANTS_BASE = (22, 26, 35)
    PANTS_SHADE = (14, 16, 24)
    PANTS_STRIPE = (230, 230, 235)
    
    WATCH_GOLD = (240, 195, 30)
    WATCH_DIAL = (255, 255, 255)
    WATCH_DARK = (180, 140, 20)
    
    BOOTS_BASE = (18, 16, 15)
    BOOTS_LIGHT = (32, 28, 26)

    # ------------------ ГОЛОВА (Base) ------------------
    fill_rect(skin, 0, 8, 32, 8, SKIN_BASE)
    fill_rect(skin, 8, 0, 8, 8, HAIR_DARK)
    fill_rect(skin, 16, 0, 8, 8, SKIN_SHADE)

    # Волосы по бокам и затылку
    for y in range(8, 16):
        put_px(skin, 0, y, *HAIR_DARK); put_px(skin, 1, y, *HAIR_MID)
        put_px(skin, 22, y, *HAIR_MID); put_px(skin, 23, y, *HAIR_DARK)
        for x in range(24, 32):
            put_px(skin, x, y, *HAIR_DARK if y < 13 else HAIR_MID)
    fill_rect(skin, 0, 8, 8, 3, HAIR_DARK)
    fill_rect(skin, 16, 8, 8, 3, HAIR_DARK)

    # Лицо: (8..15, 8..15)
    for x in range(8, 16):
        put_px(skin, x, 8, *HAIR_DARK)
    put_px(skin, 8, 9, *HAIR_DARK); put_px(skin, 15, 9, *HAIR_DARK)
    
    # ТАТУ "666" на лбу
    put_px(skin, 10, 9, *TATTOO_BLUE); put_px(skin, 12, 9, *TATTOO_BLUE); put_px(skin, 14, 9, *TATTOO_BLUE)
    put_px(skin, 10, 10, *TATTOO_BLUE); put_px(skin, 12, 10, *TATTOO_BLUE); put_px(skin, 14, 10, *TATTOO_BLUE)

    # Брови и глаза
    put_px(skin, 8, 10, *HAIR_DARK); put_px(skin, 9, 10, *HAIR_MID); put_px(skin, 15, 10, *HAIR_DARK)
    put_px(skin, 9, 11, *EYE_WHITE); put_px(skin, 10, 11, *EYE_PUPIL)
    put_px(skin, 13, 11, *EYE_PUPIL); put_px(skin, 14, 11, *EYE_WHITE)
    put_px(skin, 9, 12, *SKIN_SHADE); put_px(skin, 14, 12, *SKIN_SHADE)
    put_px(skin, 11, 12, *SKIN_LIGHT); put_px(skin, 12, 12, *SKIN_SHADE)

    # Усы
    for x in range(9, 15):
        put_px(skin, x, 13, *HAIR_DARK)
    put_px(skin, 8, 14, *HAIR_MID); put_px(skin, 15, 14, *HAIR_MID)

    # Рот с золотой фиксой
    put_px(skin, 10, 14, *WHITE_TOOTH)
    put_px(skin, 11, 14, *GOLD_SHINE) # ЗОЛОТОЙ ЗУБ!
    put_px(skin, 12, 14, *GOLD_TOOTH)
    put_px(skin, 13, 14, *LIP)

    # Щетина
    for x in range(9, 15):
        put_px(skin, x, 15, *STUBBLE)

    # ------------------ БЕРЕТ БАТИ (Hat Layer 32..64, 0..16) ------------------
    fill_rect(skin, 40, 0, 8, 8, BERET_BASE)
    put_px(skin, 43, 3, *BERET_LIGHT); put_px(skin, 44, 3, *BERET_LIGHT)
    for x in range(40, 48):
        put_px(skin, x, 8, *BERET_BASE)
        put_px(skin, x, 9, *BERET_SHADE)
    put_px(skin, 41, 10, *BERET_LIGHT); put_px(skin, 42, 10, *BERET_BASE); put_px(skin, 43, 10, *BERET_BASE)
    fill_rect(skin, 32, 8, 8, 3, BERET_BASE)
    fill_rect(skin, 48, 8, 8, 3, BERET_BASE)
    fill_rect(skin, 56, 8, 8, 4, BERET_BASE)

    # ------------------ ТОРС (Тельняшка) ------------------
    fill_rect(skin, 20, 16, 8, 4, STRIPE_W)
    put_px(skin, 22, 18, *SKIN_BASE); put_px(skin, 23, 18, *SKIN_BASE)
    put_px(skin, 24, 18, *SKIN_BASE); put_px(skin, 25, 18, *SKIN_BASE)

    for y in range(20, 32):
        stripe_color = STRIPE_B if (y // 1) % 2 == 0 else STRIPE_W
        shade_color = STRIPE_B_SHADE if (y // 1) % 2 == 0 else STRIPE_W_SHADE
        
        for x in range(20, 28):
            if y == 20 and (22 <= x <= 25):
                put_px(skin, x, y, *SKIN_BASE)
            elif y == 21 and (23 <= x <= 24):
                put_px(skin, x, y, *SKIN_BASE)
                put_px(skin, 23, 21, *GOLD_TOOTH) # золотой крестик
            else:
                put_px(skin, x, y, *stripe_color)
        
        for x in range(16, 20):
            put_px(skin, x, y, *shade_color)
        for x in range(28, 32):
            put_px(skin, x, y, *shade_color)
        for x in range(32, 40):
            put_px(skin, x, y, *stripe_color)

    # Ремень
    for x in range(16, 40):
        put_px(skin, x, 31, *(15, 15, 18))
    put_px(skin, 23, 31, *GOLD_TOOTH); put_px(skin, 24, 31, *GOLD_TOOTH)

    # ------------------ РУКИ (Мускулы, татуировки, часы) ------------------
    # Правая рука
    fill_rect(skin, 40, 20, 16, 12, SKIN_BASE)
    fill_rect(skin, 44, 16, 8, 4, SKIN_BASE)
    for x in range(44, 48):
        put_px(skin, x, 20, *STRIPE_B) # лямка
    # Наколки
    put_px(skin, 45, 23, *TATTOO_BLUE); put_px(skin, 46, 24, *TATTOO_BLUE); put_px(skin, 45, 25, *TATTOO_BLUE)
    put_px(skin, 44, 28, *TATTOO_BLUE)

    # Левая рука
    fill_rect(skin, 32, 52, 16, 12, SKIN_BASE)
    fill_rect(skin, 36, 48, 8, 4, SKIN_BASE)
    for x in range(36, 40):
        put_px(skin, x, 52, *STRIPE_B)
    # Золотые часы
    for x in range(32, 48):
        put_px(skin, x, 59, *WATCH_GOLD)
        put_px(skin, x, 60, *WATCH_DARK)
    put_px(skin, 45, 59, *WATCH_DIAL); put_px(skin, 46, 59, *WATCH_GOLD)
    # Наколка
    put_px(skin, 37, 55, *TATTOO_BLUE); put_px(skin, 38, 56, *TATTOO_BLUE)

    # ------------------ НОГИ ------------------
    fill_rect(skin, 0, 20, 16, 12, PANTS_BASE)
    fill_rect(skin, 4, 16, 8, 4, PANTS_BASE)
    for y in range(20, 29):
        put_px(skin, 1, y, *PANTS_STRIPE)
        put_px(skin, 2, y, *PANTS_SHADE)
    fill_rect(skin, 0, 29, 16, 3, BOOTS_BASE)
    fill_rect(skin, 8, 16, 4, 4, BOOTS_BASE)
    for x in range(4, 8):
        put_px(skin, x, 29, *BOOTS_LIGHT)

    fill_rect(skin, 16, 52, 16, 12, PANTS_BASE)
    fill_rect(skin, 20, 48, 8, 4, PANTS_BASE)
    for y in range(52, 61):
        put_px(skin, 17, y, *PANTS_STRIPE)
        put_px(skin, 18, y, *PANTS_SHADE)
    fill_rect(skin, 16, 61, 16, 3, BOOTS_BASE)
    fill_rect(skin, 24, 48, 4, 4, BOOTS_BASE)
    for x in range(20, 24):
        put_px(skin, x, 61, *BOOTS_LIGHT)

    return skin

# =========================================================================
# 2. ВЕТРОВКА СТРИТ
# =========================================================================
def make_neon_street():
    skin = create_skin()

    SKIN = (235, 190, 155)
    SKIN_SHADE = (205, 160, 125)
    HAIR = (50, 36, 25)
    HAIR_LIGHT = (75, 55, 38)
    HAIR_SHADE = (38, 26, 18)
    
    WIND_WHITE = (245, 248, 255)
    WIND_TEAL = (0, 168, 180)
    WIND_NAVY = (24, 34, 58)
    WIND_ZIP = (210, 215, 225)
    
    JEANS_BASE = (45, 75, 125)
    JEANS_LIGHT = (65, 95, 155)
    
    SHOE_WHITE = (245, 245, 250)
    SHOE_TEAL = (0, 180, 195)
    HEADPHONE_RED = (220, 45, 45)

    # Голова с полным объемом волос
    draw_head_base(skin, SKIN, SKIN_SHADE, HAIR, HAIR_LIGHT, HAIR_SHADE)
    # Челка
    put_px(skin, 9, 9, *HAIR_LIGHT); put_px(skin, 10, 9, *HAIR_LIGHT); put_px(skin, 11, 9, *HAIR)
    # Глаза
    put_px(skin, 9, 11, *(240, 240, 245)); put_px(skin, 10, 11, *(35, 60, 95))
    put_px(skin, 14, 11, *(240, 240, 245)); put_px(skin, 13, 11, *(35, 60, 95))
    put_px(skin, 11, 13, *(190, 110, 100)); put_px(skin, 12, 13, *(190, 110, 100))

    # Объем волос на Hat layer
    for x in range(40, 48):
        put_px(skin, x, 7, *HAIR_LIGHT)
        put_px(skin, x, 8, *HAIR)
    # Наушники на шее
    put_px(skin, 40, 14, *HEADPHONE_RED); put_px(skin, 40, 15, *HEADPHONE_RED)
    put_px(skin, 47, 14, *HEADPHONE_RED); put_px(skin, 47, 15, *HEADPHONE_RED)

    # Торс
    fill_rect(skin, 20, 16, 8, 4, WIND_WHITE)
    for y in range(20, 32):
        for x in range(20, 28):
            col = WIND_WHITE if y < 24 else (WIND_TEAL if y < 27 else WIND_NAVY)
            put_px(skin, x, y, *col)
        put_px(skin, 23, y, *WIND_ZIP); put_px(skin, 24, y, *WIND_ZIP)
    fill_rect(skin, 16, 20, 4, 12, WIND_NAVY)
    fill_rect(skin, 28, 20, 4, 12, WIND_NAVY)
    fill_rect(skin, 32, 20, 8, 4, WIND_WHITE)
    fill_rect(skin, 32, 24, 8, 3, WIND_TEAL)
    fill_rect(skin, 32, 27, 8, 5, WIND_NAVY)

    # Руки
    fill_rect(skin, 40, 20, 16, 8, WIND_WHITE)
    fill_rect(skin, 40, 28, 16, 2, WIND_TEAL)
    fill_rect(skin, 40, 30, 16, 2, SKIN)
    fill_rect(skin, 44, 16, 8, 4, WIND_WHITE)
    put_px(skin, 45, 29, *(220, 220, 230)); put_px(skin, 46, 29, *(150, 150, 160)) # часы

    fill_rect(skin, 32, 52, 16, 8, WIND_WHITE)
    fill_rect(skin, 32, 60, 16, 2, WIND_TEAL)
    fill_rect(skin, 32, 62, 16, 2, SKIN)
    fill_rect(skin, 36, 48, 8, 4, WIND_WHITE)

    # Ноги
    fill_rect(skin, 0, 20, 16, 9, JEANS_BASE)
    put_px(skin, 5, 24, *JEANS_LIGHT); put_px(skin, 6, 24, *JEANS_LIGHT)
    fill_rect(skin, 0, 29, 16, 3, SHOE_WHITE)
    put_px(skin, 5, 29, *SHOE_TEAL); put_px(skin, 6, 29, *SHOE_TEAL)

    fill_rect(skin, 16, 52, 16, 9, JEANS_BASE)
    put_px(skin, 21, 56, *JEANS_LIGHT); put_px(skin, 22, 56, *JEANS_LIGHT)
    fill_rect(skin, 16, 61, 16, 3, SHOE_WHITE)
    put_px(skin, 21, 61, *SHOE_TEAL); put_px(skin, 22, 61, *SHOE_TEAL)

    return skin

# =========================================================================
# 3. КИБЕР-ФАНТОМ
# =========================================================================
def make_cyber_phantom():
    skin = create_skin()

    CARBON_DARK = (18, 18, 22)
    CARBON_MID = (32, 34, 40)
    CARBON_LIGHT = (50, 54, 64)
    
    NEON_YELLOW = (255, 230, 0)
    CYAN_OPTIC = (0, 240, 255)
    CYAN_GLOW = (160, 250, 255)
    SKIN = (210, 170, 140)

    # Голова
    fill_rect(skin, 0, 8, 32, 8, CARBON_MID)
    fill_rect(skin, 8, 0, 8, 8, CARBON_DARK)
    fill_rect(skin, 16, 0, 8, 8, CARBON_DARK)
    # Нижняя половина лица видна
    fill_rect(skin, 9, 12, 6, 4, SKIN)
    put_px(skin, 11, 14, *(160, 90, 80))

    # КИБЕР-ВИЗОР (на лице)
    for x in range(9, 15):
        put_px(skin, x, 10, *CYAN_OPTIC)
        put_px(skin, x, 11, *CYAN_GLOW)
    put_px(skin, 14, 10, *NEON_YELLOW); put_px(skin, 14, 11, *NEON_YELLOW)

    # Hat layer: Капюшон (ТОЛЬКО верх и бока, НЕ закрывая лицо!)
    fill_rect(skin, 40, 0, 8, 8, CARBON_MID)
    # Верх лба
    for x in range(40, 48):
        put_px(skin, x, 8, *NEON_YELLOW)
    # Бока капюшона
    fill_rect(skin, 32, 8, 8, 8, CARBON_DARK)
    fill_rect(skin, 48, 8, 8, 8, CARBON_DARK)
    fill_rect(skin, 56, 8, 8, 8, CARBON_MID)

    # Торс
    fill_rect(skin, 16, 20, 24, 12, CARBON_MID)
    fill_rect(skin, 20, 16, 8, 4, CARBON_DARK)
    for y in range(21, 30):
        put_px(skin, 21, y, *NEON_YELLOW)
        put_px(skin, 26, y, *CARBON_LIGHT)
    put_px(skin, 22, 25, *NEON_YELLOW); put_px(skin, 23, 25, *NEON_YELLOW)
    put_px(skin, 24, 25, *NEON_YELLOW); put_px(skin, 25, 25, *NEON_YELLOW)

    # Руки
    fill_rect(skin, 40, 20, 16, 12, CARBON_MID)
    fill_rect(skin, 44, 16, 8, 4, CARBON_DARK)
    for y in range(20, 24):
        put_px(skin, 45, y, *NEON_YELLOW)

    fill_rect(skin, 32, 52, 16, 12, CARBON_MID)
    fill_rect(skin, 36, 48, 8, 4, CARBON_DARK)
    put_px(skin, 37, 58, *CYAN_OPTIC); put_px(skin, 38, 58, *CYAN_OPTIC)
    put_px(skin, 37, 59, *CYAN_GLOW); put_px(skin, 38, 59, *CYAN_GLOW)

    # Ноги
    fill_rect(skin, 0, 20, 16, 12, CARBON_DARK)
    fill_rect(skin, 16, 52, 16, 12, CARBON_DARK)
    for x in range(4, 8):
        put_px(skin, x, 31, *NEON_YELLOW)
    for x in range(20, 24):
        put_px(skin, x, 63, *NEON_YELLOW)

    return skin

# =========================================================================
# 4. СПЕЦНАЗ
# =========================================================================
def make_specops():
    skin = create_skin()

    CAMO_DARK = (28, 30, 32)
    CAMO_MID = (48, 52, 54)
    VEST_BLACK = (16, 17, 18)
    VEST_POUCH = (38, 40, 42)
    NVG_GREEN = (40, 235, 90)
    NVG_LENS = (20, 120, 45)
    SKIN = (200, 160, 130)

    # Балаклава
    fill_rect(skin, 0, 8, 32, 8, CAMO_DARK)
    fill_rect(skin, 8, 0, 8, 8, CAMO_MID)
    fill_rect(skin, 16, 0, 8, 8, CAMO_DARK)
    # Прорезь для глаз
    fill_rect(skin, 9, 11, 6, 2, SKIN)
    put_px(skin, 10, 11, *(20, 20, 25)); put_px(skin, 13, 11, *(20, 20, 25))

    # Hat layer: Шлем и ПНВ
    fill_rect(skin, 40, 0, 8, 8, CAMO_MID)
    fill_rect(skin, 32, 8, 8, 4, CAMO_DARK)
    fill_rect(skin, 48, 8, 8, 4, CAMO_DARK)
    fill_rect(skin, 56, 8, 8, 5, CAMO_DARK)
    # ПНВ на лбу (НЕ закрывая прорезь для глаз!)
    put_px(skin, 41, 9, *NVG_GREEN); put_px(skin, 42, 9, *NVG_LENS)
    put_px(skin, 45, 9, *NVG_GREEN); put_px(skin, 46, 9, *NVG_LENS)

    # Торс
    fill_rect(skin, 16, 20, 24, 12, CAMO_MID)
    fill_rect(skin, 20, 20, 8, 11, VEST_BLACK)
    fill_rect(skin, 21, 25, 2, 4, VEST_POUCH)
    fill_rect(skin, 23, 25, 2, 4, VEST_POUCH)
    fill_rect(skin, 25, 25, 2, 4, VEST_POUCH)
    put_px(skin, 22, 22, *(220, 40, 40)); put_px(skin, 23, 22, *(240, 240, 240))

    # Руки
    fill_rect(skin, 40, 20, 16, 8, CAMO_MID)
    fill_rect(skin, 40, 28, 16, 4, VEST_BLACK)
    fill_rect(skin, 32, 52, 16, 8, CAMO_MID)
    fill_rect(skin, 32, 60, 16, 4, VEST_BLACK)

    # Ноги
    fill_rect(skin, 0, 20, 16, 12, CAMO_MID)
    fill_rect(skin, 16, 52, 16, 12, CAMO_MID)
    fill_rect(skin, 5, 24, 2, 3, VEST_BLACK)
    fill_rect(skin, 21, 56, 2, 3, VEST_BLACK)
    fill_rect(skin, 0, 29, 16, 3, VEST_BLACK)
    fill_rect(skin, 16, 61, 16, 3, VEST_BLACK)

    return skin

# =========================================================================
# 5. СТИВ HD
# =========================================================================
def make_steve_hd():
    skin = create_skin()

    SKIN = (205, 155, 120)
    SKIN_SHADE = (180, 130, 98)
    SKIN_LIGHT = (222, 172, 138)
    
    HAIR = (45, 30, 20)
    HAIR_LIGHT = (68, 48, 32)
    HAIR_SHADE = (34, 22, 15)
    
    EYE_WHITE = (245, 245, 250)
    EYE_BLUE = (45, 55, 130)
    MOUTH = (125, 65, 55)
    BEARD = (85, 55, 35)

    TEAL_BASE = (0, 140, 150)
    TEAL_LIGHT = (20, 165, 175)
    TEAL_SHADE = (0, 105, 115)

    JEANS_BASE = (42, 52, 118)
    JEANS_LIGHT = (62, 75, 145)
    JEANS_SHADE = (28, 36, 88)

    SHOES = (90, 88, 92)

    # Голова
    draw_head_base(skin, SKIN, SKIN_SHADE, HAIR, HAIR_LIGHT, HAIR_SHADE)
    # Лицо
    put_px(skin, 9, 11, *EYE_WHITE); put_px(skin, 10, 11, *EYE_BLUE)
    put_px(skin, 14, 11, *EYE_WHITE); put_px(skin, 13, 11, *EYE_BLUE)
    put_px(skin, 11, 12, *SKIN_LIGHT); put_px(skin, 12, 12, *SKIN_SHADE)
    put_px(skin, 11, 13, *MOUTH); put_px(skin, 12, 13, *MOUTH)
    for x in range(10, 14):
        put_px(skin, x, 14, *BEARD)
    put_px(skin, 9, 13, *BEARD); put_px(skin, 14, 13, *BEARD)

    # Торс
    fill_rect(skin, 16, 20, 24, 12, TEAL_BASE)
    fill_rect(skin, 20, 16, 8, 4, TEAL_LIGHT)
    fill_rect(skin, 22, 20, 4, 2, SKIN)
    put_px(skin, 23, 22, *SKIN_SHADE); put_px(skin, 24, 22, *SKIN_SHADE)
    put_px(skin, 21, 26, *TEAL_SHADE); put_px(skin, 26, 27, *TEAL_SHADE)

    # Руки
    fill_rect(skin, 40, 20, 16, 4, TEAL_BASE)
    fill_rect(skin, 40, 24, 16, 8, SKIN)
    fill_rect(skin, 44, 16, 8, 4, TEAL_LIGHT)

    fill_rect(skin, 32, 52, 16, 4, TEAL_BASE)
    fill_rect(skin, 32, 56, 16, 8, SKIN)
    fill_rect(skin, 36, 48, 8, 4, TEAL_LIGHT)

    # Ноги
    fill_rect(skin, 0, 20, 16, 10, JEANS_BASE)
    fill_rect(skin, 0, 30, 16, 2, SHOES)
    fill_rect(skin, 16, 52, 16, 10, JEANS_BASE)
    fill_rect(skin, 16, 62, 16, 2, SHOES)

    return skin

# =========================================================================
# 6. АЛЕКС HD
# =========================================================================
def make_alex_hd():
    skin = create_skin()

    SKIN = (235, 195, 165)
    SKIN_SHADE = (210, 170, 140)
    
    GINGER = (195, 95, 35)
    GINGER_LIGHT = (225, 120, 50)
    GINGER_SHADE = (160, 75, 25)
    
    EYE_WHITE = (245, 245, 250)
    EYE_GREEN = (35, 125, 65)

    TUNIC_BASE = (95, 125, 65)
    TUNIC_LIGHT = (115, 148, 80)
    BELT = (60, 42, 28)
    BUCKLE = (220, 180, 45)

    PANTS = (100, 70, 50)
    BOOTS = (55, 40, 30)

    # Голова
    draw_head_base(skin, SKIN, SKIN_SHADE, GINGER, GINGER_LIGHT, GINGER_SHADE)
    put_px(skin, 8, 9, *GINGER_LIGHT); put_px(skin, 9, 9, *GINGER)
    put_px(skin, 9, 11, *EYE_WHITE); put_px(skin, 10, 11, *EYE_GREEN)
    put_px(skin, 14, 11, *EYE_WHITE); put_px(skin, 13, 11, *EYE_GREEN)
    put_px(skin, 11, 13, *(185, 110, 95)); put_px(skin, 12, 13, *(185, 110, 95))

    # Рыжая коса на спине
    for y in range(8, 16):
        put_px(skin, 27, y, *GINGER_LIGHT)
        put_px(skin, 28, y, *GINGER)

    # Торс
    fill_rect(skin, 16, 20, 24, 12, TUNIC_BASE)
    fill_rect(skin, 20, 16, 8, 4, TUNIC_LIGHT)
    for x in range(16, 40):
        put_px(skin, x, 29, *BELT)
    put_px(skin, 23, 29, *BUCKLE); put_px(skin, 24, 29, *BUCKLE)

    # Руки
    fill_rect(skin, 40, 20, 16, 4, TUNIC_BASE)
    fill_rect(skin, 40, 24, 16, 8, SKIN)
    fill_rect(skin, 44, 16, 8, 4, TUNIC_LIGHT)

    fill_rect(skin, 32, 52, 16, 4, TUNIC_BASE)
    fill_rect(skin, 32, 56, 16, 8, SKIN)
    fill_rect(skin, 36, 48, 8, 4, TUNIC_LIGHT)

    # Ноги
    fill_rect(skin, 0, 20, 16, 8, PANTS)
    fill_rect(skin, 0, 28, 16, 4, BOOTS)
    fill_rect(skin, 16, 52, 16, 8, PANTS)
    fill_rect(skin, 16, 60, 16, 4, BOOTS)

    return skin

def main():
    skins = {
        "zek_batya": make_zek_batya(),
        "neon": make_neon_street(),
        "cyber_phantom": make_cyber_phantom(),
        "specops": make_specops(),
        "steve": make_steve_hd(),
        "alex": make_alex_hd(),
    }

    out_dir = "phantom-launcher/assets/skins"
    os.makedirs(out_dir, exist_ok=True)

    for name, img in skins.items():
        p = os.path.join(out_dir, f"{name}.png")
        img.save(p)
        print(f"[+] Saved {p}")

if __name__ == "__main__":
    main()
