import math
from PIL import Image

def rot_y(p, rad):
    x, y, z = p
    return (x * math.cos(rad) + z * math.sin(rad), y, -x * math.sin(rad) + z * math.cos(rad))

def rot_x(p, rad):
    x, y, z = p
    return (x, y * math.cos(rad) - z * math.sin(rad), y * math.sin(rad) + z * math.cos(rad))

def rot_z(p, rad):
    x, y, z = p
    return (x * math.cos(rad) - y * math.sin(rad), x * math.sin(rad) + y * math.cos(rad), z)

def rotate_limb(p, pivot, pitch_deg, roll_deg, yaw_deg):
    vx = p[0] - pivot[0]
    vy = p[1] - pivot[1]
    vz = p[2] - pivot[2]
    # pitch
    p1 = rot_x((vx, vy, vz), math.radians(pitch_deg))
    # roll
    p2 = rot_z(p1, math.radians(roll_deg))
    # yaw
    p3 = rot_y(p2, math.radians(yaw_deg))
    return (pivot[0] + p3[0], pivot[1] + p3[1], pivot[2] + p3[2])

def interp(a, b, t):
    return (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t)

def blend(dst, src):
    sa = src[3] / 255.0
    if sa <= 0: return dst
    if sa >= 1: return src
    da = dst[3] / 255.0
    oa = sa + da * (1.0 - sa)
    if oa <= 0: return dst
    r = int((src[0] * sa + dst[0] * da * (1.0 - sa)) / oa)
    g = int((src[1] * sa + dst[1] * da * (1.0 - sa)) / oa)
    b = int((src[2] * sa + dst[2] * da * (1.0 - sa)) / oa)
    return (r, g, b, int(oa * 255))

