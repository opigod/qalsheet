package com.qalsheet.chem

data class ChemState(
    val oxidation: Int,
    val adjective: String,
    val acidName: String? = null,
    val acidFormula: String? = null,
)

data class ChemElement(
    val symbol: String,
    val czech: String,
    val english: String,
    val metal: Boolean,
    val states: List<ChemState>,
) {
    fun state(oxidation: Int): ChemState =
        states.firstOrNull { it.oxidation == oxidation } ?: states.last()
}

data class CompoundExample(
    val name: String,
    val formula: String,
)

data class SuffixRule(
    val oxidation: Int,
    val suffix: String,
    val exampleName: String,
    val exampleFormula: String,
    val note: String? = null,
)

data class GuideExample(
    val name: String,
    val formula: String,
)

data class GuideSection(
    val title: String,
    val body: String,
    val examples: List<GuideExample> = emptyList(),
    val steps: List<String> = emptyList(),
)

/**
 * Curated Czech oxidation-state data. The valuable part is the adjective, which
 * cannot be derived mechanically from the element name (vápník → vápenatý,
 * železo → železitý, chlor → chlorečný). Sources and update rules are in
 * AGENTS.md. Keep formulas in ASCII digits; the UI renders them as subscripts.
 */
object ChemData {

    val elements: List<ChemElement> = listOf(
        ChemElement(
            symbol = "Sb",
            czech = "antimon",
            english = "Antimony",
            metal = false,
            states = listOf(
                ChemState(3, "antimonitý"),
                ChemState(5, "antimoničný"),
            ),
        ),
        ChemElement(
            symbol = "As",
            czech = "arsen",
            english = "Arsenic",
            metal = false,
            states = listOf(
                ChemState(3, "arzenitý", "kyselina arzenitá", "H3AsO3"),
                ChemState(5, "arzeničný", "kyselina arzeničná", "H3AsO4"),
            ),
        ),
        ChemElement(
            symbol = "Ba",
            czech = "baryum",
            english = "Barium",
            metal = true,
            states = listOf(ChemState(2, "barnatý")),
        ),
        ChemElement(
            symbol = "Be",
            czech = "beryllium",
            english = "Beryllium",
            metal = true,
            states = listOf(ChemState(2, "berylnatý")),
        ),
        ChemElement(
            symbol = "Bi",
            czech = "bismut",
            english = "Bismuth",
            metal = true,
            states = listOf(
                ChemState(3, "bismutitý"),
                ChemState(5, "bismutečný"),
            ),
        ),
        ChemElement(
            symbol = "B",
            czech = "bor",
            english = "Boron",
            metal = false,
            states = listOf(
                ChemState(3, "boritý", "kyselina boritá", "H3BO3"),
            ),
        ),
        ChemElement(
            symbol = "Br",
            czech = "brom",
            english = "Bromine",
            metal = false,
            states = listOf(
                ChemState(1, "bromný", "kyselina bromná", "HBrO"),
                ChemState(3, "bromitý", "kyselina bromitá", "HBrO2"),
                ChemState(4, "bromičitý"),
                ChemState(5, "bromičný", "kyselina bromičná", "HBrO3"),
                ChemState(7, "bromistý", "kyselina bromistá", "HBrO4"),
            ),
        ),
        ChemElement(
            symbol = "Sn",
            czech = "cín",
            english = "Tin",
            metal = true,
            states = listOf(
                ChemState(2, "cínatý"),
                ChemState(4, "cíničitý"),
            ),
        ),
        ChemElement(
            symbol = "K",
            czech = "draslík",
            english = "Potassium",
            metal = true,
            states = listOf(ChemState(1, "draselný")),
        ),
        ChemElement(
            symbol = "N",
            czech = "dusík",
            english = "Nitrogen",
            metal = false,
            states = listOf(
                ChemState(1, "dusný"),
                ChemState(2, "dusnatý"),
                ChemState(3, "dusitý", "kyselina dusitá", "HNO2"),
                ChemState(4, "dusičitý"),
                ChemState(5, "dusičný", "kyselina dusičná", "HNO3"),
            ),
        ),
        ChemElement(
            symbol = "P",
            czech = "fosfor",
            english = "Phosphorus",
            metal = false,
            states = listOf(
                ChemState(3, "fosforitý", "kyselina fosforitá", "H3PO3"),
                ChemState(5, "fosforečný", "kyselina fosforečná", "H3PO4"),
            ),
        ),
        ChemElement(
            symbol = "Al",
            czech = "hliník",
            english = "Aluminium",
            metal = true,
            states = listOf(ChemState(3, "hlinitý")),
        ),
        ChemElement(
            symbol = "Mg",
            czech = "hořčík",
            english = "Magnesium",
            metal = true,
            states = listOf(ChemState(2, "hořečnatý")),
        ),
        ChemElement(
            symbol = "Cl",
            czech = "chlor",
            english = "Chlorine",
            metal = false,
            states = listOf(
                ChemState(1, "chlorný", "kyselina chlorná", "HClO"),
                ChemState(3, "chloritý", "kyselina chloritá", "HClO2"),
                ChemState(4, "chloričitý"),
                ChemState(5, "chlorečný", "kyselina chlorečná", "HClO3"),
                ChemState(7, "chloristý", "kyselina chloristá", "HClO4"),
            ),
        ),
        ChemElement(
            symbol = "Cr",
            czech = "chrom",
            english = "Chromium",
            metal = true,
            states = listOf(
                ChemState(2, "chromnatý"),
                ChemState(3, "chromitý"),
                ChemState(6, "chromový", "kyselina chromová", "H2CrO4"),
            ),
        ),
        ChemElement(
            symbol = "I",
            czech = "jod",
            english = "Iodine",
            metal = false,
            states = listOf(
                ChemState(1, "jodný", "kyselina jodná", "HIO"),
                ChemState(3, "joditý", "kyselina joditá", "HIO2"),
                ChemState(4, "jodičitý"),
                ChemState(5, "jodičný", "kyselina jodičná", "HIO3"),
                ChemState(7, "jodistý", "kyselina jodistá", "HIO4"),
            ),
        ),
        ChemElement(
            symbol = "Cd",
            czech = "kadmium",
            english = "Cadmium",
            metal = true,
            states = listOf(ChemState(2, "kademnatý")),
        ),
        ChemElement(
            symbol = "Co",
            czech = "kobalt",
            english = "Cobalt",
            metal = true,
            states = listOf(
                ChemState(2, "kobaltnatý"),
                ChemState(3, "kobaltitý"),
            ),
        ),
        ChemElement(
            symbol = "Si",
            czech = "křemík",
            english = "Silicon",
            metal = false,
            states = listOf(
                ChemState(4, "křemičitý", "kyselina křemičitá", "H2SiO3"),
            ),
        ),
        ChemElement(
            symbol = "Li",
            czech = "lithium",
            english = "Lithium",
            metal = true,
            states = listOf(ChemState(1, "lithný")),
        ),
        ChemElement(
            symbol = "Mn",
            czech = "mangan",
            english = "Manganese",
            metal = true,
            states = listOf(
                ChemState(2, "manganatý"),
                ChemState(3, "manganitý"),
                ChemState(4, "manganičitý"),
                ChemState(6, "manganový"),
                ChemState(7, "manganistý", "kyselina manganistá", "HMnO4"),
            ),
        ),
        ChemElement(
            symbol = "Cu",
            czech = "měď",
            english = "Copper",
            metal = true,
            states = listOf(
                ChemState(1, "měďný"),
                ChemState(2, "měďnatý"),
            ),
        ),
        ChemElement(
            symbol = "Mo",
            czech = "molybden",
            english = "Molybdenum",
            metal = true,
            states = listOf(
                ChemState(6, "molybdenový", "kyselina molybdenová", "H2MoO4"),
            ),
        ),
        ChemElement(
            symbol = "Ni",
            czech = "nikl",
            english = "Nickel",
            metal = true,
            states = listOf(
                ChemState(2, "nikelnatý"),
                ChemState(3, "nikelitý"),
            ),
        ),
        ChemElement(
            symbol = "Pb",
            czech = "olovo",
            english = "Lead",
            metal = true,
            states = listOf(
                ChemState(2, "olovnatý"),
                ChemState(4, "olovičitý"),
            ),
        ),
        ChemElement(
            symbol = "Os",
            czech = "osmium",
            english = "Osmium",
            metal = true,
            states = listOf(
                ChemState(4, "osmičitý"),
                ChemState(8, "osmičelý"),
            ),
        ),
        ChemElement(
            symbol = "Pt",
            czech = "platina",
            english = "Platinum",
            metal = true,
            states = listOf(
                ChemState(2, "platinatý"),
                ChemState(4, "platiničitý"),
            ),
        ),
        ChemElement(
            symbol = "Hg",
            czech = "rtuť",
            english = "Mercury",
            metal = true,
            states = listOf(
                ChemState(1, "rtuťný"),
                ChemState(2, "rtuťnatý"),
            ),
        ),
        ChemElement(
            symbol = "Se",
            czech = "selen",
            english = "Selenium",
            metal = false,
            states = listOf(
                ChemState(4, "seleničitý", "kyselina seleničitá", "H2SeO3"),
                ChemState(6, "selenový", "kyselina selenová", "H2SeO4"),
            ),
        ),
        ChemElement(
            symbol = "S",
            czech = "síra",
            english = "Sulfur",
            metal = false,
            states = listOf(
                ChemState(4, "siřičitý", "kyselina siřičitá", "H2SO3"),
                ChemState(6, "sírový", "kyselina sírová", "H2SO4"),
            ),
        ),
        ChemElement(
            symbol = "Na",
            czech = "sodík",
            english = "Sodium",
            metal = true,
            states = listOf(ChemState(1, "sodný")),
        ),
        ChemElement(
            symbol = "Ag",
            czech = "stříbro",
            english = "Silver",
            metal = true,
            states = listOf(ChemState(1, "stříbrný")),
        ),
        ChemElement(
            symbol = "Sr",
            czech = "stroncium",
            english = "Strontium",
            metal = true,
            states = listOf(ChemState(2, "strontnatý")),
        ),
        ChemElement(
            symbol = "Te",
            czech = "tellur",
            english = "Tellurium",
            metal = false,
            states = listOf(
                ChemState(4, "telluričitý", "kyselina telluričitá", "H2TeO3"),
                ChemState(6, "tellurový", "kyselina tellurová", "H2TeO4"),
            ),
        ),
        ChemElement(
            symbol = "Ti",
            czech = "titan",
            english = "Titanium",
            metal = true,
            states = listOf(ChemState(4, "titaničitý")),
        ),
        ChemElement(
            symbol = "C",
            czech = "uhlík",
            english = "Carbon",
            metal = false,
            states = listOf(
                ChemState(2, "uhelnatý"),
                ChemState(4, "uhličitý", "kyselina uhličitá", "H2CO3"),
            ),
        ),
        ChemElement(
            symbol = "U",
            czech = "uran",
            english = "Uranium",
            metal = true,
            states = listOf(
                ChemState(4, "uraničitý"),
                ChemState(6, "uranový"),
            ),
        ),
        ChemElement(
            symbol = "V",
            czech = "vanad",
            english = "Vanadium",
            metal = true,
            states = listOf(
                ChemState(2, "vanadnatý"),
                ChemState(3, "vanaditý"),
                ChemState(4, "vanadičitý"),
                ChemState(5, "vanadičný"),
            ),
        ),
        ChemElement(
            symbol = "Ca",
            czech = "vápník",
            english = "Calcium",
            metal = true,
            states = listOf(ChemState(2, "vápenatý")),
        ),
        ChemElement(
            symbol = "W",
            czech = "wolfram",
            english = "Tungsten",
            metal = true,
            states = listOf(
                ChemState(6, "wolframový", "kyselina wolframová", "H2WO4"),
            ),
        ),
        ChemElement(
            symbol = "Xe",
            czech = "xenon",
            english = "Xenon",
            metal = false,
            states = listOf(ChemState(8, "xenoničelý")),
        ),
        ChemElement(
            symbol = "Zn",
            czech = "zinek",
            english = "Zinc",
            metal = true,
            states = listOf(ChemState(2, "zinečnatý")),
        ),
        ChemElement(
            symbol = "Au",
            czech = "zlato",
            english = "Gold",
            metal = true,
            states = listOf(
                ChemState(1, "zlatný"),
                ChemState(3, "zlatitý"),
            ),
        ),
        ChemElement(
            symbol = "Fe",
            czech = "železo",
            english = "Iron",
            metal = true,
            states = listOf(
                ChemState(2, "železnatý"),
                ChemState(3, "železitý"),
            ),
        ),
    )

