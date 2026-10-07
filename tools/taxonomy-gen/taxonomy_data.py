"""Source data for the fixed-wing aircraft taxonomy.

Matching is done on NORMALIZED strings:
  make_norm  = uppercase, every non [A-Z0-9] char -> space, collapse spaces, trim
  model_norm = uppercase, strip every non [A-Z0-9] char   ("PA-28-161" -> "PA28161")
All regexes must be compatible with the Rust `regex` crate (no lookaround, no backrefs).
"""

# ---------------------------------------------------------------- classes
CLASSES = [
    ("sep_fixed", "Single-engine piston, fixed tricycle gear",
     "Trainers, tourers and fixed-gear high performance singles (C172, PA-28, SR22, DA40).",
     dict(engines=1, propulsion="piston", gear="fixed", gear_config="tricycle")),
    ("sep_retract", "Single-engine piston, retractable gear",
     "Complex / high-performance retract singles (Bonanza, Mooney, Arrow, C210, PA-46 piston).",
     dict(engines=1, propulsion="piston", gear="retract", gear_config="tricycle")),
    ("sep_tailwheel", "Single-engine piston, conventional gear",
     "Tailwheel light, utility and bush aircraft (J-3, Super Cub, Citabria, C180/185, Husky, Maule).",
     dict(engines=1, propulsion="piston", gear="fixed", gear_config="tailwheel")),
    ("sep_aerobatic", "Aerobatic",
     "Purpose-built aerobatic aircraft (Pitts, Extra, Decathlon, Edge 540, Christen Eagle).",
     dict(engines=1, propulsion="piston", gear="fixed", gear_config="tailwheel")),
    ("lsa", "Light sport (S-LSA / E-LSA)",
     "Factory light-sport designs (CTLS, Tecnam P92/P2008, Cessna 162, Icon A5, SportCruiser).",
     dict(engines=1, propulsion="piston", gear="fixed", gear_config="tricycle", cert="lsa")),
    ("experimental", "Experimental amateur-built (kit)",
     "Kit/plans-built types usually registered under the builder's name (RV, Lancair, Glasair, Kitfox).",
     dict(engines=1, propulsion="piston", cert="experimental")),
    ("vintage", "Vintage & warbird",
     "Ex-military trainers/fighters and classic biplanes (Stearman, T-6, P-51, Yak-52, L-39, Waco).",
     dict(cert="limited_or_exhibition")),
    ("agricultural", "Agricultural",
     "Purpose-built ag aircraft (Air Tractor, Thrush, Ag Cat, Pawnee, AgWagon).",
     dict(engines=1, gear="fixed", gear_config="tailwheel", cert="restricted_or_normal")),
    ("amphibian", "Amphibian / flying boat",
     "Hull amphibians (Lake, Grumman Goose/Widgeon/Mallard, Seabee, SeaRey). Floatplanes stay in their base class.",
     dict(amphibian=True)),
    ("mep", "Multi-engine piston",
     "Piston twins (Baron, Seneca, Seminole, C310/340/414/421, Aztec, Navajo, DA42).",
     dict(engines=2, propulsion="piston", gear="retract", gear_config="tricycle")),
    ("set", "Single-engine turboprop",
     "Caravan, PC-12, TBM, PA-46 turbine, Kodiak, PC-6.",
     dict(engines=1, propulsion="turboprop")),
    ("met", "Multi-engine turboprop (business/utility)",
     "King Air, Conquest, MU-2, Cheyenne, Turbo Commander, Merlin, P180.",
     dict(engines=2, propulsion="turboprop", gear="retract", gear_config="tricycle")),
    ("regional_turboprop", "Commuter & regional turboprop",
     "19+ seat commuter/airline turboprops (B1900, Metro, Jetstream, EMB-120, Saab 340, ATR, Dash 8, Twin Otter).",
     dict(engines=2, propulsion="turboprop")),
    ("bizjet_light", "Business jet - very light / light",
     "Citation Mustang/CJ/I/II/V, Phenom, Eclipse, HondaJet, Learjet 20/30/40, Beechjet/Premier, SF50.",
     dict(engines=2, propulsion="turbofan", gear="retract", gear_config="tricycle")),
    ("bizjet_midsize", "Business jet - midsize / super-midsize",
     "Citation Excel/III/Sovereign/Latitude/X/Longitude, Hawker 125, Learjet 55/60, G100-G280, Challenger 300.",
     dict(engines=2, propulsion="turbofan", gear="retract", gear_config="tricycle")),
    ("bizjet_large", "Business jet - large / long range",
     "Gulfstream GII-G700, Challenger 600-650, Global, Falcon 50/900/2000/7X/8X, Legacy 600.",
     dict(engines=2, propulsion="turbofan", gear="retract", gear_config="tricycle")),
    ("regional_jet", "Regional jet",
     "CRJ, ERJ-135/145, E-Jets, BAe 146/Avro RJ, Fokker 70/100, 328JET.",
     dict(engines=2, propulsion="turbofan", gear="retract", gear_config="tricycle", cert="part25")),
    ("narrowbody", "Narrowbody airliner",
     "737, 757, 727, 707, 717, DC-8, DC-9/MD-80/90, A220, A320 family.",
     dict(propulsion="turbofan", gear="retract", gear_config="tricycle", cert="part25")),
    ("widebody", "Widebody airliner",
     "747, 767, 777, 787, DC-10/MD-11, L-1011, A300/310/330/340/350/380.",
     dict(propulsion="turbofan", gear="retract", gear_config="tricycle", cert="part25")),
    ("legacy_transport", "Legacy piston/turboprop transport",
     "DC-3, DC-4/6/7, Convair 240-640, C-46, L-188 Electra, L-382/C-130.",
     dict(gear="retract", cert="part25_or_car4b")),
]

# Fallback buckets for AIR records that match no family (assigned from NTSB fields).
FALLBACKS = [
    ("unclassified_experimental", "Unclassified amateur-built", dict(homebuilt=True)),
    ("unclassified_jet", "Unclassified jet", dict(eng_type=["TF", "TJ"])),
    ("unclassified_turboprop", "Unclassified turboprop", dict(eng_type=["TP"])),
    ("unclassified_mep", "Unclassified multi-engine piston", dict(eng_type=["REC"], num_eng_min=2)),
    ("unclassified_sep", "Unclassified single-engine piston", dict(eng_type=["REC"], num_eng_max=1)),
    ("unknown", "Unknown", dict()),
]

# ---------------------------------------------------------------- make alias groups
# Regexes on make_norm.
MAKES = {
    "cessna": [r"^CESSNA\b", r"^REIMS\b", r"^TEXTRON\b"],
    "beech": [r"^BEECH", r"^RAYTHEON\b", r"^HAWKER BEECHCRAFT\b", r"^TEXTRON\b"],
    "piper": [r"^PIPER\b", r"^NEW PIPER\b"],
    "mooney": [r"^MOONEY\b"],
    "cirrus": [r"^CIRRUS\b"],
    "diamond": [r"^DIAMOND\b"],
    "grumman_american": [r"^GRUMMAN\b", r"^AMERICAN AVIATION\b", r"^GULFSTREAM AMERICAN\b",
                         r"^AMERICAN GEN", r"^TIGER AIRCRAFT\b"],
    "socata": [r"^SOCATA\b", r"^EADS SOCATA\b", r"^AEROSPATIALE\b", r"^DAHER\b"],
    "commander_single": [r"^ROCKWELL\b", r"^COMMANDER\b", r"^AERO COMMANDER\b"],
    "navion": [r"^NAVION\b", r"^RYAN\b", r"^NORTH AMERICAN\b", r"^TUSCO\b"],
    "erco": [r"^ERCO\b", r"^ENGINEERING (AND|&)? ?RESEARCH\b", r"^ALON\b", r"^FORNEY\b",
             r"^FORNAIRE\b", r"^AIRCOUPE\b", r"^MOONEY\b"],
    "luscombe": [r"^LUSCOMBE\b", r"^SILVAIRE\b", r"^TEMCO\b"],
    "aeronca": [r"^AERONCA\b", r"^CHAMPION\b", r"^AMERICAN CHAMPION\b", r"^BELLANCA\b"],
    "champion": [r"^CHAMPION\b", r"^AMERICAN CHAMPION\b", r"^BELLANCA\b", r"^AERONCA\b"],
    "bellanca": [r"^BELLANCA\b"],
    "taylorcraft": [r"^TAYLORCRAFT\b", r"^TAYLOR\b"],
    "stinson": [r"^STINSON\b"],
    "maule": [r"^MAULE\b"],
    "aviat": [r"^AVIAT\b", r"^CHRISTEN\b", r"^PITTS\b", r"^HUSKY\b"],
    "christen": [r"^CHRISTEN\b", r"^AVIAT\b"],
    "cubcrafters": [r"^CUB ?CRAFTERS\b"],
    "american_legend": [r"^AMERICAN LEGEND\b"],
    "extra": [r"^EXTRA\b", r"^WALTER EXTRA\b"],
    "zivko": [r"^ZIVKO\b"],
    "sukhoi": [r"^SUKHOI\b"],
    "globe_temco": [r"^GLOBE\b", r"^TEMCO\b", r"^SWIFT\b"],
    "helio": [r"^HELIO\b"],
    "gippsland": [r"^GIPPSLAND\b", r"^MAHINDRA\b"],
    "lake": [r"^LAKE\b", r"^REVO\b", r"^CONSOLIDATED AERONAUTICS\b", r"^COLONIAL\b"],
    "icon": [r"^ICON\b"],
    "flight_design": [r"^FLIGHT DESIGN\b"],
    "tecnam": [r"^TECNAM\b", r"^COSTRUZIONI AERONAUTICHE TECNAM\b"],
    "czech_sport": [r"^CZECH\b", r"^PIPER\b"],
    "sling": [r"^SLING\b", r"^THE AIRPLANE FACTORY\b", r"^AIRPLANE FACTORY\b"],
    "pipistrel": [r"^PIPISTREL\b"],
    "evektor": [r"^EVEKTOR\b"],
    "remos": [r"^REMOS\b"],
    "jabiru": [r"^JABIRU\b"],
    "aeropro": [r"^AEROPRO\b"],
    "vans": [r"^VAN ?S\b"],
    "lancair": [r"^LANCAIR\b", r"^NEICO\b"],
    "columbia": [r"^COLUMBIA\b", r"^LANCAIR\b", r"^CESSNA\b"],
    "glasair": [r"^GLASAIR\b", r"^GLASSAIR\b", r"^STODDARD\b"],
    "kitfox": [r"^KITFOX\b", r"^DENNEY\b", r"^SKYSTAR\b"],
    "zenith": [r"^ZENITH\b", r"^ZENAIR\b", r"^AMD\b"],
    "sonex": [r"^SONEX\b"],
    "rans": [r"^RANS\b"],
    "canard": [r"^RUTAN\b", r"^VELOCITY\b", r"^COZY\b"],
    "murphy": [r"^MURPHY\b"],
    "just": [r"^JUST AIRCRAFT\b"],
    "kolb": [r"^KOLB\b"],
    "avid": [r"^AVID\b"],
    "progressive": [r"^PROGRESSIVE AERODYNE\b", r"^SEAREY\b"],
    "seawind": [r"^SEAWIND\b"],
    "republic": [r"^REPUBLIC\b"],
    "grumman_amph": [r"^GRUMMAN\b", r"^MCKINNON\b", r"^FRAKES\b"],
    "dehavilland": [r"^DE ?HAVILLAND\b", r"^DHC\b", r"^VIKING\b", r"^BOMBARDIER\b"],
    "pilatus": [r"^PILATUS\b"],
    "britten_norman": [r"^BRITTEN", r"^PILATUS BRITTEN"],
    "partenavia": [r"^PARTENAVIA\b", r"^VULCANAIR\b"],
    "kodiak": [r"^QUEST\b", r"^KODIAK\b", r"^DAHER\b"],
    "epic": [r"^EPIC\b"],
    "pac": [r"^PACIFIC AEROSPACE\b"],
    "air_tractor": [r"^AIR TRACTOR\b"],
    "thrush": [r"^THRUSH\b", r"^AYRES\b", r"^ROCKWELL\b", r"^SNOW\b", r"^AERO COMMANDER\b",
               r"^QUALITY AEROSPACE\b"],
    "agcat": [r"^GRUMMAN\b", r"^SCHWEIZER\b", r"^AG CAT\b", r"^ALLIED AG CAT\b"],
    "weatherly": [r"^WEATHERLY\b"],
    "pzl": [r"^PZL\b", r"^WSK\b"],
    "twin_commander": [r"^AERO COMMANDER\b", r"^ROCKWELL\b", r"^GULFSTREAM\b",
                       r"^TWIN COMMANDER\b", r"^COMMANDER\b"],
    "aerostar": [r"^PIPER\b", r"^AEROSTAR\b", r"^TED SMITH\b", r"^MACHEN\b"],
    "mitsubishi": [r"^MITSUBISHI\b"],
    "piaggio": [r"^PIAGGIO\b"],
    "swearingen": [r"^SWEARINGEN\b", r"^FAIRCHILD\b", r"^M7 AEROSPACE\b"],
    "jetstream": [r"^BRITISH AEROSPACE\b", r"^BAE\b", r"^JETSTREAM\b", r"^HANDLEY PAGE\b",
                  r"^SCOTTISH AVIATION\b"],
    "saab": [r"^SAAB\b"],
    "embraer": [r"^EMBRAER\b"],
    "atr": [r"^ATR\b", r"^AVIONS DE TRANSPORT\b", r"^AEROSPATIALE\b"],
    "dornier": [r"^DORNIER\b", r"^FAIRCHILD DORNIER\b", r"^RUAG\b"],
    "shorts": [r"^SHORT\b", r"^SHORTS\b"],
    "casa": [r"^CASA\b", r"^CONSTRUCCIONES AERONAUTICAS\b", r"^AIRBUS\b", r"^IPTN\b"],
    "let": [r"^LET\b"],
    "fokker": [r"^FOKKER\b", r"^FAIRCHILD\b"],
    "eclipse": [r"^ECLIPSE\b", r"^ONE AVIATION\b"],
    "honda": [r"^HONDA\b"],
    "learjet": [r"^LEAR", r"^GATES LEAR", r"^BOMBARDIER\b"],
    "hawker": [r"^HAWKER\b", r"^RAYTHEON\b", r"^BRITISH AEROSPACE\b", r"^BAE\b",
               r"^HAWKER SIDDELEY\b", r"^DE ?HAVILLAND\b", r"^BEECH"],
    "beechjet": [r"^BEECH", r"^RAYTHEON\b", r"^HAWKER\b", r"^MITSUBISHI\b", r"^TEXTRON\b"],
    "dassault": [r"^DASSAULT\b", r"^AVIONS MARCEL DASSAULT\b", r"^FALCON\b"],
    "gulfstream": [r"^GULFSTREAM\b", r"^GRUMMAN\b", r"^ISRAEL AIRCRAFT\b", r"^ISRAEL AEROSPACE\b",
                   r"^IAI\b"],
    "iai": [r"^ISRAEL AIRCRAFT\b", r"^ISRAEL AEROSPACE\b", r"^IAI\b"],
    "bombardier": [r"^BOMBARDIER\b", r"^CANADAIR\b"],
    "sabreliner": [r"^SABRELINER\b", r"^NORTH AMERICAN\b", r"^ROCKWELL\b"],
    "lockheed": [r"^LOCKHEED\b"],
    "boeing": [r"^BOEING\b"],
    "douglas": [r"^DOUGLAS\b", r"^MCDONNELL\b", r"^BOEING\b", r"^BASLER\b"],
    "airbus": [r"^AIRBUS\b", r"^BOMBARDIER\b"],
    "bae146": [r"^BRITISH AEROSPACE\b", r"^BAE\b", r"^AVRO\b"],
    "convair": [r"^CONVAIR\b", r"^GENERAL DYNAMICS\b", r"^KELOWNA\b"],
    "curtiss": [r"^CURTISS\b"],
    "stearman": [r"^BOEING\b", r"^STEARMAN\b"],
    "north_american": [r"^NORTH AMERICAN\b", r"^NORTH AMER\b", r"^NOORDUYN\b", r"^CANADIAN CAR\b",
                       r"^CCF\b", r"^CAVALIER\b", r"^COMMONWEALTH\b"],
    "yak": [r"^YAK", r"^IAK\b", r"^AEROSTAR S ?A\b", r"^NANCHANG\b"],
    "aero_vodochody": [r"^AERO VODOCHODY\b", r"^AERO\b"],
    "waco": [r"^WACO\b", r"^CLASSIC AIRCRAFT\b"],
    "ryan": [r"^RYAN\b"],
    "fairchild_vintage": [r"^FAIRCHILD\b"],
    "great_lakes": [r"^GREAT LAKES\b"],
}

