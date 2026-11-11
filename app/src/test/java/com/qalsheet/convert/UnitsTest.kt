package com.qalsheet.convert

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class UnitsTest {

    private val length = Units.length
    private val mass = Units.mass
    private val temperature = Units.temperature
    private val data = Units.data
    private val volume = Units.volume

    @Test
    fun convertsCentimetersToMillimeters() {
        val result = Units.convert(12.5, length.unit("cm"), length.unit("mm"))
        assertEquals(125.0, result, 1e-9)
    }

    @Test
    fun convertsKilogramsToGrams() {
        val result = Units.convert(2.0, mass.unit("kg"), mass.unit("g"))
        assertEquals(2_000.0, result, 1e-9)
    }

    @Test
    fun convertsInchesToMillimeters() {
        val result = Units.convert(1.0, length.unit("in"), length.unit("mm"))
        assertEquals(25.4, result, 1e-9)
    }

    @Test
    fun convertsTemperatures() {
        assertEquals(32.0, Units.convert(0.0, temperature.unit("°C"), temperature.unit("°F")), 1e-9)
        assertEquals(212.0, Units.convert(100.0, temperature.unit("°C"), temperature.unit("°F")), 1e-9)
        assertEquals(-40.0, Units.convert(-40.0, temperature.unit("°C"), temperature.unit("°F")), 1e-9)
        assertEquals(273.15, Units.convert(0.0, temperature.unit("°C"), temperature.unit("K")), 1e-9)
    }

    @Test
    fun convertsDataUsingDecimalAndBinaryPrefixes() {
        assertEquals(1_000.0, Units.convert(1.0, data.unit("GB"), data.unit("MB")), 1e-9)
        assertEquals(1_024.0, Units.convert(1.0, data.unit("GiB"), data.unit("MiB")), 1e-9)
    }

    @Test
    fun convertsGallonsToLiters() {
        assertEquals(3.785411784, Units.convert(1.0, volume.unit("gal"), volume.unit("l")), 1e-12)
    }

    @Test
    fun everyCategoryHasValidDefaultUnits() {
        Units.categories.forEach { category ->
            assertTrue(category.units.size >= 2)
            assertTrue(category.units.any { it.symbol == category.defaultFrom })
            assertTrue(category.units.any { it.symbol == category.defaultTo })
            assertEquals(category.defaultFromIndex, category.units.indexOfFirst { it.symbol == category.defaultFrom })
        }
    }

    @Test
    fun formatsReferenceAndNumbers() {
        assertEquals("1 cm = 10 mm", Units.formatReference(length.unit("cm"), length.unit("mm")))
        assertEquals("1\u00A0234\u00A0567", Units.format(1_234_567.0))
    }

    @Test
    fun converterStateSwapsAndKeepsTheRelationship() {
        val state = ConverterState()
        assertEquals("cm", state.from.symbol)
        assertEquals("mm", state.to.symbol)
        assertEquals(10.0, state.result, 1e-9)
        state.swap()
        assertEquals("mm", state.from.symbol)
        assertEquals("cm", state.to.symbol)
        assertEquals("10", state.input)
        assertEquals(1.0, state.result, 1e-9)
    }
}