    val suffixRules = listOf(
        SuffixRule(1, "-ný", "oxid měďný", "Cu2O"),
        SuffixRule(2, "-natý", "oxid železnatý", "FeO"),
        SuffixRule(3, "-itý", "oxid železitý", "Fe2O3"),
        SuffixRule(4, "-ičitý", "oxid olovičitý", "PbO2"),
        SuffixRule(
            5,
            "-ičný / -ečný",
            "oxid dusičný",
            "N2O5",
            "-ečný: fosforečný, chlorečný, bismutečný",
        ),
        SuffixRule(6, "-ový", "oxid sírový", "SO3"),
        SuffixRule(7, "-istý", "oxid chloristý", "Cl2O7"),
        SuffixRule(8, "-ičelý", "oxid osmičelý", "OsO4"),
    )

    val guideSections = listOf(
        GuideSection(
            title = "Binary compounds",
            body = "The anion is the noun (oxid, chlorid, sulfid, …). The cation is an adjective whose ending carries the oxidation state.",
            examples = listOf(
                GuideExample("chlorid železnatý", "FeCl2"),
                GuideExample("chlorid železitý", "FeCl3"),
                GuideExample("oxid hlinitý", "Al2O3"),
            ),
        ),
        GuideSection(
            title = "Hydroxides",
            body = "Use hydroxid plus the cation adjective:",
            examples = listOf(
                GuideExample("hydroxid sodný", "NaOH"),
                GuideExample("hydroxid vápenatý", "Ca(OH)2"),
            ),
        ),
        GuideSection(
            title = "Acids",
            body = "Binary acids take -ová. Oxoacids use the adjective of the central atom: H2SO4 has sulfur(VI), HClO4 has chlorine(VII).",
            examples = listOf(
                GuideExample("kyselina chlorovodíková", "HCl"),
                GuideExample("kyselina sírová", "H2SO4"),
                GuideExample("kyselina chloristá", "HClO4"),
            ),
        ),
        GuideSection(
            title = "Salts",
            body = "Name the anion first, then the cation adjective. Hydrogen salts add the hydrogen- prefix.",
            examples = listOf(
                GuideExample("chlorid sodný", "NaCl"),
                GuideExample("síran draselný", "K2SO4"),
                GuideExample("uhličitan vápenatý", "CaCO3"),
                GuideExample("hydrogenuhličitan sodný", "NaHCO3"),
            ),
        ),
        GuideSection(
            title = "Organic names",
            body = "Czech organic nomenclature follows IUPAC substitution rules.",
            steps = listOf(
                "Pick the principal characteristic group; it becomes the suffix.",
                "Find the longest parent chain that contains it.",
                "Number the chain to give that group the lowest locant.",
                "Name substituents as prefixes and list them alphabetically.",
                "Assemble: prefixes + parent + suffix.",
            ),
            examples = listOf(
                GuideExample("2-chlorpropan-1-ol", "CH3-CH(Cl)-CH2-OH"),
            ),
        ),
    )