# ---------------------------------------------------------------- families
# F(id, name, class, makes, model_re, attrs, variants, priority, homebuilt_model_re, notes)
FAMILIES = []


def F(id, name, cls, makes, model_re, attrs=None, variants=(), priority=100,
      homebuilt_model_re=None, notes=None):
    FAMILIES.append(dict(id=id, name=name, cls=cls, makes=makes, model_re=model_re,
                         attrs=attrs or {}, variants=list(variants), priority=priority,
                         homebuilt_model_re=homebuilt_model_re, notes=notes))


def V(id, name, model_re, cls=None, **attrs):
    return dict(id=id, name=name, model_re=model_re, cls=cls, attrs=attrs)


HW = dict(wing="high")
LW = dict(wing="low")

# ======== Cessna singles
F("cessna_120_140", "Cessna 120/140", "sep_tailwheel", ["cessna"], r"^C?1[24]0A?$", HW,
  [V("c120", "120", r"^C?120"), V("c140", "140 / 140A", r"^C?140")])
F("cessna_150_152", "Cessna 150/152", "sep_fixed", ["cessna"],
  r"^(C|F|A|FA)?15[02]([A-Z][A-Z0-9]*)?$|^AEROBAT", HW,
  [V("aerobat", "A150/A152 Aerobat", r"^F?A15[02]|AEROBAT"),
   V("c150", "150", r"^(C|F)?150"), V("c152", "152", r"^(C|F)?152")])
F("cessna_162", "Cessna 162 Skycatcher", "lsa", ["cessna"], r"^C?162$|^SKYCATCHER", HW)
F("cessna_170", "Cessna 170", "sep_tailwheel", ["cessna"], r"^C?170[AB]?$", HW)
F("cessna_172", "Cessna 172 Skyhawk", "sep_fixed", ["cessna"],
  r"^(C|F|P|R|FR)?172([A-Z][A-Z0-9]*)?$|^SKYHAWK|^T41[A-D]?$|^CUTLASS|^HAWKXP", HW,
  [V("c172rg", "172RG Cutlass RG", r"^C?172RG|^CUTLASSRG", cls="sep_retract", gear="retract"),
   V("r172", "R172 / FR172 Hawk XP", r"^F?R172|^HAWKXP|^172XP"),
   V("p172", "P172 Powermatic", r"^P172"),
   V("c172_early", "172 / 172A (straight tail)", r"^C?172A?$"),
   V("c172_bm", "172B-172M", r"^(C|F)?172[B-M]$"),
   V("c172n", "172N", r"^(C|F)?172N"),
   V("c172pq", "172P / 172Q", r"^(C|F)?172[PQ]"),
   V("c172r", "172R", r"^C?172R$|^C?172R[0-9]"),
   V("c172s", "172S / 172SP", r"^C?172S($|P|[0-9])"),
   V("t41", "T-41 Mescalero", r"^T41")])
F("cessna_175", "Cessna 175 Skylark", "sep_fixed", ["cessna"], r"^C?175[A-C]?$|^SKYLARK", HW)
F("cessna_177", "Cessna 177 Cardinal", "sep_fixed", ["cessna"], r"^C?177([A-Z][A-Z0-9]*)?$|^CARDINAL", HW,
  [V("c177rg", "177RG Cardinal RG", r"^C?177RG|^CARDINALRG", cls="sep_retract", gear="retract"),
   V("c177", "177 / 177A / 177B", r"^C?177")])
F("cessna_180", "Cessna 180 Skywagon", "sep_tailwheel", ["cessna"], r"^C?180[A-K]?$", HW)
F("cessna_182", "Cessna 182 Skylane", "sep_fixed", ["cessna"],
  r"^(C|F|R|T|TR|FR)?182([A-Z][A-Z0-9]*)?$|^SKYLANE", HW,
  [V("c182rg", "R182 / TR182 Skylane RG", r"^(F?R|TR)182|^SKYLANERG|^182RG", cls="sep_retract", gear="retract"),
   V("t182", "T182 Turbo Skylane", r"^T182"),
   V("c182", "182 / 182A-182T", r"^(C|F)?182")])
F("cessna_185", "Cessna 185 Skywagon", "sep_tailwheel", ["cessna"], r"^A?185[A-F]?[A-Z0-9]*$|^SKYWAGON", HW)
F("cessna_188", "Cessna 188 AgWagon/AgTruck/AgHusky", "agricultural", ["cessna"],
  r"^A?188[A-Z0-9]*$|^AG(WAGON|TRUCK|HUSKY|PICKUP)", dict(propulsion="piston", wing="low"))
F("cessna_190_195", "Cessna 190/195", "sep_tailwheel", ["cessna"], r"^C?19[05][AB]?$|^LC126", HW)
F("cessna_206", "Cessna 205/206/207 Stationair", "sep_fixed", ["cessna"],
  r"^(C|U|P|TU|TP|T)?20[567]([A-Z][A-Z0-9]*)?$|^STATIONAIR|^SKYWAGON20[67]", HW,
  [V("c205", "205", r"^C?205"),
   V("c206", "206 / U206 / P206 / T206", r"^(C|U|P|TU|TP|T)?206|^STATIONAIR"),
   V("c207", "207 Skywagon / Stationair 8", r"^T?207")])
F("cessna_210", "Cessna 210 Centurion", "sep_retract", ["cessna"],
  r"^(C|T|P|TP)?210([A-Z][A-Z0-9]*)?$|^CENTURION", HW,
  [V("p210", "P210 Pressurized Centurion", r"^T?P210", pressurized=True),
   V("t210", "T210 Turbo Centurion", r"^T210"),
   V("c210", "210", r"^C?210")])
F("cessna_corvalis", "Columbia/Cessna 300/350/400 (Corvalis)", "sep_fixed", ["columbia"],
  r"^LC4[012]|^COLUMBIA|^CORVALIS|^T?240$|^350$|^400$", LW, priority=150,
  notes="'350'/'400' only match make CESSNA/COLUMBIA/LANCAIR; a TEXTRON-made 350 falls to King Air.")

# ======== Cessna twins / turboprops / jets
F("cessna_303", "Cessna 303 Crusader", "mep", ["cessna"], r"^T?303$|^CRUSADER", LW)
F("cessna_310_320", "Cessna 310/320", "mep", ["cessna"], r"^(T|E)?3[12]0[A-Z0-9]*$|^U3[AB]?$|^SKYKNIGHT", LW,
  [V("c310", "310", r"^T?310|^U3"), V("c320", "320 Skyknight", r"^(E)?320|^SKYKNIGHT")])
F("cessna_336_337", "Cessna 336/337 Skymaster", "mep", ["cessna"],
  r"^(T|P|F|FT|M|T)?33[67][A-Z0-9]*$|^SKYMASTER|^O2[AB]?$", dict(wing="high", engine_layout="centerline"),
  [V("c336", "336 (fixed gear)", r"^336", gear="fixed"),
   V("p337", "P337 Pressurized", r"^T?P337", pressurized=True),
   V("c337", "337", r"337|^O2")])
