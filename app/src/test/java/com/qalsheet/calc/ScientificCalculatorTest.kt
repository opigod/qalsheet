package com.qalsheet.calc

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlin.math.PI

class ScientificCalculatorTest {

    private fun value(expression: String, mode: AngleMode = AngleMode.Degrees): Double {
        val evaluable = CalculatorEngine.evaluable(expression)
        checkNotNull(evaluable) { "Could not make '$expression' evaluable" }
        return checkNotNull(CalculatorEngine.evaluate(evaluable, mode)) {
            "Could not evaluate '$expression' in $mode"
        }
    }

    @Test
    fun trigonometryRespectsDegreeAndRadianModes() {
        assertEquals(0.5, value("sin(30)"), 1e-12)
        assertEquals(0.5, value("cos(60)"), 1e-12)
        assertEquals(1.0, value("tan(45)"), 1e-12)
        assertEquals(0.0, value("sin(180)"), 1e-12)
        assertEquals(1.0, value("sin(π÷2)", AngleMode.Radians), 1e-12)

        assertEquals(30.0, value("asin(0.5)"), 1e-9)
        assertEquals(60.0, value("acos(0.5)"), 1e-9)
        assertEquals(45.0, value("atan(1)"), 1e-9)
        assertEquals(PI / 6.0, value("asin(0.5)", AngleMode.Radians), 1e-9)
    }

    @Test
    fun logarithmsAndRoots() {
        assertEquals(1.0, value("ln(e)"), 1e-12)
        assertEquals(2.0, value("log(100)"), 1e-12)
        assertEquals(3.0, value("log10(1000)"), 1e-12)
        assertEquals(3.0, value("√(9)"), 1e-12)
        assertEquals(4.0, value("√16"), 1e-12)
        assertEquals(5.0, value("sqrt(25)"), 1e-12)
    }

    @Test
    fun powersAreRightAssociativeAndRespectUnaryMinus() {
        assertEquals(512.0, value("2^3^2"), 1e-12)
        assertEquals(64.0, value("(2^3)^2"), 1e-12)
        assertEquals(0.25, value("2^−2"), 1e-12)
        assertEquals(-4.0, value("−2^2"), 1e-12)
        assertEquals(64.0, value("2^3!"), 1e-12)
    }

    @Test
    fun factorialsAndPercentages() {
        assertEquals(120.0, value("5!"), 1e-12)
        assertEquals(1.0, value("0!"), 1e-12)
        assertEquals(0.5, value("50%"), 1e-12)
        assertEquals(20.0, value("200×10%"), 1e-12)
        assertEquals(0.05, value("(2+3)%"), 1e-12)
    }

    @Test
    fun implicitMultiplicationWorks() {
        assertEquals(2 * PI, value("2π"), 1e-12)
        assertEquals(14.0, value("2(3+4)"), 1e-12)
        assertEquals(1.0, value("2sin(30)"), 1e-12)
        assertEquals(6.0, value("(2)(3)"), 1e-12)
        assertEquals(2 * Math.E, value("2e"), 1e-12)
        assertEquals(0.25, value("sin(30)cos(60)"), 1e-12)
    }

