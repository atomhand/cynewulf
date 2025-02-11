use super::MarkovChainModel;
use bevy::prelude::*;
use std::collections::HashSet;

// Each empire has its own separately configured name generator. (but the list of exhausted names is shared across everyone)

#[derive(Resource)]
pub struct UsedPlanetNames(HashSet<String>);

impl Default for UsedPlanetNames {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

pub struct PlanetNameGenerator {
    markov: MarkovChainModel,
}

use rand::prelude::*;

impl PlanetNameGenerator {
    // just the list of exoplanet proper names from wikipedia, for now
    pub const SOURCE_NAMES: &'static [&'static str] = &[
        "Abol",
        "Ægir",
        "Agouto",
        "Ahra",
        "Albmi",
        "Alef",
        "Amateru",
        "Arber",
        "Arion",
        "Arkas",
        "Astrolábos",
        "Asye",
        "Aumatex",
        "Awasis",
        "Awohali",
        "Babylonia",
        "Bagan",
        "Baiduri",
        "Bambaruush",
        "Banksia",
        "Barajeel",
        "Beirut",
        "Bélisama",
        "Bendida",
        "Bocaprins",
        "Boinayel",
        "Brahe",
        "Buru",
        "Caleuche",
        "Catalineta",
        "Cayahuanca",
        "Chura",
        "Cruinlagh",
        "Cuancoá",
        "Cuptor",
        "Dagon",
        "Dimidium",
        "Ditsö̀",
        "Dopere",
        "Draugr",
        "Dulcinea",
        "Eburonia",
        "Eiger",
        "Enaiposha",
        "Equiano",
        "Eyeke",
        "Finlay",
        "Fold",
        "Fortitudo",
        "Galileo",
        "Ganja",
        "Ġgantija",
        "Göktürk",
        "Guarani",
        "Guataubá",
        "Haik",
        "Hairu",
        "Halla",
        "Hämarik",
        "Harriot",
        "Hiisi",
        "Hypatia",
        "Ibirapitá",
        "Indépendance",
        "Iolaus",
        "Isagel",
        "Isli",
        "Ixbalanqué",
        "Iztok",
        "Janssen",
        "Jebus",
        "Kavian",
        "Kererū",
        "Khomsa",
        "Koyopa'",
        "Kráľomoc",
        "Krotoa",
        "Kua'kua",
        "Laligurans",
        "Leklsullun",
        "Lete",
        "Levantes",
        "Lipperhey",
        "Madalitso",
        "Madriu",
        "Maeping",
        "Magor",
        "Majriti",
        "Makombé",
        "Makropulos",
        "Mastika",
        "Melquíades",
        "Meztli",
        "Mintome",
        "Mulchatna",
        "Nachtwacht",
        "Najsakopajk",
        "Nakanbé",
        "Naqaỹa",
        "Naron",
        "Negoiu",
        "Neri",
        "Noifasui",
        "Onasilos",
        "Orbitar",
        "Peitruss",
        "Perwana",
        "Phailinsiam",
        "Phobetor",
        "Pipitea",
        "Pirx",
        "Pollera",
        "Poltergeist",
        "Puli",
        "Qingluan",
        "Quijote",
        "Ramajay",
        "Regoč",
        "Riosar",
        "Rocinante",
        "Saffar",
        "Samagiya",
        "Samh",
        "Sancho",
        "Santamasa",
        "Sazum",
        "Sissi",
        "Smertrios",
        "Spe",
        "Staburags",
        "Su",
        "Sumajmajta",
        "Surt",
        "Tadmor",
        "Tahay",
        "Tanzanite",
        // Due to lack of other spaces in the current dataset, these wouldn't work so well
        //"Taphao Kaew",
        //"Taphao Thong",
        "Taphao",
        "Kaew",
        "Thong",
        "Tassili",
        "Teberda",
        "Thestias",
        "Toge",
        "Tondra",
        "Trimobe",
        "Tryzub",
        "Tumearandu",
        "Tylos",
        "Ugarit",
        "Umbäässa",
        "Veles",
        //"Victoriapeak",
        "Viculus",
        "Viriato",
        "Vlasina",
        "Vytis",
        "Wadirum",
        "Wangshu",
        "Xólotl",
        "Xolotlan",
        "Yanyan",
        "Yvaga",
        "Zembretta",
        "Earth",
        "Terra",
    ];

    pub fn new(used_planet_names: &mut UsedPlanetNames) -> Self {
        let mut markov = MarkovChainModel::new(3);
        let names = Self::create_biased_input_set();
        for starname in Self::SOURCE_NAMES {
            used_planet_names.0.insert(starname.to_string());
        }
        markov.build(&names, 0.00001);

        Self { markov }
    }

    fn create_biased_input_set() -> Vec<String> {
        let mut rng = rand::rng();

        let ascii_only = Self::SOURCE_NAMES
            .into_iter()
            .filter(|x| x.is_ascii())
            .collect::<Vec<_>>();

        let n = Self::SOURCE_NAMES.len() / 2;
        let subset = ascii_only
            .choose_multiple(&mut rng, n)
            .map(|x| x.to_string())
            .collect::<Vec<_>>();

        let letter_to_skip = subset.choose(&mut rng).unwrap().chars().next().unwrap();

        let with_skipped = subset
            .iter()
            .filter_map(|x| {
                if x.chars().next() != Some(letter_to_skip) {
                    Some(x.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        if with_skipped.len() > 20 {
            with_skipped
        } else {
            subset
        }
    }

    pub fn next(&mut self, used_planet_names: &mut UsedPlanetNames) -> String {
        let mut res: String = self.markov.generate();

        while res.len() > 15 || used_planet_names.0.contains(&res) {
            res = self.markov.generate();
        }
        used_planet_names.0.insert(res.clone());

        res
    }
}