F("cessna_335_340", "Cessna 335/340", "mep", ["cessna"], r"^3(35|40)[A-Z0-9]*$", dict(wing="low", pressurized=True))
F("cessna_401_402_411", "Cessna 401/402/411", "mep", ["cessna"], r"^4(01|02|11)[A-Z0-9]*$", LW,
  [V("c401", "401", r"^401"), V("c402", "402 / Businessliner / Utiliner", r"^402"), V("c411", "411", r"^411")])
F("cessna_404", "Cessna 404 Titan", "mep", ["cessna"], r"^404[A-Z0-9]*$|^TITAN", LW)
F("cessna_414_421", "Cessna 414/421", "mep", ["cessna"], r"^4(14|21)[A-Z0-9]*$|^CHANCELLOR|^GOLDENEAGLE",
  dict(wing="low", pressurized=True),
  [V("c414", "414 Chancellor", r"^414|^CHANCELLOR"), V("c421", "421 Golden Eagle", r"^421|^GOLDENEAGLE")])
F("cessna_406", "Cessna 406 Caravan II", "met", ["cessna"], r"^406$|^F406", LW)
F("cessna_425_441", "Cessna 425 Conquest I / 441 Conquest II", "met", ["cessna"],
  r"^4(25|41)[A-Z0-9]*$|^CONQUEST|^CORSAIR", dict(wing="low", pressurized=True),
  [V("c425", "425 Corsair / Conquest I", r"^425|^CORSAIR|^CONQUESTI$"), V("c441", "441 Conquest II", r"^441|^CONQUEST")])
F("cessna_208", "Cessna 208 Caravan", "set", ["cessna"], r"^C?208[AB]?[A-Z0-9]*$|^CARAVAN|^GRANDCARAVAN",
  dict(wing="high", gear="fixed", gear_config="tricycle"),
  [V("c208b", "208B Grand Caravan / EX", r"^C?208B|^GRANDCARAVAN"), V("c208", "208 / 208A Caravan", r"^C?208")])
F("cessna_408", "Cessna 408 SkyCourier", "regional_turboprop", ["cessna"], r"^408$|^SKYCOURIER",
  dict(wing="high", gear="fixed"))
F("citation_mustang", "Cessna Citation Mustang (510)", "bizjet_light", ["cessna"], r"^C?510$|^MUSTANG", LW)
F("citation_cj", "Cessna CitationJet / M2 / CJ1-CJ4 (525)", "bizjet_light", ["cessna"],
  r"^C?525[ABC]?[A-Z0-9]*$|^CJ[1-4]|^CITATIONJET|^CITATIONM2", LW,
  [V("cj4", "525C CJ4", r"^C?525C|^CJ4"), V("cj3", "525B CJ3", r"^C?525B|^CJ3"),
   V("cj2", "525A CJ2", r"^C?525A|^CJ2"), V("cj1_m2", "525 CitationJet / CJ1 / M2", r"^C?525|^CJ1|^CITATION")])
F("citation_500_series", "Cessna Citation I/II/V/Ultra/Encore/Bravo (500/501/550/551/560)", "bizjet_light",
  ["cessna"], r"^C?5(00|01|50|51|60)[A-Z0-9]*$|^T47", LW,
  [V("c500_501", "500/501 Citation I", r"^C?50[01]"),
   V("c550_551", "550/551 Citation II / S/II / Bravo", r"^C?55[01]|^T47"),
   V("c560", "560 Citation V / Ultra / Encore", r"^C?560")])
F("citation_excel", "Cessna Citation Excel/XLS (560XL)", "bizjet_midsize", ["cessna"], r"^C?560XL", LW, priority=150)
F("citation_650", "Cessna Citation III/VI/VII (650)", "bizjet_midsize", ["cessna"], r"^C?650$", LW)
F("citation_680", "Cessna Citation Sovereign/Latitude (680/680A)", "bizjet_midsize", ["cessna"],
  r"^C?680A?[A-Z0-9]*$|^SOVEREIGN|^LATITUDE", LW,
  [V("c680a", "680A Latitude", r"^C?680A|^LATITUDE"), V("c680", "680 Sovereign", r"^C?680|^SOVEREIGN")])
F("citation_700", "Cessna Citation Longitude (700)", "bizjet_midsize", ["cessna"], r"^C?700$|^LONGITUDE", LW)
F("citation_750", "Cessna Citation X (750)", "bizjet_midsize", ["cessna"], r"^C?750$|^CITATIONX", LW)

# ======== Piper
F("piper_j_cub", "Piper/Taylor J-2/J-3/J-4/J-5 Cub", "sep_tailwheel", ["piper", "taylorcraft"],
  r"^J[2345][A-Z0-9]*$|^CUB$|^L4[A-J]?$|^NE1$", HW,
  [V("j3", "J-3 Cub", r"^J3|^CUB$|^L4|^NE1"), V("j5", "J-5 Cub Cruiser", r"^J5"),
   V("j2_j4", "J-2 / J-4", r"^J[24]")])
F("piper_pa11", "Piper PA-11 Cub Special", "sep_tailwheel", ["piper"], r"^PA11", HW)
F("piper_pa12", "Piper PA-12 Super Cruiser", "sep_tailwheel", ["piper"], r"^PA12", HW)
F("piper_pa14_16", "Piper PA-14 Family Cruiser / PA-16 Clipper", "sep_tailwheel", ["piper"], r"^PA1[46]", HW)
F("piper_pa15_17", "Piper PA-15/PA-17 Vagabond", "sep_tailwheel", ["piper"], r"^PA1[57]", HW)
F("piper_pa18", "Piper PA-18 Super Cub", "sep_tailwheel", ["piper"], r"^PA18[A-Z0-9]*$|^SUPERCUB|^L21|^L18", HW)
F("piper_pa20_22", "Piper PA-20 Pacer / PA-22 Tri-Pacer & Colt", "sep_fixed", ["piper"],
  r"^PA2[02][A-Z0-9]*$|^TRIPACER|^PACER|^COLT$", HW,
  [V("pa20", "PA-20 Pacer", r"^PA20|^PACER", cls="sep_tailwheel", gear_config="tailwheel"),
   V("pa22_108", "PA-22-108 Colt", r"^PA22108|^COLT"),
   V("pa22", "PA-22 Tri-Pacer", r"^PA22|^TRIPACER")])
F("piper_pa23", "Piper PA-23 Apache / Aztec", "mep", ["piper"], r"^PA23[A-Z0-9]*$|^AZTEC|^APACHE", LW,
  [V("aztec", "PA-23-250 Aztec", r"^PA23250|^AZTEC"), V("apache", "PA-23 Apache (150/160/235)", r"^PA23|^APACHE")])
F("piper_pa24", "Piper PA-24 Comanche", "sep_retract", ["piper"], r"^PA24[A-Z0-9]*$|^COMANCHE$|^COMANCHE[0-9]", LW)
F("piper_pa25", "Piper PA-25 Pawnee", "agricultural", ["piper"], r"^PA25|^PAWNEE$", dict(propulsion="piston", wing="low"))
F("piper_pa28", "Piper PA-28 Cherokee family (Cherokee, Warrior, Archer, Dakota, Arrow)", "sep_fixed", ["piper"],
  r"^PA28[A-Z0-9]*$|^CHEROKEE$|^CHEROKEE(140|150|160|180|235)|^WARRIOR|^ARCHER|^DAKOTA|^ARROW|^CRUISER$", LW,
  [V("arrow", "PA-28R / PA-28RT Arrow", r"^PA28R|^ARROW", cls="sep_retract", gear="retract"),
   V("cherokee_140", "PA-28-140 Cherokee 140 / Cruiser", r"^PA28140|^CHEROKEE140|^CRUISER"),
   V("cherokee_150_180", "PA-28-150/160/180 Cherokee", r"^PA28(150|160|180)|^CHEROKEE(150|160|180)"),
   V("warrior", "PA-28-151/161 Warrior", r"^PA28(151|161)|^WARRIOR"),
   V("archer", "PA-28-181 Archer", r"^PA28181|^ARCHER"),
   V("dakota_235", "PA-28-235/236 / 201T Dakota & Pathfinder", r"^PA28(235|236|201T)|^DAKOTA|^CHEROKEE235")])
F("piper_pa30_39", "Piper PA-30/PA-39 Twin Comanche", "mep", ["piper"], r"^PA(30|39)[A-Z0-9]*$|^TWINCOMANCHE", LW)
F("piper_pa31", "Piper PA-31 Navajo / Chieftain / Cheyenne I-II", "mep", ["piper"],
  r"^PA31[A-Z0-9]*$|^NAVAJO|^CHIEFTAIN|^CHEYENNE(I|II|IIXL)?$|^MOJAVE", LW,
  [V("cheyenne", "PA-31T Cheyenne I/II/IIXL", r"^PA31T|^CHEYENNE", cls="met", propulsion="turboprop", pressurized=True),
   V("chieftain", "PA-31-350 Chieftain", r"^PA31350|^CHIEFTAIN"),
   V("mojave", "PA-31P-350 Mojave", r"^PA31P350|^MOJAVE", pressurized=True),
   V("navajo", "PA-31 / 31-310 / 31-325 / 31P Navajo", r"^PA31|^NAVAJO")])
F("piper_pa32", "Piper PA-32 Cherokee Six / Saratoga / Lance", "sep_fixed", ["piper"],
  r"^PA32[A-Z0-9]*$|^CHEROKEESIX|^SARATOGA|^LANCE", LW,
  [V("pa32r", "PA-32R Lance / Saratoga SP / Saratoga II", r"^PA32R|^LANCE|^SARATOGASP|^SARATOGAII",
     cls="sep_retract", gear="retract"),
   V("pa32_fixed", "PA-32-260/300/301 Cherokee Six / Saratoga / 6X", r"^PA32(260|300|301)|^CHEROKEESIX")])
F("piper_pa34", "Piper PA-34 Seneca", "mep", ["piper"], r"^PA34[A-Z0-9]*$|^SENECA|^EMB810", LW)
F("piper_pa36", "Piper PA-36 Pawnee Brave", "agricultural", ["piper"], r"^PA36|^BRAVE|^PAWNEEBRAVE",
  dict(propulsion="piston", wing="low"))
F("piper_pa38", "Piper PA-38 Tomahawk", "sep_fixed", ["piper"], r"^PA38|^TOMAHAWK", LW)
F("piper_pa42", "Piper PA-42 Cheyenne III / 400", "met", ["piper"], r"^PA42|^CHEYENNE(III|IIIA|400)", dict(wing="low", pressurized=True), priority=150)
F("piper_pa44", "Piper PA-44 Seminole", "mep", ["piper"], r"^PA44|^SEMINOLE", LW)
F("piper_pa46", "Piper PA-46 Malibu / Mirage / Matrix / Meridian / M-series", "sep_retract", ["piper"],
  r"^PA46[A-Z0-9]*$|^MALIBU|^MIRAGE|^MATRIX|^MERIDIAN|^M(350|500|600|700)[A-Z0-9]*$",
  dict(wing="low", pressurized=True),
  [V("pa46_turbine", "PA-46-500TP Meridian / M500 / M600 / M700", r"^PA46(500|600|701)|^MERIDIAN|^M(500|600|700)",
     cls="set", propulsion="turboprop"),
   V("pa46_piston", "PA-46-310P/350P Malibu / Mirage / Matrix / M350", r"^PA46(310|350)|^MALIBU|^MIRAGE|^MATRIX|^M350")],
  notes="JetPROP / Silver Eagle turbine conversions keep the piston model string; resolve with engines.eng_type.")