    fun find(symbol: String): ChemElement = elements.first { it.symbol == symbol }

    fun examplesFor(element: ChemElement, state: ChemState): List<CompoundExample> {
        val examples = mutableListOf<CompoundExample>()
        examples += CompoundExample(
            name = "oxid ${state.adjective}",
            formula = oxideFormula(element.symbol, state.oxidation),
        )
        if (element.metal && state.oxidation in 2..3) {
            examples += CompoundExample(
                name = "hydroxid ${state.adjective}",
                formula = hydroxideFormula(element.symbol, state.oxidation),
            )
        }
        if (state.acidName != null && state.acidFormula != null) {
            examples += CompoundExample(state.acidName, state.acidFormula)
        }
        return examples.take(3)
    }

    fun oxideFormula(symbol: String, oxidation: Int): String {
        require(oxidation in 1..8) { "Unsupported oxidation state $oxidation" }
        val divisor = gcd(2, oxidation)
        val elementCount = 2 / divisor
        val oxygenCount = oxidation / divisor
        return buildString {
            append(symbol)
            if (elementCount > 1) append(elementCount)
            append('O')
            if (oxygenCount > 1) append(oxygenCount)
        }
    }

    fun hydroxideFormula(symbol: String, oxidation: Int): String {
        require(oxidation in 2..3) { "Simple hydroxides are generated for states II and III only" }
        return buildString {
            append(symbol)
            append("(OH)")
            if (oxidation > 1) append(oxidation)
        }
    }

    fun suffixOf(adjective: String): String? {
        val suffixes = listOf(
            "ičelý",
            "ičitý",
            "ečný",
            "ičný",
            "natý",
            "istý",
            "itý",
            "ový",
            "ný",
        )
        val suffix = suffixes.firstOrNull { adjective.endsWith(it) } ?: return null
        return "-$suffix"
    }

    fun superscriptCharge(oxidation: Int): String {
        val digits = mapOf(
            '0' to '⁰', '1' to '¹', '2' to '²', '3' to '³', '4' to '⁴',
            '5' to '⁵', '6' to '⁶', '7' to '⁷', '8' to '⁸', '9' to '⁹',
        )
        return buildString {
            oxidation.toString().forEach { append(digits[it] ?: it) }
            append('⁺')
        }
    }

    fun roman(oxidation: Int): String = when (oxidation) {
        1 -> "I"
        2 -> "II"
        3 -> "III"
        4 -> "IV"
        5 -> "V"
        6 -> "VI"
        7 -> "VII"
        8 -> "VIII"
        else -> oxidation.toString()
    }

    private fun gcd(a: Int, b: Int): Int {
        var x = a
        var y = b
        while (y != 0) {
            val temp = y
            y = x % y
            x = temp
        }
        return x
    }
}
