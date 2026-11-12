package com.qalsheet.algebra

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class AlgebraEngineTest {

    @Test
    fun solvesBasicLinearEquations() {
        assertEquals(AlgebraResult.Solved('x', 5.0), AlgebraEngine.solve("3x + 5 = 20"))
        assertEquals(AlgebraResult.Solved('x', 7.0), AlgebraEngine.solve("2x-4=10"))
        assertEquals(AlgebraResult.Solved('x', -1.0), AlgebraEngine.solve("5(x+2)=3x+8"))
        assertEquals(AlgebraResult.Solved('x', -3.0), AlgebraEngine.solve("2(x-3)=4x"))
        assertEquals(AlgebraResult.Solved('x', 8.0), AlgebraEngine.solve("x/2 + 3 = 7"))
        assertEquals(AlgebraResult.Solved('x', 5.0), AlgebraEngine.solve("0.5x=2.5"))
        assertEquals(AlgebraResult.Solved('x', -3.0), AlgebraEngine.solve("-3x=9"))
        assertEquals(AlgebraResult.Solved('x', 6.0), AlgebraEngine.solve("2x+3=4x-9"))
    }

    @Test
    fun supportsVariablesOtherThanX() {
        assertEquals(AlgebraResult.Solved('y', 5.0), AlgebraEngine.solve("3y + 5 = 20"))
        assertEquals(AlgebraResult.Solved('a', 2.0), AlgebraEngine.solve("2a+3a=10"))
        assertEquals(AlgebraResult.Solved('m', 4.0), AlgebraEngine.solve("3m+2=14"))
        // The parser finds the only variable even if the selected letter differs.
        assertEquals(AlgebraResult.Solved('z', 5.0), AlgebraEngine.solve("4z+5=25", variable = 'x'))
    }

    @Test
    fun solvesMultiLetterEquationsForTheSelectedLetter() {
        assertEquals(
            AlgebraResult.Expression('x', "6 − 1.5y"),
            AlgebraEngine.solve("2x + 3y = 12", variable = 'x'),
        )
        assertEquals(
            AlgebraResult.Expression('y', "4 − 0.666667x"),
            AlgebraEngine.solve("2x + 3y = 12", variable = 'y'),
        )
        assertEquals(
            AlgebraResult.Expression('x', "10 − y"),
            AlgebraEngine.solve("x + y = 10", variable = 'x'),
        )
        assertEquals(
            AlgebraResult.Expression('x', "y"),
            AlgebraEngine.solve("x - y = 0", variable = 'x'),
        )
        assertEquals(
            AlgebraResult.Expression('x', "1.5y"),
            AlgebraEngine.solve("2x = 3y", variable = 'x'),
        )
        assertEquals(
            AlgebraResult.Error("Choose a letter in the equation"),
            AlgebraEngine.solve("2x + 3y = 12", variable = 'q'),
        )
    }

    @Test
    fun handlesSpecialEquations() {
        assertEquals(AlgebraResult.InfiniteSolutions, AlgebraEngine.solve("x = x"))
        assertEquals(AlgebraResult.InfiniteSolutions, AlgebraEngine.solve("5 = 5"))
        assertEquals(AlgebraResult.NoSolution, AlgebraEngine.solve("x + 1 = x + 2"))
        assertEquals(AlgebraResult.NoSolution, AlgebraEngine.solve("5 = 6"))
        assertEquals(AlgebraResult.Solved('x', 5.0), AlgebraEngine.solve("x = 5"))
        assertEquals(AlgebraResult.Solved('x', 1.0), AlgebraEngine.solve("2 = x + 1"))
    }

    @Test
    fun rejectsInvalidOrNonlinearInput() {
        assertEquals(AlgebraResult.Error("Add an = sign"), AlgebraEngine.solve("3x+5"))
        assertEquals(AlgebraResult.Error("Use one = sign"), AlgebraEngine.solve("3x+5=20=30"))
        assertEquals(AlgebraResult.Error("Complete both sides"), AlgebraEngine.solve("3x+5="))
        assertEquals(AlgebraResult.Error("Only linear equations"), AlgebraEngine.solve("x*x=4"))
        assertEquals(AlgebraResult.Error("Only linear equations"), AlgebraEngine.solve("x(x+1)=2"))
        assertEquals(AlgebraResult.Error("No letters in denominator"), AlgebraEngine.solve("1/x=2"))
        assertEquals(AlgebraResult.Error("Division by zero"), AlgebraEngine.solve("x/0=2"))
        assertEquals(AlgebraResult.Error("Enter an equation"), AlgebraEngine.solve(""))
        assertEquals(AlgebraResult.Error("Check the expression"), AlgebraEngine.solve("x^2=4"))
    }

    @Test
    fun formatsSolutionsAndSymbolicExpressions() {
        assertEquals("5", AlgebraEngine.formatSolution(5.0))
        assertEquals("−2.5", AlgebraEngine.formatSolution(-2.5))
        assertEquals("0.333333", AlgebraEngine.formatSolution(1.0 / 3.0))
        assertEquals("0", AlgebraEngine.formatSolution(0.0))
        assertEquals(
            "6 − 1.5y",
            AlgebraEngine.formatExpression(6.0, mapOf('y' to -1.5)),
        )
        assertEquals("x", AlgebraEngine.formatExpression(0.0, mapOf('x' to 1.0)))
        assertEquals("10 − y", AlgebraEngine.formatExpression(10.0, mapOf('y' to -1.0)))
        assertEquals("−5", AlgebraEngine.formatExpression(-5.0, emptyMap()))
    }

    @Test
    fun stateTypesAndSolvesAnEquation() {
        val state = AlgebraState()
        "3x+5=20".forEach { char ->
            state.press(char.toString())
        }
        assertEquals("3x+5=20", state.equation)
        state.solve()
        assertEquals("x = 5", state.displayText)
        assertEquals("3x+5=20", state.hintText)
        assertTrue(state.isSolution)
    }

    @Test
    fun stateSupportsOtherVariablesAndSymbolicResults() {
        val state = AlgebraState()
        state.selectVariable('y')
        "3y+5=20".forEach { char ->
            state.press(char.toString())
        }
        state.solve()
        assertEquals("y = 5", state.displayText)

        state.clear()
        state.selectVariable('x')
        state.press("2")
        state.insertVariable()
        state.press("+")
        state.press("3")
        state.selectVariable('y')
        state.insertVariable()
        state.press("=")
        state.press("1")
        state.press("2")
        state.selectVariable('x')
        state.solve()
        assertEquals("x = 6 − 1.5y", state.displayText)
    }

    @Test
    fun editingAfterSolveClearsTheResult() {
        val state = AlgebraState()
        "3x+5=20".forEach { char ->
            state.press(char.toString())
        }
        state.solve()
        assertTrue(state.isSolution)
        state.press("1")
        assertEquals(null, state.result)
        assertEquals("3x+5=201", state.equation)
    }

    @Test
    fun secondEqualsPressSolves() {
        val state = AlgebraState()
        "3x+5=20".forEach { char ->
            state.press(char.toString())
        }
        state.press("=")
        assertEquals("x = 5", state.displayText)
    }
}