F("piper_aerostar", "Piper / Ted Smith Aerostar (PA-60, 600/601/602)", "mep", ["aerostar"],
  r"^PA60[A-Z0-9]*$|^60[012][A-Z0-9]*$|^AEROSTAR", LW)
F("piper_ps28", "Piper Sport / Czech SportCruiser (PS-28)", "lsa", ["czech_sport"], r"^PS28|^SPORTCRUISER|^PIPERSPORT", LW)

# ======== Beech
F("beech_bonanza", "Beech Bonanza / Debonair (33/35/36)", "sep_retract", ["beech"],
  r"^(BE)?(35)?[A-H]?33[A-Z]?$|^(BE)?[A-Z]?35[A-Z]?[0-9]*$|^(BE)?[A-G]?36[A-Z]*$|^BONANZA|^DEBONAIR", LW,
  [V("be33", "33 / F33 Debonair & Bonanza", r"^(BE)?(35)?[A-H]?33|^DEBONAIR"),
   V("be35", "35 V-tail Bonanza", r"^(BE)?[A-Z]?35"),
   V("be36", "36 / A36 / G36 Bonanza", r"^(BE)?[A-G]?36")])
F("beech_musketeer", "Beech Musketeer / Sport / Sundowner / Sierra (19/23/24)", "sep_fixed", ["beech"],
  r"^(BE)?[A-C]?(19|23|24)[A-Z0-9]*$|^MUSKETEER|^SUNDOWNER|^SIERRA", LW,
  [V("sierra", "24R / A24R / B24R Sierra", r"^(BE)?[A-C]?24R|^SIERRA", cls="sep_retract", gear="retract"),
   V("sundowner", "C23 Sundowner / 23 / A23 Musketeer", r"^(BE)?[A-C]?23|^SUNDOWNER|^MUSKETEER"),
   V("sport", "19 / B19 Sport", r"^(BE)?[A-C]?19")])
F("beech_77", "Beech 77 Skipper", "sep_fixed", ["beech"], r"^(BE)?77$|^SKIPPER", LW)
F("beech_76", "Beech 76 Duchess", "mep", ["beech"], r"^(BE)?76$|^DUCHESS", LW)
F("beech_95", "Beech 95 Travel Air", "mep", ["beech"], r"^(BE)?[A-E]?95[A-Z0-9]*$|^TRAVELAIR", LW)
F("beech_baron", "Beech Baron (55/56/58)", "mep", ["beech"],
  r"^(BE)?(95)?[A-E]?55[A-Z0-9]*$|^(BE)?56TC|^(BE)?58[A-Z0-9]*$|^BARON", LW,
  [V("be58", "58 / 58P / 58TC / G58", r"^(BE)?G?58"), V("be56", "56TC Turbo Baron", r"^(BE)?56"),
   V("be55", "55 / 95-55 Baron", r"55")], priority=150)
F("beech_60", "Beech 60 Duke", "mep", ["beech"], r"^(BE)?[AB]?60$|^DUKE", dict(wing="low", pressurized=True))
F("beech_50", "Beech 50 Twin Bonanza", "mep", ["beech"], r"^(BE)?[A-J]?50$|^TWINBONANZA", LW)
F("beech_queen_air", "Beech Queen Air (65/70/80/88)", "mep", ["beech"],
  r"^(BE)?(65)?[A-Z]?(65|70|80|88)[A-Z]?$|^QUEENAIR|^U8[DF]?$", LW)
F("beech_18", "Beech 18 Twin Beech", "mep", ["beech"], r"^(BE)?[A-H]?18[A-Z]?[0-9]*[A-Z]?$|^C45|^TWINBEECH|^SNB",
  dict(wing="low", gear_config="tailwheel", engine="radial"))
F("beech_king_air_90_100", "Beech King Air 90/100", "met", ["beech"],
  r"^(BE)?(65)?[A-Z]?90[A-Z0-9]*$|^(BE)?[A-Z]?100$|^KINGAIR(90|100)|^C90|^U21", LW,
  [V("ka100", "A100 / B100", r"100$|KINGAIR100"), V("ka90", "C90 / E90 / F90 / 65-90", r"90|^U21")], priority=150)
F("beech_king_air_200_300", "Beech King Air 200/250/300/350", "met", ["beech"],
  r"^(BE)?[A-Z]?(200|300|350)[A-Z0-9]*$|^KINGAIR(200|250|260|300|350|360)|^SUPERKINGAIR|^C12[A-Z]?$|^(BE)?(250|260|360)$",
  dict(wing="low", pressurized=True),
  [V("ka300_350", "300 / B300 / 350 / 360", r"^(BE)?[A-Z]?3[056]0|KINGAIR3"),
   V("ka200_250", "200 / B200 / 250 / 260 / C-12", r"^(BE)?[A-Z]?2[056]0|KINGAIR2|^C12|SUPERKINGAIR")],
  priority=140)
F("beech_99", "Beech 99 Airliner", "regional_turboprop", ["beech"], r"^(BE)?[A-C]?99[A-Z0-9]*$", LW)
F("beech_1900", "Beech 1900 Airliner", "regional_turboprop", ["beech"], r"^(BE)?1900[A-Z0-9]*$|^C12J", LW, priority=150)
F("beech_2000", "Beech 2000 Starship", "met", ["beech"], r"^2000A?$|^STARSHIP", dict(wing="mid", layout="canard_pusher"), priority=200)
F("beech_t34", "Beech T-34 Mentor", "vintage", ["beech"], r"^(BE)?[AB]?45$|^T34[A-C]?$|^MENTOR",
  dict(engines=1, propulsion="piston", gear="retract", gear_config="tricycle", wing="low"))
F("beechjet_400", "Beechjet / Hawker 400 / Diamond (MU-300, 400, 400A, 400XP)", "bizjet_light", ["beechjet"],
  r"^(BE)?400[A-Z]*$|^MU300|^BEECHJET|^HAWKER400|^T1A$|^DIAMOND(I|II)?$", LW)
F("beech_premier", "Beech Premier I/IA (390)", "bizjet_light", ["beechjet"], r"^390$|^PREMIER", LW)

# ======== Other GA singles
F("mooney_m20", "Mooney M20 series", "sep_retract", ["mooney"],
  r"^M20[A-Z0-9]*$|^OVATION|^ACCLAIM|^MOONEY(201|231|252)|^EXECUTIVE|^RANGER|^ENCORE|^EAGLE$", LW,
  [V("m20a_e", "M20/M20A-M20E (early)", r"^M20[A-E]?$"),
   V("m20f_g", "M20F Executive / M20G Statesman", r"^M20[FG]|^EXECUTIVE"),
   V("m20j", "M20J 201 / MSE / Allegro", r"^M20J|^MOONEY201"),
   V("m20k", "M20K 231/252/Encore", r"^M20K|^MOONEY2[35]|^ENCORE"),
   V("m20m", "M20M Bravo / TLS", r"^M20M"),
   V("m20r", "M20R Ovation", r"^M20R|^OVATION"),
   V("m20s", "M20S Eagle", r"^M20S|^EAGLE"),
   V("m20tn", "M20TN Acclaim", r"^M20TN|^ACCLAIM"),
   V("m20u_v", "M20U Ovation Ultra / M20V Acclaim Ultra", r"^M20[UV]")])
F("cirrus_sr", "Cirrus SR20 / SR22 / SR22T", "sep_fixed", ["cirrus"], r"^SR2[02][A-Z0-9]*$", dict(wing="low", parachute=True),
  [V("sr22t", "SR22T", r"^SR22T"), V("sr22", "SR22", r"^SR22"), V("sr20", "SR20", r"^SR20")])
F("cirrus_sf50", "Cirrus SF50 Vision Jet", "bizjet_light", ["cirrus"], r"^SF50|^VISION", dict(engines=1, wing="low", parachute=True))
F("diamond_da20", "Diamond DA20 / DV20 Katana", "sep_fixed", ["diamond"], r"^D[AV]20[A-Z0-9]*$|^KATANA|^ECLIPSE$", LW)
F("diamond_da40", "Diamond DA40 Diamond Star", "sep_fixed", ["diamond"], r"^DA40[A-Z0-9]*$|^DIAMONDSTAR|^STAR$", LW)
F("diamond_da42", "Diamond DA42 Twin Star", "mep", ["diamond"], r"^DA42[A-Z0-9]*$|^TWINSTAR", dict(wing="low", fuel="jet_a_diesel"))
F("diamond_da62", "Diamond DA62", "mep", ["diamond"], r"^DA62", dict(wing="low", fuel="jet_a_diesel"))
F("grumman_aa1", "Grumman American AA-1 Yankee / Trainer / T-Cat / Lynx", "sep_fixed", ["grumman_american"],
  r"^AA1[A-Z0-9]*$|^YANKEE|^TCAT|^LYNX|^TR2$", LW)
F("grumman_aa5", "Grumman American AA-5 Traveler / Cheetah / Tiger", "sep_fixed", ["grumman_american"],
  r"^A[AG]5[A-Z0-9]*$|^TRAVELER|^CHEETAH|^TIGER$", LW,
  [V("aa5b", "AA-5B / AG-5B Tiger", r"^A[AG]5B|^TIGER"), V("aa5a", "AA-5A Cheetah", r"^AA5A|^CHEETAH"),
   V("aa5", "AA-5 Traveler", r"^AA5|^TRAVELER")])
F("grumman_ga7", "Grumman American GA-7 Cougar", "mep", ["grumman_american"], r"^GA7|^COUGAR", LW)
F("socata_tb", "Socata TB Tampico / Tobago / Trinidad", "sep_fixed", ["socata"],
  r"^TB(9|10|20|21|200)[A-Z0-9]*$|^TOBAGO|^TRINIDAD|^TAMPICO", LW,
  [V("tb20_21", "TB20 / TB21 Trinidad", r"^TB2[01]|^TRINIDAD", cls="sep_retract", gear="retract"),
   V("tb9_10_200", "TB9 / TB10 / TB200", r"^TB(9|10|200)|^TOBAGO|^TAMPICO")])
F("commander_112_114", "Rockwell / Commander 112 / 114", "sep_retract", ["commander_single"],
  r"^1(12|14)[A-Z0-9]*$|^COMMANDER11[24]", LW)
F("navion", "Navion", "sep_retract", ["navion"], r"^NAVION|^L17[A-C]?$", LW)
F("ercoupe", "Ercoupe / Aircoupe / Alon / Mooney M10", "sep_fixed", ["erco"],
  r"^415[A-Z0-9]*$|^ERCOUPE|^A2A?$|^F1A?$|^M10$|^AIRCOUPE|^CADET", dict(wing="low", two_control=True), priority=150)
F("luscombe_8", "Luscombe 8 Silvaire", "sep_tailwheel", ["luscombe"], r"^T?8[A-F]?[A-Z0-9]*$|^SILVAIRE", HW)
F("aeronca_champ", "Aeronca / Champion 7AC Champ series", "sep_tailwheel", ["aeronca"],
  r"^7(AC|BCM|CCM|DC|EC|FC)[A-Z0-9]*$|^CHAMP$|^CHAMPION|^L16", HW,
  [V("7ac", "7AC Champ", r"^7AC|^CHAMP$"), V("7ec", "7EC/7FC Traveler / Tri-Traveler", r"^7[EF]C"),
   V("7bcm_dc", "7BCM / 7CCM / 7DC / L-16", r"^7(BCM|CCM|DC)|^L16")])
F("aeronca_chief", "Aeronca 11 Chief / 15 Sedan", "sep_tailwheel", ["aeronca"],
  r"^11(AC|BC|CC)[A-Z0-9]*$|^CHIEF|^SUPERCHIEF|^15AC|^SEDAN", HW)
