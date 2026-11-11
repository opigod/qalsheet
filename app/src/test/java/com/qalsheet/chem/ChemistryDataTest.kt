package com.qalsheet.chem

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ChemistryDataTest {

    @Test
    fun elementSymbolsAreUniqueAndEveryStateHasAnAdjective() {
        assertEquals(ChemData.elements.size, ChemData.elements.map { it.symbol }.toSet().size)
        ChemData.elements.forEach { element ->
            assertTrue("${element.symbol} has no states", element.states.isNotEmpty())
            assertEquals(
                "Duplicate oxidation state for ${element.symbol}",
                element.states.size,
                element.states.map { it.oxidation }.toSet().size,
            )
            element.states.forEach { state ->
                assertTrue(state.oxidation in 1..8)
                assertTrue(state.adjective.isNotBlank())
                assertNotNull(
                    "Unrecognised suffix for ${state.adjective}",
                    ChemData.suffixOf(state.adjective),
                )
                if (state.acidName != null) {
                    assertNotNull(state.acidFormula)
                    assertTrue(state.acidName.startsWith("kyselina"))
                }
            }
        }
    }

    @Test
    fun ironThreeIsZelezity() {
        val iron = ChemData.find("Fe")
        val state = iron.state(3)
        assertEquals("železitý", state.adjective)
        assertEquals("-itý", ChemData.suffixOf(state.adjective))
        assertEquals("Fe2O3", ChemData.oxideFormula("Fe", 3))
        assertEquals("Fe(OH)3", ChemData.hydroxideFormula("Fe", 3))
        val examples = ChemData.examplesFor(iron, state)
        assertTrue(examples.any { it.name == "oxid železitý" && it.formula == "Fe2O3" })
        assertTrue(examples.any { it.name == "hydroxid železitý" && it.formula == "Fe(OH)3" })
    }

    @Test
    fun commonSuffixesMatchTheCzechSystem() {
        assertEquals("-ný", ChemData.suffixOf("měďný"))
        assertEquals("-natý", ChemData.suffixOf("železnatý"))
        assertEquals("-itý", ChemData.suffixOf("hlinitý"))
        assertEquals("-ičitý", ChemData.suffixOf("olovičitý"))
        assertEquals("-ečný", ChemData.suffixOf("fosforečný"))
        assertEquals("-ičný", ChemData.suffixOf("dusičný"))
        assertEquals("-ový", ChemData.suffixOf("sírový"))
        assertEquals("-istý", ChemData.suffixOf("manganistý"))
        assertEquals("-ičelý", ChemData.suffixOf("osmičelý"))
    }

    @Test
    fun oxideFormulasBalanceTheSelectedState() {
        assertEquals("Cu2O", ChemData.oxideFormula("Cu", 1))
        assertEquals("FeO", ChemData.oxideFormula("Fe", 2))
        assertEquals("Fe2O3", ChemData.oxideFormula("Fe", 3))
        assertEquals("PbO2", ChemData.oxideFormula("Pb", 4))
        assertEquals("N2O5", ChemData.oxideFormula("N", 5))
        assertEquals("SO3", ChemData.oxideFormula("S", 6))
        assertEquals("Mn2O7", ChemData.oxideFormula("Mn", 7))
        assertEquals("OsO4", ChemData.oxideFormula("Os", 8))
        assertEquals("XeO4", ChemData.oxideFormula("Xe", 8))
    }

    @Test
    fun keyElementsAndAcidsArePresent() {
        val nitrogen = ChemData.find("N").state(5)
        assertEquals("dusičný", nitrogen.adjective)
        assertEquals("kyselina dusičná", nitrogen.acidName)
        assertEquals("HNO3", nitrogen.acidFormula)

        val sulfur = ChemData.find("S").state(6)
        assertEquals("sírový", sulfur.adjective)
        assertEquals("H2SO4", sulfur.acidFormula)

        val chlorine = ChemData.find("Cl")
        assertEquals("chlorečný", chlorine.state(5).adjective)
        assertEquals("-ečný", ChemData.suffixOf(chlorine.state(5).adjective))
        assertEquals("chloristý", chlorine.state(7).adjective)

        val manganese = ChemData.find("Mn")
        assertEquals("manganistý", manganese.state(7).adjective)
        assertEquals("Mn2O7", ChemData.oxideFormula("Mn", 7))

        val osmium = ChemData.find("Os")
        assertEquals("osmičelý", osmium.state(8).adjective)

        assertEquals("xenoničelý", ChemData.find("Xe").state(8).adjective)
    }

    @Test
    fun exampleCompoundsStayCorrectForRepresentativeElements() {
        val copperOne = ChemData.examplesFor(ChemData.find("Cu"), ChemData.find("Cu").state(1))
        assertEquals("oxid měďný", copperOne.first().name)
        assertEquals("Cu2O", copperOne.first().formula)

        val magnesium = ChemData.find("Mg").state(2)
        val magnesiumExamples = ChemData.examplesFor(ChemData.find("Mg"), magnesium)
        assertEquals("oxid hořečnatý", magnesiumExamples[0].name)
        assertEquals("MgO", magnesiumExamples[0].formula)
        assertEquals("hydroxid hořečnatý", magnesiumExamples[1].name)
        assertEquals("Mg(OH)2", magnesiumExamples[1].formula)
    }

    @Test
    fun guideHasAllSectionsAndSuffixRules() {
        assertEquals(8, ChemData.suffixRules.size)
        assertEquals(5, ChemData.guideSections.size)
        assertTrue(ChemData.guideSections.any { it.title == "Organic names" && it.steps.size >= 5 })
        assertTrue(ChemData.suffixRules.any { it.suffix == "-ičný / -ečný" })
    }
}
