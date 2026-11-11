package com.qalsheet.calc

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class CalculatorEngineTest {

    private fun value(expression: String): Double {
        val evaluable = CalculatorEngine.evaluable(expression)
        checkNotNull(evaluable) { "Could not make '$expression' evaluable" }
        return checkNotNull(CalculatorEngine.evaluate(evaluable)) { "Could not evaluate '$expression'" }
    }

    @Test
    fun respectsOperatorPrecedence() {
        assertEquals(14.0, value("2+3×4"), 1e-9)
        assertEquals(20.0, value("(2+3)×4"), 1e-9)
        assertEquals(2.0, value("10÷5"), 1e-9)
        assertEquals(1.0, value("10−6−3"), 1e-9)
    }

    @Test
    fun supportsUnaryMinus() {
        assertEquals(-3.0, value("−5+2"), 1e-9)
        assertEquals(-10.0, value("5×−2"), 1e-9)
        assertEquals(25.0, value("−5×−5"), 1e-9)
    }

    @Test
    fun supportsDecimalsAndPowers() {
        assertEquals(0.3, value("0.1+0.2"), 1e-9)
        assertEquals(8.0, value("2^3"), 1e-9)
        assertEquals(512.0, value("2^3^2"), 1e-9)
    }

    @Test
    fun rejectsInvalidExpressions() {
        assertNull(CalculatorEngine.evaluate("12÷0"))
        assertNull(CalculatorEngine.evaluate("2+"))
        assertNull(CalculatorEngine.evaluable("2+)"))
        assertNull(CalculatorEngine.evaluable(""))
        assertNull(CalculatorEngine.evaluable("."))
    }

    @Test
    fun repairsUnfinishedExpressions() {
        assertEquals("2+3", CalculatorEngine.evaluable("2+3×"))
        assertEquals("(2+3)", CalculatorEngine.evaluable("(2+3"))
        assertEquals("5", CalculatorEngine.evaluable("5×("))
    }

    @Test
    fun formatsNumbersForAWatch() {
        assertEquals("0", CalculatorEngine.format(0.0))
        assertEquals("15", CalculatorEngine.format(15.0000000000001))
        assertEquals("0.3", CalculatorEngine.format(0.1 + 0.2))
        assertEquals("−12.5", CalculatorEngine.format(-12.5))
        assertEquals("1\u00A0234\u00A0567", CalculatorEngine.format(1_234_567.0, grouping = true))
        assertTrue(CalculatorEngine.format(0.000000001).contains("e-"))
    }

    @Test
    fun stateBuildsHistoryAndContinuesFromResult() {
        val state = CalculatorState()
        "12+3=".forEach { state.press(it.toString()) }
        assertEquals("15", state.displayText)
        assertEquals(1, state.history.size)
        assertEquals("12+3", state.history.first().expression)
        assertEquals("15", state.history.first().result)

        state.press("×")
        state.press("2")
        state.press("=")
        assertEquals("30", state.displayText)
        assertEquals(2, state.history.size)
    }

    @Test
    fun decimalPointIsOnlyAddedOncePerNumber() {
        val state = CalculatorState()
        listOf("1", ".", "2", ".", "3").forEach(state::press)
        assertEquals("1.23", state.displayText)
    }

    @Test
    fun clearAndBackspaceWork() {
        val state = CalculatorState()
        listOf("9", "9", "⌫").forEach(state::press)
        assertEquals("9", state.displayText)
        state.press("C")
        assertEquals("0", state.displayText)
        state.press("7")
        assertEquals("7", state.displayText)
    }
}