F("champion_citabria", "Citabria (7ECA/7GCAA/7GCBC/7KCAB)", "sep_tailwheel", ["champion"],
  r"^7(ECA|GCA|GCAA|GCB|GCBA|GCBC|KC|KCAB)[A-Z0-9]*$|^CITABRIA|^AURORA|^ADVENTURE|^EXPLORER", HW, priority=150)
F("champion_decathlon", "Decathlon / Super Decathlon (8KCAB)", "sep_aerobatic", ["champion"],
  r"^8KCAB|^DECATHLON|^SUPERDECATHLON", HW)
F("champion_scout", "Scout (8GCBC)", "sep_tailwheel", ["champion"], r"^8GCBC|^SCOUT", HW)
F("bellanca_viking", "Bellanca 14-19 / 17-30 Viking (& 14-13 Cruisair)", "sep_retract", ["bellanca"],
  r"^1[47][0-9]{2,3}[A-Z]*$|^VIKING|^SUPERVIKING|^CRUISAIR|^CRUISEMASTER", LW,
  [V("cruisair", "14-12 / 14-13 Cruisair", r"^141[23]|^CRUISAIR", cls="sep_tailwheel", gear="fixed", gear_config="tailwheel"),
   V("viking", "17-30 / 17-31 Viking", r"^17|^VIKING|^SUPERVIKING"),
   V("cruisemaster", "14-19 Cruisemaster", r"^1419|^CRUISEMASTER")])
F("taylorcraft", "Taylorcraft (BC-12, F-19, F-21, L-2)", "sep_tailwheel", ["taylorcraft"], r".", HW, priority=50)
F("stinson", "Stinson 108 Voyager / L-5", "sep_tailwheel", ["stinson"], r".", HW,
  [V("s108", "108 Voyager / Station Wagon", r"^108|VOYAGER|STATIONWAGON"), V("l5", "L-5 Sentinel / V-77", r"^L5|^V77")])
F("maule", "Maule M-4 through M-9", "sep_tailwheel", ["maule"], r".", HW,
  [V("maule_tricycle", "MXT-7 / MT-7 (tricycle)", r"^M?X?T7|^MXT|^MT7", gear_config="tricycle"),
   V("maule_m4_m6", "M-4 / M-5 / M-6", r"^M[456]"),
   V("maule_m7_m9", "M-7 / MX-7 / M-8 / M-9", r"^MX?[789]")])
F("aviat_husky", "Aviat Husky (A-1)", "sep_tailwheel", ["aviat"], r"^A1[A-C]?[A-Z0-9]*$|^HUSKY", HW)
F("cubcrafters", "CubCrafters Carbon Cub / Top Cub / XCub / NX Cub", "sep_tailwheel", ["cubcrafters"], r".", HW,
  [V("cc11", "CC11 Carbon Cub (S-LSA / EX)", r"^CC11|^CARBONCUB|^EX", cert="lsa_or_experimental"),
   V("cc18", "CC18 Top Cub", r"^CC18|^TOPCUB|^PA18"),
   V("cc19", "CC19 XCub / NX Cub", r"^CC19|^XCUB|^NXCUB")],
  homebuilt_model_re=r"^CARBONCUB|^CCK", priority=90)
F("globe_swift", "Globe / Temco Swift", "sep_tailwheel", ["globe_temco"], r"^GC1[A-Z]?$|^SWIFT", dict(wing="low", gear="retract"),
  priority=150)
F("helio_courier", "Helio Courier / Super Courier", "sep_tailwheel", ["helio"], r"^H[0-9]{3}|^COURIER|^U10", HW)
F("gippsland_ga8", "Gippsland GA8 Airvan", "sep_fixed", ["gippsland"], r"^GA8|^AIRVAN", HW)

# ======== Aerobatic
F("pitts", "Pitts Special (S-1 / S-2)", "sep_aerobatic", ["aviat"], r"^(PITTS)?S[12][A-Z0-9]*$|^PITTS", dict(wing="biplane"),
  [V("s1", "S-1 (single seat)", r"^(PITTS)?S1"), V("s2", "S-2 (two seat)", r"^(PITTS)?S2")],
  homebuilt_model_re=r"^(PITTS)?S1(C|D|E|S|T|SS|11B)?$|^(PITTS)?S2[ABCES]?$|^PITTS")
F("christen_eagle", "Christen Eagle", "sep_aerobatic", ["christen"], r"^(CHRISTEN)?EAGLE(II|2)?$|^CHRISTEN",
  dict(wing="biplane"), homebuilt_model_re=r"^CHRISTEN|^EAGLEII$")
F("extra", "Extra 200/230/260/300/330", "sep_aerobatic", ["extra"], r".", LW,
  [V("ea300_330", "EA-300 / 330", r"(300|330)"), V("ea200_260", "EA-200 / 230 / 260", r"(200|230|260)")])
F("zivko_edge", "Zivko Edge 540", "sep_aerobatic", ["zivko"], r".", LW, homebuilt_model_re=r"^EDGE540|^EDGE")
F("sukhoi", "Sukhoi Su-26/29/31", "sep_aerobatic", ["sukhoi"], r".", LW)

# ======== LSA
F("flight_design_ct", "Flight Design CT series", "lsa", ["flight_design"], r".", HW)
F("tecnam", "Tecnam (P92 / P2002 / P2008 / P2010 / P2006T)", "lsa", ["tecnam"], r".", HW,
  [V("p2006t", "P2006T twin", r"^P2006", cls="mep", engines=2, gear="retract", cert="part23"),
   V("p2010", "P2010", r"^P2010", cls="sep_fixed", cert="part23"),
   V("p2002", "P2002 Sierra (low wing)", r"^P2002", wing="low"),
   V("p2008", "P2008", r"^P2008"), V("p92", "P92 Echo / Eaglet", r"^P92")])
F("icon_a5", "Icon A5", "lsa", ["icon"], r".", dict(wing="high", amphibian=True, gear="retract"))
F("sling", "Sling 2 / Sling 4 / TSi", "lsa", ["sling"], r".", LW,
  homebuilt_model_re=r"^SLING")
F("pipistrel", "Pipistrel (Alpha, Virus, Sinus, Velis)", "lsa", ["pipistrel"], r".", HW)
F("evektor_sportstar", "Evektor SportStar / Harmony", "lsa", ["evektor"], r".", LW)
F("remos", "Remos GX", "lsa", ["remos"], r".", HW)
F("jabiru", "Jabiru J170/J230/J250", "lsa", ["jabiru"], r".", HW, homebuilt_model_re=r"^JABIRU")
F("aeropro_eurofox", "AeroPro EuroFOX", "lsa", ["aeropro"], r".", HW)
F("american_legend_cub", "American Legend Cub", "lsa", ["american_legend"], r".", dict(wing="high", gear_config="tailwheel"))

# ======== Experimental amateur-built (kits; builders' names often appear as make)
RV_VARIANTS = [V("rv10", "RV-10", r"^(VANS)?RV10", gear_config="tricycle"),
               V("rv12", "RV-12 / 12iS", r"^(VANS)?RV12", gear_config="tricycle", cert="lsa_or_experimental"),
               V("rv14a", "RV-14A", r"^(VANS)?RV14A", gear_config="tricycle"),
               V("rv14", "RV-14", r"^(VANS)?RV14", gear_config="tailwheel"),
               V("rv15", "RV-15", r"^(VANS)?RV15", gear_config="tailwheel")]
for n in ["3", "4", "6", "7", "8", "9"]:
    if n in ("6", "7", "8", "9"):
        RV_VARIANTS.append(V(f"rv{n}a", f"RV-{n}A", rf"^(VANS)?RV{n}A", gear_config="tricycle"))
    RV_VARIANTS.append(V(f"rv{n}", f"RV-{n}", rf"^(VANS)?RV{n}", gear_config="tailwheel"))
F("vans_rv", "Van's RV series", "experimental", ["vans"], r"^(VANS)?RV[0-9]{1,2}[A-Z0-9]*$", LW, RV_VARIANTS,
  homebuilt_model_re=r"^(VANS)?RV[0-9]{1,2}[A-Z0-9]*$")
F("lancair", "Lancair (235/320/360/IV/ES/Legacy/Evolution)", "experimental", ["lancair"],
  r"^LANCAIR|^LEGACY(2000|FG|RG)?$|^EVOLUTION$|^(235|320|360)(MKII)?$|^(IV|IVP|4P|ES|ESP)$", dict(wing="low", gear="retract"),
  [V("lancair_iv", "IV / IV-P / Propjet", r"IVP?$|4P|PROPJET", pressurized=True),
   V("lancair_es", "ES / ES-P (fixed gear)", r"ESP?$", gear="fixed"),
   V("lancair_legacy", "Legacy", r"LEGACY"), V("lancair_evolution", "Evolution", r"EVOLUTION"),
   V("lancair_2_3", "235 / 320 / 360", r"(235|320|360)")],
  homebuilt_model_re=r"^LANCAIR|^LEGACY(2000|FG|RG)?$|^EVOLUTION$|^(320|360)(MKII)?$")
F("glasair", "Glasair / GlaStar / Sportsman", "experimental", ["glasair"],
  r"^GLASAIR|^GLASTAR|^SPORTSMAN|^GS[12]|^(I|II|III)[A-Z]*$|^SUPERII", LW,
  [V("glastar_sportsman", "GlaStar / Sportsman 2+2", r"GLASTAR|SPORTSMAN|^GS1|^GS2", wing="high", gear="fixed"),
   V("glasair_iii", "Glasair III", r"III", gear="retract"),
   V("glasair_i_ii", "Glasair I / II / Super II", r"(I|II|SUPERII)")],
  homebuilt_model_re=r"^GLASAIR|^GLASTAR|^SPORTSMAN(22)?$")
F("kitfox", "Kitfox", "experimental", ["kitfox"], r".", dict(wing="high", gear_config="tailwheel"),
  homebuilt_model_re=r"^KITFOX|^KF[0-9]")
F("avid_flyer", "Avid Flyer / Magnum", "experimental", ["avid"], r".", dict(wing="high", gear_config="tailwheel"),
  homebuilt_model_re=r"^AVID")
F("zenith", "Zenith / Zenair CH-series (Zodiac, STOL)", "experimental", ["zenith"], r".", dict(gear_config="tricycle"),
  [V("zenith_stol", "CH-701 / CH-750 / CH-801 STOL", r"STOL|7[05]0|701|801", wing="high"),
   V("zenith_zodiac", "CH-601 / CH-640 / CH-650 Zodiac", r"ZODIAC|6(01|40|50)", wing="low"),
   V("zenith_200_300", "CH-200 / CH-250 / CH-300", r"(200|250|300)", wing="low")],
  homebuilt_model_re=r"^(ZENITH|ZENAIR)|^ZODIAC|^CH[0-9]{3}|^STOL(CH)?[0-9]{3}")
F("sonex", "Sonex / Onex / Waiex / Xenos", "experimental", ["sonex"], r".", LW,
  homebuilt_model_re=r"^SONEX|^ONEX|^WAIEX|^XENOS|^SUBSONEX")
F("rans", "Rans S-series (Coyote, Courier, Airaile, S-20)", "experimental", ["rans"], r".", HW,
  homebuilt_model_re=r"^RANS|^COYOTE|^AIRAILE|^S(4|5|6|7|9|10|12|14|19|20|21)(ES|S|XL|LS|C|COYOTE|COURIER|AIRAILE)[A-Z0-9]*$")
F("canard_pusher", "Rutan-type canards (Long-EZ, VariEze, Cozy, Velocity, Berkut)", "experimental", ["canard"],
  r"^LONGEZ|^VARIEZE|^COZY|^VELOCITY|^BERKUT|^DEFIANT|^EZ$|^VARIVIGGEN", dict(wing="canard_pusher"),
  [V("velocity", "Velocity", r"VELOCITY"), V("cozy", "Cozy", r"COZY"), V("longez", "Long-EZ", r"LONGEZ|^EZ$"),
   V("varieze", "VariEze", r"VARIEZE"), V("berkut", "Berkut", r"BERKUT"), V("defiant", "Defiant (twin)", r"DEFIANT", engines=2)],
  homebuilt_model_re=r"^LONGEZ|^VARIEZE|^COZY|^VELOCITY|^BERKUT|^DEFIANT|^VARIVIGGEN")