def render_3d(skin_path, out_path, is_slim=False, yaw_deg=22.0, pitch_deg=10.0, W=130, H=240):
    skin = Image.open(skin_path).convert('RGBA')
    sw, sh = skin.size
    is_64x64 = sh >= 64
    arm_w = 3 if is_slim else 4

    canvas = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    c_px = canvas.load()
    z_buf = [-999999.0] * (W * H)

    cam_yaw = math.radians(yaw_deg)
    cam_pitch = math.radians(pitch_deg)
    scale = (H * 0.72) / 32.0

    def rot_cam(p):
        p1 = rot_y(p, cam_yaw)
        p2 = rot_x(p1, cam_pitch)
        return p2

    def proj(p_rot):
        sx = W * 0.5 + p_rot[0] * scale
        sy = H * 0.88 - p_rot[1] * scale
        return (sx, sy, p_rot[2])

    def draw_tri(p0, p1, p2, color):
        # 2D edge check
        def edge(a, b, c):
            return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])

        area = edge(p0, p1, p2)
        if area <= 1e-4:
            return # Back-face or degenerate
        inv_area = 1.0 / area

        min_x = max(0, min(W - 1, int(math.floor(min(p0[0], p1[0], p2[0])))))
        max_x = max(0, min(W - 1, int(math.ceil(max(p0[0], p1[0], p2[0])))))
        min_y = max(0, min(H - 1, int(math.floor(min(p0[1], p1[1], p2[1])))))
        max_y = max(0, min(H - 1, int(math.ceil(max(p0[1], p1[1], p2[1])))))

        for y in range(min_y, max_y + 1):
            py = y + 0.5
            for x in range(min_x, max_x + 1):
                px = x + 0.5
                p = (px, py)
                w0 = edge(p1, p2, p) * inv_area
                w1 = edge(p2, p0, p) * inv_area
                w2 = edge(p0, p1, p) * inv_area

                if w0 >= -0.001 and w1 >= -0.001 and w2 >= -0.001:
                    z = w0 * p0[2] + w1 * p1[2] + w2 * p2[2]
                    idx = y * W + x
                    if z > z_buf[idx] - 0.001:
                        if color[3] >= 240:
                            c_px[x, y] = color
                            z_buf[idx] = z
                        elif color[3] > 10:
                            c_px[x, y] = blend(c_px[x, y], color)
                            if color[3] >= 180:
                                z_buf[idx] = z

    def add_box(bx, by, bz, w, h, d, uv_map, pivot, l_pitch, l_roll, l_yaw, inflate=0.0):
        x0 = bx - inflate; x1 = bx + w + inflate
        y0 = by - inflate; y1 = by + h + inflate
        z0 = bz - inflate; z1 = bz + d + inflate

        faces = [
            ("top",    [(x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)], 1.15),
            ("bottom", [(x0, y0, z1), (x1, y0, z1), (x1, y0, z0), (x0, y0, z0)], 0.55),
            ("right",  [(x0, y1, z0), (x0, y1, z1), (x0, y0, z1), (x0, y0, z0)], 0.82),
            ("left",   [(x1, y1, z1), (x1, y1, z0), (x1, y0, z0), (x1, y0, z1)], 0.75),
            ("front",  [(x0, y1, z1), (x1, y1, z1), (x1, y0, z1), (x0, y0, z1)], 1.00),
            ("back",   [(x1, y1, z0), (x0, y1, z0), (x0, y0, z0), (x1, y0, z0)], 0.88),
        ]

        for fname, corners, shade in faces:
            uv = None
            for name, mapping in uv_map:
                if name == fname:
                    uv = mapping
                    break
            if not uv: continue
            ux, uy, uw, uh = uv
            p00, p10, p11, p01 = corners

            for v in range(uh):
                for u in range(uw):
                    sx = ux + u
                    sy = uy + v
                    if sx >= sw or sy >= sh: continue
                    px = skin.getpixel((sx, sy))
                    if px[3] < 12: continue

                    u0 = u / float(uw); u1 = (u + 1) / float(uw)
                    v0 = v / float(uh); v1 = (v + 1) / float(uh)

                    top_0 = interp(p00, p10, u0); top_1 = interp(p00, p10, u1)
                    bot_0 = interp(p01, p11, u0); bot_1 = interp(p01, p11, u1)

                    c0 = interp(top_0, bot_0, v0)
                    c1 = interp(top_1, bot_1, v0)
                    c2 = interp(top_1, bot_1, v1)
                    c3 = interp(top_0, bot_0, v1)

                    c0 = rotate_limb(c0, pivot, l_pitch, l_roll, l_yaw)
                    c1 = rotate_limb(c1, pivot, l_pitch, l_roll, l_yaw)
                    c2 = rotate_limb(c2, pivot, l_pitch, l_roll, l_yaw)
                    c3 = rotate_limb(c3, pivot, l_pitch, l_roll, l_yaw)

                    r0 = proj(rot_cam(c0))
                    r1 = proj(rot_cam(c1))
                    r2 = proj(rot_cam(c2))
                    r3 = proj(rot_cam(c3))

                    r = int(min(255, px[0] * shade))
                    g = int(min(255, px[1] * shade))
                    b = int(min(255, px[2] * shade))
                    col = (r, g, b, px[3])

                    draw_tri(r0, r1, r2, col)
                    draw_tri(r0, r2, r3, col)

    # Pose: Natural dynamic standing/walking pose
    # Head
    head_uv = [("top", (8, 0, 8, 8)), ("bottom", (16, 0, 8, 8)), ("right", (0, 8, 8, 8)),
               ("front", (8, 8, 8, 8)), ("left", (16, 8, 8, 8)), ("back", (24, 8, 8, 8))]
    hat_uv = [("top", (40, 0, 8, 8)), ("bottom", (48, 0, 8, 8)), ("right", (32, 8, 8, 8)),
              ("front", (40, 8, 8, 8)), ("left", (48, 8, 8, 8)), ("back", (56, 8, 8, 8))]
    add_box(-4.0, 24.0, -4.0, 8.0, 8.0, 8.0, head_uv, (0.0, 24.0, 0.0), -2.0, 0.0, 6.0, 0.0)
    add_box(-4.0, 24.0, -4.0, 8.0, 8.0, 8.0, hat_uv, (0.0, 24.0, 0.0), -2.0, 0.0, 6.0, 0.45)

    # Torso
    torso_uv = [("top", (20, 16, 8, 4)), ("bottom", (28, 16, 8, 4)), ("right", (16, 20, 4, 12)),
                ("front", (20, 20, 8, 12)), ("left", (28, 20, 4, 12)), ("back", (32, 20, 8, 12))]
    jacket_uv = [("top", (20, 32, 8, 4)), ("bottom", (28, 32, 8, 4)), ("right", (16, 36, 4, 12)),
                 ("front", (20, 36, 8, 12)), ("left", (28, 36, 4, 12)), ("back", (32, 36, 8, 12))]
    add_box(-4.0, 12.0, -2.0, 8.0, 12.0, 4.0, torso_uv, (0.0, 18.0, 0.0), 0.0, 0.0, 0.0, 0.0)
    if is_64x64:
        add_box(-4.0, 12.0, -2.0, 8.0, 12.0, 4.0, jacket_uv, (0.0, 18.0, 0.0), 0.0, 0.0, 0.0, 0.35)

    # Right Arm
    aw = float(arm_w)
    rarm_uv = [("top", (44, 16, arm_w, 4)), ("bottom", (44 + arm_w, 16, arm_w, 4)),
               ("right", (40, 20, 4, 12)), ("front", (44, 20, arm_w, 12)),
               ("left", (44 + arm_w, 20, 4, 12)), ("back", (48 + arm_w, 20, arm_w, 12))]
    rsleeve_uv = [("top", (44, 32, arm_w, 4)), ("bottom", (44 + arm_w, 32, arm_w, 4)),
                  ("right", (40, 36, 4, 12)), ("front", (44, 36, arm_w, 12)),
                  ("left", (44 + arm_w, 36, 4, 12)), ("back", (48 + arm_w, 36, arm_w, 12))]
    add_box(-4.0 - aw, 12.0, -2.0, aw, 12.0, 4.0, rarm_uv, (-4.0, 22.0, 0.0), -14.0, -5.0, 0.0, 0.0)
    if is_64x64:
        add_box(-4.0 - aw, 12.0, -2.0, aw, 12.0, 4.0, rsleeve_uv, (-4.0, 22.0, 0.0), -14.0, -5.0, 0.0, 0.30)

    # Left Arm
    if is_64x64:
        larm_uv = [("top", (36, 48, arm_w, 4)), ("bottom", (36 + arm_w, 48, arm_w, 4)),
                   ("right", (32, 52, 4, 12)), ("front", (36, 52, arm_w, 12)),
                   ("left", (36 + arm_w, 52, 4, 12)), ("back", (40 + arm_w, 52, arm_w, 12))]
        lsleeve_uv = [("top", (52, 48, arm_w, 4)), ("bottom", (52 + arm_w, 48, arm_w, 4)),
                      ("right", (48, 52, 4, 12)), ("front", (52, 52, arm_w, 12)),
                      ("left", (52 + arm_w, 52, 4, 12)), ("back", (56 + arm_w, 52, arm_w, 12))]
        add_box(4.0, 12.0, -2.0, aw, 12.0, 4.0, larm_uv, (4.0, 22.0, 0.0), 14.0, 5.0, 0.0, 0.0)
        add_box(4.0, 12.0, -2.0, aw, 12.0, 4.0, lsleeve_uv, (4.0, 22.0, 0.0), 14.0, 5.0, 0.0, 0.30)
    else:
        add_box(4.0, 12.0, -2.0, aw, 12.0, 4.0, rarm_uv, (4.0, 22.0, 0.0), 14.0, 5.0, 0.0, 0.0)

    # Right Leg
    rleg_uv = [("top", (4, 16, 4, 4)), ("bottom", (8, 16, 4, 4)), ("right", (0, 20, 4, 12)),
               ("front", (4, 20, 4, 12)), ("left", (8, 20, 4, 12)), ("back", (12, 20, 4, 12))]
    rpants_uv = [("top", (4, 32, 4, 4)), ("bottom", (8, 32, 4, 4)), ("right", (0, 36, 4, 12)),
                 ("front", (4, 36, 4, 12)), ("left", (8, 36, 4, 12)), ("back", (12, 36, 4, 12))]
    add_box(-4.0, 0.0, -2.0, 4.0, 12.0, 4.0, rleg_uv, (-2.0, 12.0, 0.0), 10.0, -1.0, 0.0, 0.0)
    if is_64x64:
        add_box(-4.0, 0.0, -2.0, 4.0, 12.0, 4.0, rpants_uv, (-2.0, 12.0, 0.0), 10.0, -1.0, 0.0, 0.30)

    # Left Leg
    if is_64x64:
        lleg_uv = [("top", (20, 48, 4, 4)), ("bottom", (24, 48, 4, 4)), ("right", (16, 52, 4, 12)),
                   ("front", (20, 52, 4, 12)), ("left", (24, 52, 4, 12)), ("back", (28, 52, 4, 12))]
        lpants_uv = [("top", (4, 48, 4, 4)), ("bottom", (8, 48, 4, 4)), ("right", (0, 52, 4, 12)),
                     ("front", (4, 52, 4, 12)), ("left", (8, 52, 4, 12)), ("back", (12, 52, 4, 12))]
        add_box(0.0, 0.0, -2.0, 4.0, 12.0, 4.0, lleg_uv, (2.0, 12.0, 0.0), -10.0, 1.0, 0.0, 0.0)
        add_box(0.0, 0.0, -2.0, 4.0, 12.0, 4.0, lpants_uv, (2.0, 12.0, 0.0), -10.0, 1.0, 0.0, 0.30)
    else:
        add_box(0.0, 0.0, -2.0, 4.0, 12.0, 4.0, rleg_uv, (2.0, 12.0, 0.0), -10.0, 1.0, 0.0, 0.0)

    canvas.save(out_path)
    print(f"[✓] Rendered {out_path} ({W}x{H})")

render_3d("phantom-launcher/assets/skins/zek_batya.png", "target/test_batya_front.png", False, 22.0, 10.0, 130, 240)
render_3d("phantom-launcher/assets/skins/zek_batya.png", "target/test_batya_back.png", False, -158.0, 10.0, 130, 240)
render_3d("phantom-launcher/assets/skins/neon.png", "target/test_neon_front.png", False, 22.0, 10.0, 130, 240)
render_3d("phantom-launcher/assets/skins/steve.png", "target/test_steve_front.png", False, 22.0, 10.0, 130, 240)
