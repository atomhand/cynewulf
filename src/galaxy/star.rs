use bevy::prelude::*;
use super::GalaxyConfig;

#[derive(Component)]
pub struct OverlaysTriangulationVertex {

}

#[derive(Component)]
pub struct Star {
    pub pos : Vec3,
    pub node_id : u32,
    pub orbiters : Vec::<Entity>,
    pub mass : f32, // in stellar masses
    pub name : String,
}

impl Star {
    // Structure

    // Optional: Begin consonstant
    // Begin_part
    // Middle part -- Can be doubled? (No repetition)
    // Optional: Ext. Middle part + Vowel
    // Ending

    fn system_radius_au(&self) -> f32 {
        7.0
    }
    pub fn system_radius_actual(&self) -> f32{
        self.system_radius_au() * GalaxyConfig::AU_SCALE
    }

    pub const NAMES: &'static [&'static str] = &[
        "Acamar",
        "Alpha",
        "Arian",
        "Aghran",
        "Al-aqrab",
        "Ahir",
        "Andromeda",
        "An-nhar",
        "Aquarius",
        "Auriga",
        "Auron",
        "Austrinus",

        "Barrion",
        "Belhammond",
        "Berria",
        "Beta",
        "Brand",

        "Cassiopeia",
        "Cetus",
        "Caladan",
        "Canes",
        //"Canes Venatici",
        "Carina",
        "Centaurus",
        "Cepheus",
        "Cernan",
        "Certus",
        "Coma",
        "Corascia",
        "Corusca",
        //"Coma Berenices",
        "Crax",
        "Crucis",
        "Cygnus",
        "Cygnon",

        "Daedlus",
        "Delrune",
        "Dendra",
        "Dietmar",
        "Dithimar",
        "Dolens",
        "Doom",
        "Dragon",
        "Dresden",
        "Dromar",
        "Dryad",
        "Dyad",

        "Ea",
        "Eidre",
        "Elisande",
        "Elrond",
        "Eradon",
        "Eriand",
        "Eridanus",
        "Eros",
        "Ersand",
        "Ezar",
        "Ezor",

        "Fallia",
        "Fand",
        "Feylan",
        "Fellian",
        "Font",
        "Fornax",
        "Fornath",
        "Foryx",
        "Fu",
        "Fune",
        "Furan",
        "Furiosa",
        "Furyx",

        "Ganon",
        "Gail",
        "Geisand",
        "Gemini",
        "Gydaron",
        "Giausar",
        "Ginan",
        "Gloas",
        "Gomeisa",
        "Gond",
        "Guahayona",
        "Gudja",
        "Gumala",
        "Gyron",

        "Hadar",
        "Haedis",
        "Hatysa",
        "Helion",
        "Helm",
        "Hexam",
        "Hydra",
        "Hylix",
        "Hund",
        "Hunor",

        "Iklil",
        "Indus",
        "Irena",
        "Iskand",
        "Itonda",
        "Izar",

        "Jabbah",
        "Jalar",
        "Jeyan",
        "Jerush",
        "Jishui",
        "Joshan",

        "Kaewkosin",
        "Kalausi",
        "Kamuy",
        "Karaka",
        "Keid",
        "Keran",
        "Khambalia",
        "Kitalpha",
        "Kolan",
        "Komndor",
        "Kornephoros",
        "Kuma",
        "Kurhah",

        "Landrig",
        "Lantion",
        "Landuhar",
        "Larawag",
        "Lerna",
        "Lesath",
        "Lich",
        "Liscor",
        "Lonsan",
        "Lusan",
        "Lushar",
        "Lutris",
        "Lycilin",

        "Maasaym",
        "Macondo",
        "Marfik",
        "Marsic",
        "Menkalinan",
        "Menkar",
        "Mensa",
        "Mirach",
        "Miram",
        "Mizar",
        "Monch",
        "Monoceros",
        "Moriah",
        "Mouhoun",
        "Mpingo",
        "Muscida",

        "Nahn",
        "Nalbus",
        "Naledi",
        "Nekkar",
        "Nembus",
        "Nenque",
        "Nihal",
        "Nimh",
        "Noquisi",
        "Norion",
        "Nosaxa",
        "Nox",
        "Nuki",
        "Nusakan",
        "Nushagak",
        "Nyamien",

        "Oan",
        "Ogma",
        "Okab",
        "Omoyo",
        "Ophiiochus",
        "Orion",
        "Orkaria",
        "Oryx",

        "Parumleo",
        "Petra",
        "Peylus",
        "Pendayo",
        "Phact",
        "Phecda",
        "Pherkad",
        "Philhammon",
        "Phorion",
        "Phondar",
        "Pisces",
        "Pipirima",
        "Poerava",
        "Polaris",
        "Polis",
        "Pollux",
        "Prim",
        "Procyon",
        "Propus",
        "Proxam",

        "Ran",
        "Rasalas",
        "Rasalgethi",
        "Rastaban",
        "Regor",
        "Regulus",
        "Revati",
        "Rhaan",
        "Rigel",
        "Rohan",
        "Rotanev",
        "Ruchbah",
        "Rukbat",

        "Sabik",
        "Saclateni",
        "Sadachbia",
        "Sadalbari",
        "Sadr",
        "Saiph",
        "Samaya",
        "Sargas",
        "Sculptor",
        "Serpens",
        "Sextans",
        "Sham",
        "Sharjah",
        "Shaula",
        "Sheliak",
        "Sheratan",
        "Sika",
        "Sirius",
        "Situla",
        "Solaris",
        "Spica",
        "Stribor",
        "Styx",
        "Sualocin",
        "Subra",
        "Suhail",
        "Sulafat",
        "Syrma",

        "Tabit",
        "Taika",
        "Taiyangshou",
        "Taiyi",
        "Tangra",
        "Tarazad",
        "Tenthan",
        "Terebellum",
        "Tevel",
        "Thabit",
        "Theemin",
        "Thuban",
        "Tiaki",
        "Tianguan",
        "Tianyi",
        "Timir",
        "Tojil",
        "Toliman",
        "Tonatiuh",
        "Trantor",
        "Trinus",
        "Tsang",
        "Tsin",
        "Tucana",
        "Tuiren",
        "Tureis",
        
        "Uabb",
        "Uhlan",
        "Uht",
        "Ukdah",
        "Ukhab",
        "Uklun",
        "Ull",
        "Umman",
        "Unukalhai",
        "Unurgunite",
        "Ur",
        "Uruk",
        "Uten",
        "Uuba",

        "Vega",
        "Veylan",
        "Vindemiatrix",
        "Viss",
        "Virgo",
        "Voss",
        "Vox",

        "Wasat",
        "Walhannis",
        "Weynhab",
        "Wezen",
        "Windarr",
        "Wolcan",
        "Wulrant",
        "Wouri",
        "Wurren",

        "Xamidimura",
        "Xerant",
        "Xeyhab",
        "Xihe",
        "Xo",
        "Xoss",
        "Xuange",

        "Yanda",
        "Yantris",
        "Yed",
        "Yehalan",
        "Yildun",
        "Yixam",
        "Yondarr",
        "Yoss",
        "Yorian",
        "Yoht",

        "Zaniah",
        "Zass",
        "Zaurak",
        "Zhang",
        "Zibal",
        "Zin",
        "Zoss",
        "Zoloss",
        "Zosma",
        "Zuben"
    ];

    pub fn new(star_name_gen : &mut crate::markov_chain::StarNameGenerator, id : u32, pos : Vec3, stellar_masses : f32) -> Star {
        Star {
            node_id : id,
            pos,
            orbiters : Vec::new(),
            mass : stellar_masses,
            name : star_name_gen.next()
        }
    }

    pub fn get_raw_radius(&self) -> f32 {
        // return as fraction of Sun mass
        self.mass.sqrt()
    }
    
    pub fn get_scaled_radius(&self) -> f32 {
        // return as fraction of Sun radius
        self.get_raw_radius() * GalaxyConfig::SOLAR_RADIUS
    }

    // https://physics.stackexchange.com/questions/6771/star-surface-temperature-vs-mass/6772#6772
    // Kelvin
    pub fn get_temperature(&self) -> f32 {
        self.mass.powf(0.625) * 5772.0
    }

    pub fn _get_luminosity(&self) -> f32 {
        self.mass.powf(3.5)
    }

    fn simple_planck(temperature : f32) -> Vec3 {
        let mut res : Vec3 = Vec3::ZERO;
        let m = 1.0;
        for i in 0..3 {  // +=.1 if you want to better sample the spectrum.
            let f = 1.+0.5*i as f32; 
            res[i as usize] += 10.0 / m * (f*f*f) / (f32::exp(19.0e3*f/temperature) - 1.);  // Planck law
        }

        //res = res / res.max_element();
        res
    }

    pub fn get_color(&self) -> Vec3 {
        let planck = Self::simple_planck(self.get_temperature());
        planck
    }
}