F("murphy", "Murphy Rebel / Moose / Elite", "experimental", ["murphy"], r".", HW,
  homebuilt_model_re=r"^MURPHY|^REBEL$|^MOOSE")
F("just_highlander", "Just Aircraft Highlander / SuperSTOL", "experimental", ["just"], r".", dict(wing="high", gear_config="tailwheel"),
  homebuilt_model_re=r"^HIGHLANDER|^SUPERSTOL|^JUST")
F("bearhawk", "Bearhawk", "experimental", [], r"^BEARHAWK", dict(wing="high", gear_config="tailwheel"),
  homebuilt_model_re=r"^BEARHAWK")
F("pietenpol", "Pietenpol Air Camper", "experimental", [], r"^PIETENPOL|^AIRCAMPER",
  dict(wing="parasol", gear_config="tailwheel"), homebuilt_model_re=r"^PIETENPOL|^AIRCAMPER")
F("kolb", "Kolb Firestar / Twinstar / Mark III", "experimental", ["kolb"], r".", HW,
  homebuilt_model_re=r"^KOLB|^FIRESTAR|^FIREFLY")

# ======== Agricultural
F("air_tractor", "Air Tractor AT-300 through AT-800 series", "agricultural", ["air_tractor"], r".",
  dict(propulsion="turboprop", wing="low"),
  [V("at300_400p", "AT-301 / AT-401 (radial piston)", r"^AT(30[0-9]|401)", propulsion="piston"),
   V("at802", "AT-802 / 802A / Fire Boss", r"^AT8"), V("at602", "AT-602", r"^AT6"),
   V("at50x", "AT-502 / 503 / 504", r"^AT50"), V("at40x", "AT-402", r"^AT40")])
F("thrush", "Thrush / Ayres / Rockwell S2R", "agricultural", ["thrush"], r"^S2R[A-Z0-9]*$|^THRUSH|^(510|710)[A-Z]*$",
  dict(wing="low"),
  [V("thrush_turbine", "S2R-T / S2R-G / S2R-H / 510 / 710 (turbine)", r"^S2R(T|G|H|HG)|^S2R[0-9]+T|^(510|710)", propulsion="turboprop"),
   V("thrush_piston", "S2R / S2R-R1340 / R1820 (radial)", r"^S2R", propulsion="piston")])
F("grumman_agcat", "Grumman / Schweizer G-164 Ag Cat", "agricultural", ["agcat"], r"^G164[A-Z0-9]*$|^AGCAT|^SUPERAGCAT|^164[A-Z0-9]*$",
  dict(wing="biplane", propulsion="piston"), priority=150)
F("weatherly", "Weatherly 201/620", "agricultural", ["weatherly"], r".", dict(wing="low", propulsion="piston"))
F("pzl_dromader", "PZL M-18 Dromader", "agricultural", ["pzl"], r"^M18|^DROMADER", dict(wing="low", propulsion="piston"))

# ======== Amphibians
F("lake", "Lake LA-4 Buccaneer / Renegade / Seawolf", "amphibian", ["lake"], r".",
  dict(engines=1, propulsion="piston", gear="retract", engine_layout="pylon_pusher"),
  [V("la250", "LA-250 Renegade / Seawolf", r"^LA250|RENEGADE|SEAWOLF|^250"),
   V("la4", "LA-4 / LA-4-200 Buccaneer", r"^LA4|BUCCANEER")])
F("grumman_amphibians", "Grumman Goose / Widgeon / Mallard / Albatross", "amphibian", ["grumman_amph"],
  r"^G(21|44|73)[A-Z0-9]*$|^G111[A-Z]?$|^GOOSE|^WIDGEON|^MALLARD|^ALBATROSS|^HU16|^SCAN30", dict(engines=2, propulsion="piston", gear="retract"),
  [V("g21", "G-21 Goose", r"^G21|GOOSE"), V("g44", "G-44 Widgeon", r"^G44|WIDGEON|SCAN30"),
   V("g73", "G-73 Mallard", r"^G73|MALLARD"), V("g111", "G-111 / HU-16 Albatross", r"^G111|^HU16|ALBATROSS")], priority=150)
F("republic_seabee", "Republic RC-3 Seabee", "amphibian", ["republic"], r"^RC3|^SEABEE", dict(engines=1, propulsion="piston"))
F("seawind", "Seawind 3000/300C", "amphibian", ["seawind"], r".", dict(engines=1, propulsion="piston", gear="retract"),
  homebuilt_model_re=r"^SEAWIND")
F("searey", "Progressive Aerodyne SeaRey", "amphibian", ["progressive"], r".", dict(engines=1, propulsion="piston", cert="experimental"),
  homebuilt_model_re=r"^SEAREY")

# ======== Multi-engine piston (other)
F("britten_norman_islander", "Britten-Norman BN-2 Islander / Trislander", "mep", ["britten_norman"], r"^BN2", dict(wing="high", gear="fixed"))
F("partenavia_p68", "Partenavia / Vulcanair P68", "mep", ["partenavia"], r"^P68|^AP68|^OBSERVER", dict(wing="high", gear="fixed"))
F("aero_commander_piston", "Aero Commander 500/520/560/680 (piston)", "mep", ["twin_commander"],
  r"^(AC)?(500|520|560)[A-Z0-9]*$|^(AC)?680(E|F|FL|FLP|P)?$|^SHRIKE|^GRANDCOMMANDER", HW)

# ======== Single-engine turboprop
F("dhc2_beaver", "de Havilland Canada DHC-2 Beaver", "sep_tailwheel", ["dehavilland"], r"^DHC2[A-Z0-9]*$|^BEAVER|^L20|^U6A?$", HW,
  [V("dhc2_turbo", "DHC-2 Mk III Turbo Beaver", r"^DHC2(MKIII|MK3|T|III)", cls="set", propulsion="turboprop"),
   V("dhc2", "DHC-2 Mk I Beaver", r"^DHC2|BEAVER|^L20|^U6")])
F("dhc3_otter", "de Havilland Canada DHC-3 Otter", "set", ["dehavilland"], r"^DHC3[A-Z0-9]*$|^OTTER$|^U1A?$",
  dict(wing="high", gear_config="tailwheel"),
  notes="Original is radial piston; most active US Otters are turbine conversions. Resolve propulsion from engines table.")
F("pilatus_pc6", "Pilatus PC-6 Porter", "set", ["pilatus"], r"^PC6[A-Z0-9]*$|^PORTER", dict(wing="high", gear="fixed", gear_config="tailwheel"))
F("pilatus_pc12", "Pilatus PC-12", "set", ["pilatus"], r"^PC12[A-Z0-9]*$", dict(wing="low", gear="retract", pressurized=True),
  [V("pc12_ngx", "PC-12/47E (NG / NGX)", r"^PC1247E|NG"), V("pc12_legacy", "PC-12 / PC-12/45 / PC-12/47", r"^PC12")])
F("daher_tbm", "Socata / Daher TBM 700-960", "set", ["socata"], r"^TBM[0-9]*[A-Z0-9]*$", dict(wing="low", gear="retract", pressurized=True),
  [V("tbm700", "TBM 700", r"^TBM700"), V("tbm850", "TBM 850", r"^TBM850"), V("tbm9xx", "TBM 900-960", r"^TBM9")], priority=150)
F("quest_kodiak", "Quest / Daher Kodiak 100/900", "set", ["kodiak"], r"^KODIAK|^(K|KODIAK)?(100|900)[A-Z0-9]*$",
  dict(wing="high", gear="fixed", gear_config="tricycle"))
F("epic", "Epic LT / E1000", "set", ["epic"], r".", dict(wing="low", gear="retract", pressurized=True),
  homebuilt_model_re=r"^EPICLT|^LT$")
F("pac_750xl", "PAC 750XL", "set", ["pac"], r"^750XL|^P750", dict(wing="low", gear="fixed"))

# ======== Multi-engine turboprop (business / utility)
F("turbo_commander", "Turbo Commander / Jetprop (680T/V/W, 681, 690, 695)", "met", ["twin_commander"],
  r"^(AC)?68(0T|0V|0W|1)[A-Z0-9]*$|^(AC)?69[05][A-Z0-9]*$|^JETPROP|^TURBOCOMMANDER", dict(wing="high", pressurized=True))
F("mitsubishi_mu2", "Mitsubishi MU-2", "met", ["mitsubishi"], r"^MU2[A-Z0-9]*$|^SOLITAIRE|^MARQUISE", dict(wing="high", pressurized=True),
  [V("mu2_long", "Long body (MU-2B-36/60, Marquise)", r"^MU2B(36|60)|MARQUISE"),
   V("mu2_short", "Short body (MU-2B-25/35/40, Solitaire)", r"^MU2")])
F("piaggio_p180", "Piaggio P.180 Avanti", "met", ["piaggio"], r"^P180|^AVANTI", dict(wing="mid", layout="three_surface_pusher", pressurized=True))
F("swearingen_merlin", "Swearingen / Fairchild Merlin", "met", ["swearingen"],
  r"^SA26[A-Z0-9]*$|^SA226(T|TB|AT)$|^SA227(TT|AT)$|^MERLIN", dict(wing="low", pressurized=True), priority=150)

# ======== Regional / commuter turboprop
F("swearingen_metro", "Swearingen / Fairchild Metro (SA226-TC / SA227)", "regional_turboprop", ["swearingen"],
  r"^SA22[67][A-Z0-9]*$|^METRO|^C26[AB]?$", dict(wing="low", pressurized=True),
  [V("metro_ii", "SA226-TC Metro II", r"^SA226"), V("metro_iii_23", "SA227 Metro III / 23", r"^SA227|^METRO|^C26")])
F("dhc6_twin_otter", "de Havilland Canada DHC-6 Twin Otter", "regional_turboprop", ["dehavilland"], r"^DHC6[A-Z0-9]*$|^TWINOTTER|^UV18",
  dict(wing="high", gear="fixed"))
F("dhc7", "de Havilland Canada DHC-7 Dash 7", "regional_turboprop", ["dehavilland"], r"^DHC7", dict(wing="high", engines=4))
F("dhc8", "de Havilland Canada / Bombardier DHC-8 Dash 8 / Q400", "regional_turboprop", ["dehavilland"],
  r"^DHC8[A-Z0-9]*$|^DASH8|^Q[1234]00", HW,
  [V("dhc8_400", "DHC-8-400 / Q400", r"^DHC84|Q400|DASH8400"), V("dhc8_300", "DHC-8-300", r"^DHC83|Q300"),
   V("dhc8_100_200", "DHC-8-100 / 200", r"^DHC8[12]|Q[12]00")])
F("jetstream", "BAe / Handley Page Jetstream 31/32/41", "regional_turboprop", ["jetstream"],
  r"JETSTREAM|^(BA|BAE|HP)?(J|JS)?(31|32|41)[0-9]{2}[A-Z]*$|^HP137|^J(31|32|41)$", dict(wing="low", pressurized=True),
  [V("j41", "Jetstream 41", r"41[0-9]{2}|JETSTREAM41|^J41"), V("j31_32", "Jetstream 31 / 32", r"3[12]|JETSTREAM|HP137")])
F("saab_340_2000", "Saab 340 / 2000", "regional_turboprop", ["saab"], r"^(SF)?340[A-Z0-9]*$|^2000$", LW,
  [V("saab340", "340A / 340B", r"340"), V("saab2000", "2000", r"^2000")])
