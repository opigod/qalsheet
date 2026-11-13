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
    fun supportsSquareAndCurlyBracketGroups() {
        assertEquals(
            AlgebraResult.Solved('y', 2.0),
            AlgebraEngine.solve("4[y−3(2y−1)]=2[y−2(3y+2)]"),
        )
        assertEquals(
            AlgebraResult.InfiniteSolutions,
            AlgebraEngine.solve("3[(z+2)−4]−z+1=2z−5"),
        )
        assertEquals(
            AlgebraResult.Solved('x', 2.0),
            AlgebraEngine.solve("2{3x+1}=14"),
        )
        assertEquals(
            AlgebraResult.Error("Missing ]"),
            AlgebraEngine.solve("2[3x+1=14"),
        )
    }

    @Test
    fun solvesTextbookExerciseSheetExamples() {
        assertNumericSolution("4x−2=3x+10", 'x', "12")
        assertNumericSolution("2y+8+y−2=2y−3", 'y', "−9")
        assertNumericSolution("(x−2)/3=5", 'x', "17")
        assertNumericSolution("(2x−1)/5=3+(1−3x)/4", 'x', "3")
        assertNumericSolution("2−(2/3)(−2x+1)=−3(1+x)−3", 'x', "−1.692308")
        assertNumericSolution("(x−10)/2=2−(5x−2)/7", 'x', "6")
        assertNumericSolution("(4y+3)/3−1/3=1−(5(1−y))/6", 'y', "−1")
        assertNumericSolution("3x−2=(3/5)(5x−3)−(4x)/10", 'x', "0.5")
    }

    @Test
    fun solvesUnknownsInDenominatorsWhenTheResultIsLinear() {
        assertEquals(AlgebraResult.Solved('x', 7.5), AlgebraEngine.solve("5/x=2/3"))
        assertEquals(AlgebraResult.Solved('x', -0.5), AlgebraEngine.solve("1/(x+1)=2"))
        assertEquals(AlgebraResult.Solved('x', 3.0), AlgebraEngine.solve("(x+1)/(x-1)=2"))
        assertEquals(AlgebraResult.Solved('x', -0.5), AlgebraEngine.solve("1/x+1/(x+1)=0"))
        // x = 1 would divide by zero, so the rational equation has no solution.
        assertEquals(AlgebraResult.NoSolution, AlgebraEngine.solve("(x-1)/(x-1)=2"))
        assertEquals(AlgebraResult.NoSolution, AlgebraEngine.solve("5/x=0"))
        assertEquals(AlgebraResult.NoSolution, AlgebraEngine.solve("1/(2/x)=0"))
        assertEquals(AlgebraResult.Solved('x', 2.0), AlgebraEngine.solve("1/(2/x)=1"))
        assertEquals(
            AlgebraResult.NoRealSolution,
            AlgebraEngine.solve("x/(x+1)=x/(x+2)+1"),
        )
    }

    @Test
    fun supportsIntegerPowersAndQuadraticRoots() {
        assertEquals(
            AlgebraResult.SolvedRoots('x', listOf(-2.0, 2.0)),
            AlgebraEngine.solve("x^2=4"),
        )
        assertEquals(
            AlgebraResult.SolvedRoots('x', listOf(-3.0, 3.0)),
            AlgebraEngine.solve("x²=9"),
        )
        assertEquals(
            AlgebraResult.SolvedRoots('x', listOf(-2.5, 1.0)),
            AlgebraEngine.solve("2x^2+3x−5=0"),
        )
        assertEquals(AlgebraResult.Solved('x', 1.0), AlgebraEngine.solve("(x−1)^2=0"))
        assertEquals(AlgebraResult.NoRealSolution, AlgebraEngine.solve("x^2+1=0"))
        assertEquals(
            AlgebraResult.InfiniteSolutions,
            AlgebraEngine.solve("x^2+2x+1=(x+1)^2"),
        )
        assertEquals(AlgebraResult.Solved('x', 1.0), AlgebraEngine.solve("2^3x=8"))
        assertEquals(AlgebraResult.InfiniteSolutions, AlgebraEngine.solve("x^2*x^3=x^5"))
    }

    @Test
    fun supportsHigherPowersWithNumericRealRoots() {
        assertRoots("x^3−6x^2+11x−6=0", listOf("1", "2", "3"))
        assertRoots("x^3=x", listOf("−1", "0", "1"))
        assertRoots("x^4−5x^2+4=0", listOf("−2", "−1", "1", "2"))
        assertRoots("x^3=1", listOf("1"))
        assertRoots("x^12=1", listOf("−1", "1"))
        assertRoots("x^2=2", listOf("−1.414214", "1.414214"))
        // Repeated numeric roots are shown once.
        assertRoots("x^3=0", listOf("0"))
        assertRoots("x^3+3x^2+3x+1=0", listOf("−1"))
        assertRoots("(x−1)^3=0", listOf("1"))
        assertEquals(AlgebraResult.NoRealSolution, AlgebraEngine.solve("x^4+1=0"))
    }

    @Test
    fun rejectsUnsupportedPowers() {
        assertEquals(
            AlgebraResult.Error("Use whole-number powers"),
            AlgebraEngine.solve("x^y=2"),
        )
        assertEquals(
            AlgebraResult.Error("Use whole-number powers"),
            AlgebraEngine.solve("x^0.5=2"),
        )
        assertEquals(
            AlgebraResult.Error("Use positive powers"),
            AlgebraEngine.solve("x^-1=2"),
        )
        assertEquals(
            AlgebraResult.Error("Power too high"),
            AlgebraEngine.solve("x^13=1"),
        )
    }

    @Test
    fun acceptsDecimalCommas() {
        assertEquals(AlgebraResult.Solved('x', 2.5), AlgebraEngine.solve("x+7,25=9,75"))
        assertEquals(AlgebraResult.Solved('x', 2.0), AlgebraEngine.solve("1,5x=3"))
        assertEquals(
            AlgebraResult.Error("Check the expression"),
            AlgebraEngine.solve("1,2.3x=4"),
        )
    }

    @Test
    fun rejectsMalformedInputAndNonlinearMultiLetterEquations() {
        assertEquals(AlgebraResult.Error("Add an = sign"), AlgebraEngine.solve("3x+5"))
        assertEquals(AlgebraResult.Error("Use one = sign"), AlgebraEngine.solve("3x+5=20=30"))
        assertEquals(AlgebraResult.Error("Complete both sides"), AlgebraEngine.solve("3x+5="))
        assertEquals(
            AlgebraResult.Error("Use one letter with powers"),
            AlgebraEngine.solve("x*y=4"),
        )
        assertEquals(
            AlgebraResult.Error("Use one letter with powers"),
            AlgebraEngine.solve("x^2+y=4"),
        )
        assertEquals(AlgebraResult.Solved('x', 0.5), AlgebraEngine.solve("1/x=2"))
        assertEquals(AlgebraResult.Error("Division by zero"), AlgebraEngine.solve("x/0=2"))
        assertEquals(AlgebraResult.Error("Enter an equation"), AlgebraEngine.solve(""))
        assertEquals(AlgebraResult.Error("Check the expression"), AlgebraEngine.solve("x^^2=4"))
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
    fun stateUsesTextbookNestingAndSolvesNestedGroups() {
        val state = AlgebraState()
        "4".forEach { state.press(it.toString()) }
        state.press("(")
        assertEquals("4×(", state.equation)
        assertEquals('[', state.nextOpenGroup)
        state.insertVariable()
        state.press("−")
        state.press("3")
        state.press("(")
        assertEquals('[', state.equation.last())
        state.press("2")
        state.insertVariable()
        state.press("−")
        state.press("1")
        state.press(")")
        state.press(")")
        assertEquals("4×(x−3×[2x−1])", state.equation)
        assertEquals('(', state.nextOpenGroup)

        state.press("=")
        state.press("2")
        state.press("(")
        state.insertVariable()
        state.press("−")
        state.press("2")
        state.press("(")
        state.press("3")
        state.insertVariable()
        state.press("+")
        state.press("2")
        state.press(")")
        state.press(")")
        state.solve()
        assertEquals("4×(x−3×[2x−1])=2×(x−2×[3x+2])", state.equation)
        assertEquals("x = 2", state.displayText)
    }

    @Test
    fun stateInsertsPowersAndShowsMultipleRoots() {
        val state = AlgebraState()
        state.insertVariable()
        state.press("^")
        state.press("2")
        state.press("=")
        state.press("4")
        assertEquals("x^2=4", state.equation)
        state.solve()
        assertEquals("x = −2, 2", state.displayText)
    }

    @Test
    fun stateShowsWhenThereIsNoRealRoot() {
        val state = AlgebraState()
        "x^2+1=0".forEach { char ->
            state.press(char.toString())
        }
        state.solve()
        assertEquals("No real solution", state.displayText)
    }

    @Test
    fun stateSolvesAnUnknownInTheDenominator() {
        val state = AlgebraState()
        state.press("5")
        state.press("÷")
        state.insertVariable()
        state.press("=")
        state.press("2")
        state.press("÷")
        state.press("3")
        state.solve()
        assertEquals("5÷x=2÷3", state.equation)
        assertEquals("x = 7.5", state.displayText)
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

    private fun assertNumericSolution(equation: String, variable: Char, expected: String) {
        val solved = AlgebraEngine.solve(equation, variable) as? AlgebraResult.Solved
        assertTrue("Expected a numeric solution for $equation, got $solved", solved != null)
        assertEquals(expected, AlgebraEngine.formatSolution(solved!!.value))
    }

    private fun assertRoots(equation: String, expected: List<String>) {
        val result = AlgebraEngine.solve(equation)
        val values = when (result) {
            is AlgebraResult.Solved -> listOf(result.value)
            is AlgebraResult.SolvedRoots -> result.values
            else -> emptyList()
        }
        assertEquals(
            "Unexpected result for $equation: $result",
            expected,
            values.map { AlgebraEngine.formatSolution(it) },
        )
    }
}
