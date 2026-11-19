use std::{
    env, fs,
    path::Path,
};

fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("phantom.ico");
        let _ = res.compile();
    }

    println!("cargo:rerun-if-changed=assets/skins");
    generate_skins_catalog();
}

fn get_skin_meta(filename: &str) -> (&'static str, bool) {
    match filename {
        // Priority (Page 1)
        "zek_batya.png" => ("Батя", false),
        "steve.png" => ("Стив", false),
        "alex.png" => ("Алекс", true),
        "entity_303.png" => ("Энтити 303", false),
        "herobrine.png" => ("Херобрин", false),
        "town_woman.png" => ("Горожанка", false),
        "town_castaway.png" => ("Селянин-дикарь", false),
        "city_king.png" => ("Король", false),
        "city_queen.png" => ("Королева", false),
        "skin_knight.png" => ("Рыцарь", false),
        "costume_creeper.png" => ("Костюм Крипера", false),
        "costume_enderman.png" => ("Костюм Эндермена", false),
        "costume_skeleton.png" => ("Костюм Скелета", false),
        "costume_zombie.png" => ("Костюм Зомби", false),
        "city_blacksmith.png" => ("Кузнец", false),
        "city_mage.png" => ("Маг", false),

        // CityFolk
        "city_baker.png" => ("Пекарь", false),
        "city_barmaid_slim.png" => ("Барменша", true),
        "city_barman.png" => ("Бармен", false),
        "city_baron.png" => ("Барон", false),
        "city_baroness.png" => ("Баронесса", false),
        "city_butcher.png" => ("Мясник", false),
        "city_carpenter.png" => ("Плотник", false),
        "city_chef.png" => ("Шеф-повар", false),
        "city_holyman.png" => ("Священник", false),
        "city_holywoman.png" => ("Монахиня", false),
        "city_jailer.png" => ("Тюремщик", false),
        "city_postman.png" => ("Почтальон", false),
        "city_shoemaker.png" => ("Сапожник", false),
        "city_victorian.png" => ("Викторианец", false),
        "city_watchman.png" => ("Стражник", false),
        "city_weaponsmith.png" => ("Оружейник", false),

        // TownFolk
        "town_bandit_slim.png" => ("Бандит", true),
        "town_bard.png" => ("Бард", false),
        "town_crier.png" => ("Глашатай", false),
        "town_farmer.png" => ("Фермер", false),
        "town_forester.png" => ("Лесник", false),
        "town_gardener.png" => ("Садовник", false),
        "town_mime.png" => ("Мим", false),
        "town_miner.png" => ("Шахтёр", false),
        "town_monk.png" => ("Монах", false),
        "town_old_lady.png" => ("Бабушка", false),
        "town_old_man.png" => ("Дедушка", false),
        "town_peasant.png" => ("Крестьянин", false),
        "town_rogue.png" => ("Разбойник", false),
        "town_shopkeeper_slim.png" => ("Торговка", true),
        "town_strongman.png" => ("Силач", false),
        "town_thief.png" => ("Вор", false),
        "town_vagrant.png" => ("Бродяга", false),
        "town_witch_slim.png" => ("Ведьма", true),

        // Costumes
        "costume_cow.png" => ("Костюм Коровы", false),
        "costume_ghast.png" => ("Костюм Гаста", false),
        "costume_golem.png" => ("Костюм Голема", false),
        "costume_mooshroom.png" => ("Мухомор", false),
        "costume_ocelot.png" => ("Костюм Оцелота", false),
        "costume_pig.png" => ("Костюм Свиньи", false),
        "costume_pigman.png" => ("Костюм Свинозомби", false),
        "costume_pink_sheep.png" => ("Розовая Овца", false),
        "costume_rainbow_sheep.png" => ("Радужная Овца", false),
        "costume_snow_golem.png" => ("Костюм Снеговика", false),
        "costume_spider.png" => ("Костюм Паука", false),

        // Custom & Downloaded
        "skin_ender_ninja.png" => ("Эндер Ниндзя", false),
        "skin_shadow.png" => ("Тень", false),
        "skin_hunter.png" => ("Охотник", false),
        "skin_cyber_suit.png" => ("Кибер Костюм", false),
        "skin_stone_golem.png" => ("Каменный Голем", false),
        "skin_swamp_dweller.png" => ("Болотник", false),
        "skin_hoodie_guy.png" => ("Парень в худи", false),
        "skin_adventurer.png" => ("Искатель приключений", false),
        "skin_soldier.png" => ("Солдат", false),
        "skin_wanderer.png" => ("Странник", false),
        "skin_stealth.png" => ("Стелс", false),
        "skin_camper.png" => ("Следопыт", false),
        "skin_urban_style.png" => ("Уличный стиль", false),
        "skin_retro_crafter.png" => ("Ретро Крафтер", false),

        _ => ("", filename.contains("slim")),
    }
}

fn generate_skins_catalog() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("skins_catalog.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let skins_dir = Path::new(&manifest_dir).join("assets").join("skins");

    let priority_order = [
        "zek_batya.png",
        "steve.png",
        "alex.png",
        "entity_303.png",
        "herobrine.png",
        "town_woman.png",
        "town_castaway.png",
        "city_king.png",
        "city_queen.png",
        "skin_knight.png",
        "costume_creeper.png",
        "costume_enderman.png",
        "costume_skeleton.png",
        "costume_zombie.png",
        "city_blacksmith.png",
        "city_mage.png",
    ];

    let mut all_files: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&skins_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "png") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    all_files.push(name.to_string());
                }
            }
        }
    }

    // Sort: priority first in exact order, then others alphabetically by display name
    all_files.sort_by(|a, b| {
        let idx_a = priority_order.iter().position(|&x| x == a);
        let idx_b = priority_order.iter().position(|&x| x == b);
        match (idx_a, idx_b) {
            (Some(ia), Some(ib)) => ia.cmp(&ib),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                let (name_a, _) = get_skin_meta(a);
                let (name_b, _) = get_skin_meta(b);
                name_a.cmp(name_b).then_with(|| a.cmp(b))
            }
        }
    });

    let mut code = String::new();
    code.push_str("// Auto-generated by build.rs. Do not edit.\n\n");
    code.push_str("#[derive(Clone, Copy, Debug)]\n");
    code.push_str("pub struct EmbeddedSkin {\n");
    code.push_str("    pub name: &'static str,\n");
    code.push_str("    pub filename: &'static str,\n");
    code.push_str("    pub bytes: &'static [u8],\n");
    code.push_str("    pub is_slim: bool,\n");
    code.push_str("}\n\n");
    code.push_str("pub static EMBEDDED_SKINS: &[EmbeddedSkin] = &[\n");

    for file in &all_files {
        let (name, is_slim) = get_skin_meta(file);
        let display_name = if name.is_empty() {
            file.trim_end_matches(".png")
        } else {
            name
        };
        code.push_str(&format!(
            "    EmbeddedSkin {{\n        name: {:?},\n        filename: {:?},\n        bytes: include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/skins/{}\")),\n        is_slim: {},\n    }},\n",
            display_name, file, file, is_slim
        ));
    }

    code.push_str("];\n");

    fs::write(&dest_path, code).unwrap();
}