F("embraer_110", "Embraer EMB-110 Bandeirante", "regional_turboprop", ["embraer"], r"^EMB110|^110P|^BANDEIRANTE", LW)
F("embraer_120", "Embraer EMB-120 Brasilia", "regional_turboprop", ["embraer"], r"^EMB120|^120(ER|RT|FC)?$|^BRASILIA", dict(wing="low", pressurized=True))
F("atr", "ATR 42 / 72", "regional_turboprop", ["atr"], r"^ATR(42|72)|^(42|72)[0-9]*$|^ATR$", HW,
  [V("atr72", "ATR 72", r"72"), V("atr42", "ATR 42", r"42")])
F("dornier_228_328", "Dornier 228 / 328 / 328JET", "regional_turboprop", ["dornier"], r"^(DO)?2[23]8[A-Z0-9]*$|^(DO)?328", HW,
  [V("do328jet", "328-300 (328JET)", r"328(300|JET)", cls="regional_jet", propulsion="turbofan"),
   V("do328", "328-100", r"328"), V("do228", "228", r"228")])
F("shorts_330_360", "Shorts 330 / 360 / Skyvan / C-23 Sherpa", "regional_turboprop", ["shorts"],
  r"^(SD)?3[36]0[A-Z0-9]*$|^SC7|^SKYVAN|^SHERPA|^C23", dict(wing="high"),
  [V("sd360", "360", r"360"), V("sd330", "330 / Sherpa", r"330|SHERPA|^C23"), V("skyvan", "SC.7 Skyvan", r"SC7|SKYVAN", gear="fixed")])
F("casa_212_235", "CASA C-212 / CN-235 / C-295", "regional_turboprop", ["casa"], r"^C?212|^(CN|C)?235$|^CN235|^C295", HW, priority=90)
F("let_410", "Let L-410 Turbolet", "regional_turboprop", ["let"], r"^L410", HW)
F("fokker_f27_50", "Fokker F27 / Fairchild FH-227 / Fokker 50", "regional_turboprop", ["fokker"], r"^F27|^FH227|^F50$|^50$", HW)
F("gulfstream_i", "Grumman Gulfstream I (G-159)", "regional_turboprop", ["gulfstream"], r"^G159|^GULFSTREAMI$", LW)
F("convair_580", "Convair 580 / 600 / 640 / 5800 (turboprop)", "legacy_transport", ["convair"],
  r"^(CV|CONVAIR)?(580|600|640|5800)[A-Z0-9]*$", dict(engines=2, propulsion="turboprop", wing="low"), priority=150)

# ======== Business jets
F("eclipse", "Eclipse 500 / 550", "bizjet_light", ["eclipse"], r".", LW)
F("hondajet", "Honda HA-420 HondaJet", "bizjet_light", ["honda"], r".", dict(wing="low", engine_layout="over_wing"))
F("embraer_phenom", "Embraer Phenom 100 / 300 (EMB-500/505)", "bizjet_light", ["embraer"], r"^EMB50[05]|^PHENOM|^50[05][A-Z]*$", LW,
  [V("phenom300", "Phenom 300 (EMB-505)", r"505|PHENOM300"), V("phenom100", "Phenom 100 (EMB-500)", r"500|PHENOM100")])
F("embraer_legacy_praetor", "Embraer Legacy 450/500 / Praetor 500/600", "bizjet_midsize", ["embraer"],
  r"^EMB5(45|50)|^LEGACY(450|500)|^PRAETOR", LW)
F("embraer_legacy_600", "Embraer Legacy 600/650 (EMB-135BJ)", "bizjet_large", ["embraer"], r"^EMB135BJ|^(ERJ)?135BJ|^LEGACY(600|650)", LW,
  priority=160)
F("learjet", "Learjet (23-75)", "bizjet_light", ["learjet"],
  r"^(LJ|LR|LEARJET|GATES)?(23|24|25|28|29|31|35|36|40|45|55|60|70|75)[A-Z0-9]*$|^LEARJET|^C21", LW,
  [V("lj20s", "23 / 24 / 25 / 28 / 29", r"(23|24|25|28|29)"),
   V("lj30s", "31 / 35 / 36 / C-21", r"(31|35|36)|^C21"),
   V("lj55_60", "55 / 60", r"(55|60)", cls="bizjet_midsize"),
   V("lj40_45_70_75", "40 / 45 / 70 / 75", r"(40|45|70|75)")])
F("hawker_125", "Hawker Siddeley / BAe / Hawker 125 (700/800/850/900)", "bizjet_midsize", ["hawker"],
  r"^(HS|BAE|BH|DH)?125[A-Z0-9]*$|^HAWKER(600|700|750|800|850|900|1000)|^(600|700|750|800|850|900|1000)(A|B|XP|XPI|XPR|XP2)?$|^U125|^C29",
  LW, priority=160)
F("hawker_4000", "Hawker 4000 Horizon", "bizjet_midsize", ["hawker"], r"^4000$|^HAWKER4000|^HORIZON", LW)
F("dassault_falcon", "Dassault Falcon", "bizjet_large", ["dassault"],
  r"^(FALCON|FANJETFALCON|MYSTERE|MYSTEREFALCON|DA)?(10|20|50|100|200|900|2000|7X|8X|6X)[A-Z0-9]*$|^FALCON|^MYSTERE|^HU25|^FANJETFALCON", LW,
  [V("f2000", "Falcon 2000 series", r"2000", engines=2), V("f900", "Falcon 900 series", r"900", engines=3),
   V("f7x_8x", "Falcon 7X / 8X", r"[78]X", engines=3), V("f6x", "Falcon 6X", r"6X", engines=2),
   V("f200", "Falcon 200", r"200", cls="bizjet_midsize"),
   V("f10_100", "Falcon 10 / 100", r"(10|100)", cls="bizjet_midsize"),
   V("f50", "Falcon 50 series", r"50", engines=3),
   V("f20", "Fan Jet Falcon / Falcon 20 / HU-25", r"20|HU25|FANJET", cls="bizjet_midsize")])
F("gulfstream", "Gulfstream jets (GII-G800, G100-G280)", "bizjet_large", ["gulfstream"],
  r"^G?1159[A-Z0-9]*$|^(GULFSTREAM)?G(II|IIB|III|IV|IVSP|IVX|V|VSP|VI|VII)[A-Z0-9]*$|^GULFSTREAM(II|IIB|III|IV|IVSP|V|VSP)$|^(GULFSTREAM)?G(100|150|200|280|300|350|400|450|500|550|600|650|650ER|700|800)$|^C(20|37)[A-Z]?$|^GALAXY|^112[56]|^ASTRA",
  LW,
  [V("g1159_gii_giii", "G-1159 / GII / GIIB / GIII (C-20)", r"^G?1159[AB]?$|^(GULFSTREAM)?G?(II|IIB|III)$|^C20"),
   V("giv_g450", "GIV / GIV-SP / G300-G450", r"^G?1159C|^(GULFSTREAM)?G?IV|^(GULFSTREAM)?G(300|350|400|450)$"),
   V("gv_g550", "GV / GV-SP / G550 (C-37)", r"^(GULFSTREAM)?G?V$|^(GULFSTREAM)?G?VSP|^(GULFSTREAM)?G550$|^C37"),
   V("gvi_g650", "GVI / G650 / G650ER", r"^(GULFSTREAM)?G?VI$|^(GULFSTREAM)?G650"),
   V("gvii", "GVII G500 / G600 / G700 / G800", r"^(GULFSTREAM)?G?VII|^(GULFSTREAM)?G(600|700|800)$"),
   V("g280_g200", "G200 Galaxy / G280 (IAI 1126)", r"^(GULFSTREAM)?G(200|280)$|^GALAXY|^1126", cls="bizjet_midsize"),
   V("g100_g150", "Astra / G100 / G150 (IAI 1125)", r"^(GULFSTREAM)?G(100|150)$|^1125|^ASTRA", cls="bizjet_midsize")],
  notes="Bare 'G500' is ambiguous (GV-SP-era G500 vs GVII-G500) -> family-only.")
F("iai_westwind", "IAI 1121/1123/1124 Jet Commander / Westwind", "bizjet_midsize", ["iai", "twin_commander"],
  r"^112[134][A-Z0-9]*$|^WESTWIND|^JETCOMMANDER", LW, priority=150)
F("bombardier_challenger", "Canadair / Bombardier Challenger & Global", "bizjet_large", ["bombardier"],
  r"^CL600(1A11|2A12|2B16)[A-Z0-9]*$|^CL60[014]$|^CHALLENGER|^BD100|^BD700|^GLOBAL", LW,
  [V("global", "BD-700 Global Express / 5000-8000", r"^BD700|GLOBAL"),
   V("c300_350", "BD-100 Challenger 300 / 350 / 3500", r"^BD100|CHALLENGER3", cls="bizjet_midsize"),
   V("c604_650", "CL-600-2B16 Challenger 604 / 605 / 650", r"2B16|^CL604|CHALLENGER6(04|05|50)"),
   V("c601", "CL-600-2A12 Challenger 601", r"2A12|^CL601|CHALLENGER601"),
   V("c600", "CL-600-1A11 Challenger 600", r"1A11|^CL600$|CHALLENGER600")], priority=150)
F("sabreliner", "North American / Rockwell Sabreliner", "bizjet_midsize", ["sabreliner"], r"^(NA)?265[A-Z0-9]*$|^SABRE|^T39", LW)
F("lockheed_jetstar", "Lockheed JetStar (1329)", "bizjet_large", ["lockheed"], r"^(L)?1329|^JETSTAR", dict(wing="low", engines=4))
F("pilatus_pc24", "Pilatus PC-24", "bizjet_light", ["pilatus"], r"^PC24", LW)

# ======== Regional jets
F("bombardier_crj", "Bombardier CRJ100/200/440/700/900/1000", "regional_jet", ["bombardier"],
  r"^CL6002(B19|C10|C11|D15|D24|E25)[A-Z0-9]*$|^CRJ|^RJ(100|200|440|550|700|705|900|1000)|^CL65", LW,
  [V("crj1000", "CRJ1000 (CL-600-2E25)", r"2E25|CRJ1000|^RJ1000"),
   V("crj900", "CRJ900 (CL-600-2D24/2D15)", r"2D24|2D15|CRJ9|^RJ9"),
   V("crj700", "CRJ700/550 (CL-600-2C10/2C11)", r"2C1[01]|CRJ(7|55)|^RJ(7|55)"),
   V("crj100_200", "CRJ100/200/440 (CL-600-2B19)", r"2B19|CRJ(1|2|44)|^RJ(1|2|44)|^CL65")], priority=150)
F("embraer_erj145", "Embraer ERJ-135 / 140 / 145", "regional_jet", ["embraer"],
  r"^(EMB|ERJ)?1(35|40|45)[A-Z0-9]*$", LW,
  [V("erj135", "ERJ-135", r"135"), V("erj140", "ERJ-140", r"140"), V("erj145", "ERJ-145", r"145")])
F("embraer_ejet", "Embraer E-Jets (E170/175/190/195, E2)", "regional_jet", ["embraer"],
  r"^(EMB|ERJ|E)?1(70|75|90|95)[A-Z0-9]*$", LW,
  [V("ejet_e2", "E175-E2 / E190-E2 / E195-E2", r"E2$|^(ERJ|EMB)?190(300|400)|E2[A-Z]*$"),
   V("e175", "E175 (ERJ 170-200)", r"^(ERJ|EMB)?170200|^(ERJ|EMB|E)?175"),
   V("e170", "E170 (ERJ 170-100)", r"^(ERJ|EMB|E)?170"),
   V("e195", "E195 (ERJ 190-200)", r"^(ERJ|EMB)?190200|^(ERJ|EMB|E)?195"),
   V("e190", "E190 (ERJ 190-100)", r"^(ERJ|EMB|E)?190")])