    @Test
    fun domainAndOverflowErrorsReturnNull() {
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("1÷0")!!))
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("asin(2)")!!))
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("acos(-2)")!!))
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("ln(0)")!!))
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("log(-1)")!!))
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("√(-4)")!!))
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("tan(90)")!!))
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("10^400")!!))
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("3.5!")!!))
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("(-2)!")!!))
        assertNull(CalculatorEngine.evaluate(CalculatorEngine.evaluable("171!")!!))
        assertNull(CalculatorEngine.evaluable("sin("))
    }

    @Test
    fun evaluableClosesFunctionParentheses() {
        assertEquals("sin(30)", CalculatorEngine.evaluable("sin(30"))
        assertEquals("sin(30)", CalculatorEngine.evaluable("sin(30+"))
        assertEquals("2", CalculatorEngine.evaluable("2^"))
        assertEquals("log(100)", CalculatorEngine.evaluable("log(100"))
    }

    @Test
    fun stateWrapsTheCurrentOperand() {
        val state = CalculatorState()
        state.press("1")
        state.press("2")
        state.press("+")
        state.press("3")
        state.press("0")
        state.showScientificPage(true)
        state.applyScientific(ScientificFunction.Sin)

        assertEquals("12+sin(30)", state.expression)
        assertFalse(state.showScientific)
        state.press("=")
        assertEquals("12.5", state.displayText)
        assertEquals(AngleMode.Degrees, state.history.first().angleMode)
    }

    @Test
    fun squareFactorialPercentAndReciprocalApplyToTheOperand() {
        val state = CalculatorState()
        state.press("5")
        state.applyScientific(ScientificFunction.Square)
        assertEquals("5^2", state.expression)
        state.press("=")
        assertEquals("25", state.displayText)

        state.clear()
        state.press("5")
        state.applyScientific(ScientificFunction.Factorial)
        state.press("=")
        assertEquals("120", state.displayText)

        state.clear()
        state.press("5")
        state.press("0")
        state.applyScientific(ScientificFunction.Percent)
        state.press("=")
        assertEquals("0.5", state.displayText)

        state.clear()
        state.press("4")
        state.applyScientific(ScientificFunction.Reciprocal)
        assertEquals("1/(4)", state.expression)
        state.press("=")
        assertEquals("0.25", state.displayText)
    }

    @Test
    fun signTogglesTheWholeExpression() {
        val state = CalculatorState()
        state.press("2")
        state.press("+")
        state.press("3")
        state.applyScientific(ScientificFunction.Sign)
        assertEquals("−2+3", state.expression)
        state.applyScientific(ScientificFunction.Sign)
        assertEquals("2+3", state.expression)
    }

    @Test
    fun trigHistoryRemembersAngleMode() {
        val state = CalculatorState()
        state.press("3")
        state.press("0")
        state.applyScientific(ScientificFunction.Sin)
        state.press("=")
        assertEquals("0.5", state.displayText)
        assertEquals(AngleMode.Degrees, state.history.first().angleMode)

        state.toggleAngleMode()
        assertEquals(AngleMode.Radians, state.angleMode)
        state.press("3")
        state.press("0")
        state.applyScientific(ScientificFunction.Sin)
        state.press("=")
        assertTrue(state.history.first().angleMode == AngleMode.Radians)
        assertTrue(state.displayText.startsWith("−0.988"))

        val radiansEntry = state.history.first()
        state.loadHistory(radiansEntry)
        assertEquals(AngleMode.Radians, state.angleMode)
    }

    @Test
    fun basicHistoryHasNoAngleModeAndScientificPagePersists() {
        val state = CalculatorState()
        state.press("2")
        state.press("+")
        state.press("2")
        state.press("=")
        assertEquals(null, state.history.first().angleMode)

        assertFalse(state.showScientific)
        state.showScientificPage(true)
        assertTrue(state.showScientific)
        state.press("1")
        assertTrue(state.showScientific)
    }

    @Test
    fun errorCanBeClearedByAFunction() {
        val state = CalculatorState()
        state.press("1")
        state.press("÷")
        state.press("0")
        state.press("=")
        assertTrue(state.isError)
        state.applyScientific(ScientificFunction.Sin)
        assertEquals("sin(", state.expression)
        assertFalse(state.isError)
    }

    @Test
    fun incompleteFunctionGroupClosesBeforePostfix() {
        val state = CalculatorState()
        state.showScientificPage(true)
        state.applyScientific(ScientificFunction.Sin)
        state.press("3")
        state.press("0")
        assertEquals("sin(30", state.expression)

        state.applyScientific(ScientificFunction.Square)
        assertEquals("sin(30)^2", state.expression)
        state.press("=")
        assertEquals("0.25", state.displayText)
    }

    @Test
    fun extremeHistoryResultsReloadAsExpressions() {
        val state = CalculatorState()
        state.press("1")
        state.press("0")
        state.applyScientific(ScientificFunction.Power)
        state.press("2")
        state.press("0")
        state.press("=")
        assertEquals("1e20", state.displayText)
        val entry = state.history.first()
        assertTrue(entry.result.contains("e"))
        assertTrue(entry.value != null)

        state.clear()
        state.loadHistory(entry)
        assertEquals("1×10^20", state.expression)
        state.press("=")
        assertEquals("1e20", state.displayText)
    }

    @Test
    fun decimalAfterConstantIsIgnored() {
        val state = CalculatorState()
        state.press("2")
        state.applyScientific(ScientificFunction.Pi)
        assertEquals("2π", state.expression)
        state.press(".")
        assertEquals("2π", state.expression)
    }

    @Test
    fun percentageSemanticsArePostfixDivision() {
        assertEquals(0.5, value("50%"), 1e-12)
        assertEquals(200.1, value("200+10%"), 1e-9)
    }
}