F("bae146_avro", "BAe 146 / Avro RJ", "regional_jet", ["bae146"], r"^(BAE)?146[A-Z0-9]*$|^AVRORJ|^RJ(70|85|100)$|^AVRO146", dict(wing="high", engines=4))
F("fokker_f28_70_100", "Fokker F28 / 70 / 100", "regional_jet", ["fokker"], r"^F28[A-Z0-9]*$|^F(70|100)$|^(FOKKER)?(70|100)$", LW,
  [V("f100", "Fokker 100", r"0100|100$"), V("f70", "Fokker 70", r"0070|70$"), V("f28", "F28 Fellowship", r"^F28")])

# ======== Narrowbody
F("boeing_707_720", "Boeing 707 / 720", "narrowbody", ["boeing"], r"^7(07|20)[A-Z0-9]*$", dict(wing="low", engines=4))
F("boeing_717", "Boeing 717 (MD-95)", "narrowbody", ["douglas"], r"^717[A-Z0-9]*$|^MD95", dict(wing="low", engines=2), priority=150)
F("boeing_727", "Boeing 727", "narrowbody", ["boeing"], r"^727[A-Z0-9]*$", dict(wing="low", engines=3),
  [V("b727_200", "727-200", r"^7272"), V("b727_100", "727-100", r"^7271")])
F("boeing_737", "Boeing 737", "narrowbody", ["boeing"], r"^737[A-Z0-9]*$|^C40[AB]?$|^P8A?$|^T43", dict(wing="low", engines=2),
  [V("b737_max", "737 MAX 7/8/9/10", r"^737(MAX)?(7|8|9|10)$|MAX|^7378200|^737(7|8|9|10)MAX"),
   V("b737_ng", "737 NG (-600/700/800/900/BBJ)", r"^737[6789][0-9A-Z]{2}|^737BBJ|^C40|^P8"),
   V("b737_classic", "737 Classic (-300/400/500)", r"^737[345][0-9A-Z]{2}"),
   V("b737_original", "737 Original (-100/200, T-43)", r"^737[12][0-9A-Z]{2}|^T43")])
F("boeing_757", "Boeing 757", "narrowbody", ["boeing"], r"^757[A-Z0-9]*$|^C32", dict(wing="low", engines=2),
  [V("b757_300", "757-300", r"^7573"), V("b757_200", "757-200", r"^7572|^C32")])
F("douglas_dc8", "Douglas DC-8", "narrowbody", ["douglas"], r"^DC8[A-Z0-9]*$", dict(wing="low", engines=4))
F("douglas_dc9_md80_90", "Douglas DC-9 / MD-80 / MD-90", "narrowbody", ["douglas"],
  r"^DC9[A-Z0-9]*$|^MD8[0-9][A-Z0-9]*$|^MD90[A-Z0-9]*$|^C9[AB]?$", dict(wing="low", engines=2, engine_layout="aft_fuselage"),
  [V("md90", "MD-90", r"^MD90|^DC99"), V("md80", "MD-80 series (DC-9-81/82/83/87, MD-88)", r"^DC98|^MD8"),
   V("dc9", "DC-9-10 to -50", r"^DC9|^C9")])
F("airbus_a220", "Airbus A220 (Bombardier CS100/CS300, BD-500)", "narrowbody", ["airbus"], r"^A220|^BD500|^CS[13]00", LW, priority=150)
F("airbus_a320_family", "Airbus A318 / A319 / A320 / A321", "narrowbody", ["airbus"], r"^A3(18|19|20|21)[A-Z0-9]*$", LW,
  [V("a318", "A318", r"^A318"), V("a319", "A319", r"^A319"), V("a320", "A320", r"^A320"), V("a321", "A321", r"^A321")])

# ======== Widebody
F("boeing_747", "Boeing 747", "widebody", ["boeing"], r"^747[A-Z0-9]*$|^VC25|^E4B?$", dict(wing="low", engines=4),
  [V("b747_8", "747-8", r"^7478"), V("b747_400", "747-400", r"^7474"),
   V("b747_classic", "747-100/200/300/SP", r"^747(1|2|3|SP)|^VC25|^E4")])
F("boeing_767", "Boeing 767", "widebody", ["boeing"], r"^767[A-Z0-9]*$|^KC46", dict(wing="low", engines=2),
  [V("b767_400", "767-400", r"^7674"), V("b767_300", "767-300", r"^7673"), V("b767_200", "767-200 / KC-46", r"^7672|^KC46")])
F("boeing_777", "Boeing 777", "widebody", ["boeing"], r"^777[A-Z0-9]*$", dict(wing="low", engines=2),
  [V("b777x", "777-8 / 777-9", r"^777[89]$|^777[89]X"), V("b777_300", "777-300 / 300ER", r"^7773"),
   V("b777_200", "777-200 / 200ER / 200LR / F", r"^7772|^777F")])
F("boeing_787", "Boeing 787", "widebody", ["boeing"], r"^787[A-Z0-9]*$", dict(wing="low", engines=2),
  [V("b787_8", "787-8", r"^7878"), V("b787_9", "787-9", r"^7879"), V("b787_10", "787-10", r"^78710")])
F("douglas_dc10_md11", "McDonnell Douglas DC-10 / MD-10 / MD-11", "widebody", ["douglas"], r"^DC10[A-Z0-9]*$|^MD1[01][A-Z0-9]*$|^KC10",
  dict(wing="low", engines=3),
  [V("md11", "MD-11", r"^MD11"), V("dc10", "DC-10 / MD-10 / KC-10", r"^DC10|^MD10|^KC10")])
F("lockheed_l1011", "Lockheed L-1011 TriStar", "widebody", ["lockheed"], r"^L?1011[A-Z0-9]*$|^TRISTAR", dict(wing="low", engines=3))
F("airbus_a300_310", "Airbus A300 / A310", "widebody", ["airbus"], r"^A3[01]0[A-Z0-9]*$", dict(wing="low", engines=2),
  [V("a300", "A300", r"^A300"), V("a310", "A310", r"^A310")])
F("airbus_a330", "Airbus A330", "widebody", ["airbus"], r"^A330[A-Z0-9]*$", dict(wing="low", engines=2))
F("airbus_a340", "Airbus A340", "widebody", ["airbus"], r"^A340[A-Z0-9]*$", dict(wing="low", engines=4))
F("airbus_a350", "Airbus A350", "widebody", ["airbus"], r"^A350[A-Z0-9]*$", dict(wing="low", engines=2))
F("airbus_a380", "Airbus A380", "widebody", ["airbus"], r"^A380[A-Z0-9]*$", dict(wing="low", engines=4))

# ======== Legacy transports
F("douglas_dc3", "Douglas DC-3 / C-47 / Basler BT-67", "legacy_transport", ["douglas"],
  r"^DC3[A-Z0-9]*$|^C47[A-Z0-9]*$|^C53|^R4D|^BT67|^SUPERDC3|^C117|^DST|^TP3", dict(engines=2, propulsion="piston", gear_config="tailwheel", wing="low"),
  [V("bt67", "Basler BT-67 / turbine DC-3", r"^BT67|^DC3(TP|65TP)|^TP3", propulsion="turboprop"),
   V("dc3", "DC-3 / C-47 / R4D", r".")], priority=150)
F("douglas_dc4_6_7", "Douglas DC-4 / DC-6 / DC-7", "legacy_transport", ["douglas"], r"^DC[467][A-Z0-9]*$|^C54|^C118|^R[56]D",
  dict(engines=4, propulsion="piston", gear_config="tricycle", wing="low"),
  [V("dc4", "DC-4 / C-54", r"^DC4|^C54|^R5D"), V("dc6", "DC-6 / C-118", r"^DC6|^C118|^R6D"), V("dc7", "DC-7", r"^DC7")])
F("convair_piston", "Convair 240 / 340 / 440", "legacy_transport", ["convair"], r"^(CV|CONVAIR)?[234]40[A-Z0-9]*$|^T29|^C131",
  dict(engines=2, propulsion="piston", wing="low"))
F("curtiss_c46", "Curtiss C-46 Commando", "legacy_transport", ["curtiss"], r"^C46|^CW20", dict(engines=2, propulsion="piston", gear_config="tailwheel"))
F("lockheed_l188", "Lockheed L-188 Electra", "legacy_transport", ["lockheed"], r"^L?188[A-Z0-9]*$|^ELECTRA", dict(engines=4, propulsion="turboprop"))
F("lockheed_l382", "Lockheed L-100/L-382 Hercules / C-130", "legacy_transport", ["lockheed"], r"^L?382[A-Z0-9]*$|^L100|^C130|^HERCULES",
  dict(engines=4, propulsion="turboprop", wing="high"))

# ======== Vintage & warbird
F("stearman", "Boeing-Stearman Model 75 / PT-17 / N2S Kaydet", "vintage", ["stearman"],
  r"^[ABDE]75[A-Z]?[0-9]*[A-Z]?[0-9]*$|^75$|^PT1[378][A-Z]?$|^N2S[0-9]*$|^STEARMAN|^KAYDET",
  dict(engines=1, propulsion="piston", gear="fixed", gear_config="tailwheel", wing="biplane"), priority=150)
F("north_american_t6", "North American T-6 / SNJ / Harvard", "vintage", ["north_american"],
  r"^(AT|T)6[A-Z0-9]*$|^SNJ|^HARVARD|^TEXAN$|^NA(16|66|77|88)", dict(engines=1, propulsion="piston", gear="retract", gear_config="tailwheel", wing="low"))
F("north_american_p51", "North American P-51 Mustang", "vintage", ["north_american"], r"^T?F?P?51[A-Z]?$|^P51|^TF51|^F51|^CA18|^MUSTANG",
  dict(engines=1, propulsion="piston", gear="retract", gear_config="tailwheel", wing="low"))
F("north_american_t28", "North American T-28 Trojan", "vintage", ["north_american"], r"^T28|^TROJAN|^FENNEC",
  dict(engines=1, propulsion="piston", gear="retract", gear_config="tricycle", wing="low"))
F("yak_nanchang", "Yak-52 / Yak-18 / Yak-50/54/55 / Nanchang CJ-6", "vintage", ["yak"], r"^YAK|^IAK|^CJ6|^PT6A?$",
  dict(engines=1, propulsion="piston", gear="retract", wing="low"),
  [V("yak52", "Yak-52 / Yak-52TW", r"52"), V("cj6", "Nanchang CJ-6", r"CJ6|^PT6"),
   V("yak_aerobatic", "Yak-50 / 54 / 55", r"5[045]", cls="sep_aerobatic"), V("yak18", "Yak-18", r"18")])
F("aero_l39", "Aero L-29 Delfin / L-39 Albatros", "vintage", ["aero_vodochody"], r"^L(29|39|59)[A-Z0-9]*$|^ALBATROS|^DELFIN",
  dict(engines=1, propulsion="turbofan", gear="retract", gear_config="tricycle", wing="low"))
F("waco", "Waco biplanes (UPF-7, YMF, YKS, etc.)", "vintage", ["waco"], r".",
  dict(engines=1, propulsion="piston", gear="fixed", gear_config="tailwheel", wing="biplane"))
F("ryan_pt22", "Ryan ST / PT-22 Recruit", "vintage", ["ryan"], r"^ST[A-Z0-9]*$|^PT2[012]|^STM|^RECRUIT",
  dict(engines=1, propulsion="piston", gear="fixed", gear_config="tailwheel", wing="low"))
F("fairchild_pt19", "Fairchild PT-19 / PT-23 / PT-26 (M-62)", "vintage", ["fairchild_vintage"], r"^M62|^PT(19|23|26)|^CORNELL",
  dict(engines=1, propulsion="piston", gear="fixed", gear_config="tailwheel", wing="low"))
F("great_lakes", "Great Lakes 2T-1A", "vintage", ["great_lakes"], r".",
  dict(engines=1, propulsion="piston", gear="fixed", gear_config="tailwheel", wing="biplane"))
