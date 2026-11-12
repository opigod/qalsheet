package com.qalsheet.algebra

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.qalsheet.calc.CalculatorEngine
import java.math.BigDecimal
import java.math.RoundingMode
import kotlin.math.abs

sealed interface AlgebraResult {
    data class Solved(val variable: Char, val value: Double) : AlgebraResult
    data class Expression(val variable: Char, val expression: String) : AlgebraResult
    data object NoSolution : AlgebraResult
    data object InfiniteSolutions : AlgebraResult
    data class Error(val message: String) : AlgebraResult
}

/**
 * Linear-equation solver for the Algebra tab.
 *
 * A linear polynomial in any number of letters is represented as a constant
 * plus a map of letter → coefficient. Multiplication is only allowed when at
 * most one factor contains letters, so x·y, x², etc. are rejected as nonlinear.
 * Equations with several letters are solved for the selected letter and return
 * a symbolic expression in the remaining letters.
 */
object AlgebraEngine {

    private const val EPS = 1e-10

    fun solve(raw: String, variable: Char = 'x'): AlgebraResult {
        val equation = raw.trim()
        if (equation.isEmpty()) return AlgebraResult.Error("Enter an equation")

        val parts = equation.split('=')
        if (parts.size == 1) return AlgebraResult.Error("Add an = sign")
        if (parts.size > 2) return AlgebraResult.Error("Use one = sign")
        if (parts[0].isBlank() || parts[1].isBlank()) {
            return AlgebraResult.Error("Complete both sides")
        }

        return try {
            val left = AlgebraParser(parts[0]).parse()
            val right = AlgebraParser(parts[1]).parse()
            val difference = left - right
            val constant = clean(difference.constant)
            val terms = difference.terms
                .mapValues { clean(it.value) }
                .filterValues { abs(it) > EPS }

            if (terms.isEmpty()) {
                return if (abs(constant) < EPS) {
                    AlgebraResult.InfiniteSolutions
                } else {
                    AlgebraResult.NoSolution
                }
            }

            val solveFor = when {
                terms.containsKey(variable) -> variable
                terms.size == 1 -> terms.keys.first()
                else -> return AlgebraResult.Error("Choose a letter in the equation")
            }
            val coefficient = terms.getValue(solveFor)
            val solvedConstant = -constant / coefficient
            val others = terms
                .filterKeys { it != solveFor }
                .mapValues { -it.value / coefficient }
                .filterValues { abs(it) > EPS }

            if (others.isEmpty()) {
                AlgebraResult.Solved(solveFor, clean(solvedConstant))
            } else {
                AlgebraResult.Expression(
                    variable = solveFor,
                    expression = formatExpression(solvedConstant, others),
                )
            }
        } catch (error: AlgebraException) {
            AlgebraResult.Error(error.message ?: "Check the expression")
        } catch (_: Exception) {
            AlgebraResult.Error("Check the expression")
        }
    }

    fun formatSolution(value: Double): String {
        if (!value.isFinite()) return "Error"
        if (abs(value) >= 1_000_000_000.0 || (abs(value) > 0.0 && abs(value) < 1e-6)) {
            return CalculatorEngine.format(value)
        }
        val rounded = BigDecimal.valueOf(value)
            .setScale(6, RoundingMode.HALF_UP)
            .stripTrailingZeros()
        val text = rounded.toPlainString()
        return if (text.startsWith('-')) "−${text.substring(1)}" else text
    }

    /**
     * Formats `constant + sum(coefficient · letter)` as a readable right-hand
     * side, e.g. "6 − 1.5y" or "10 − x + 0.5z".
     */
    fun formatExpression(constant: Double, terms: Map<Char, Double>): String {
        val pieces = mutableListOf<String>()
        if (abs(constant) > EPS) {
            pieces += if (constant < 0) {
                "−${formatSolution(abs(constant))}"
            } else {
                formatSolution(constant)
            }
        }
        terms.entries
            .filter { abs(it.value) > EPS }
            .sortedBy { it.key }
            .forEach { (letter, coefficient) ->
                val magnitude = abs(coefficient)
                val body = buildString {
                    if (abs(magnitude - 1.0) > EPS) append(formatSolution(magnitude))
                    append(letter)
                }
                when {
                    pieces.isEmpty() -> pieces += if (coefficient < 0) "−$body" else body
                    coefficient < 0 -> pieces += "− $body"
                    else -> pieces += "+ $body"
                }
            }
        return if (pieces.isEmpty()) "0" else pieces.joinToString(" ")
    }

    private fun clean(value: Double): Double = if (abs(value) < EPS) 0.0 else value

    private data class Linear(
        val terms: Map<Char, Double>,
        val constant: Double,
    ) {
        operator fun plus(other: Linear): Linear {
            val merged = terms.toMutableMap()
            other.terms.forEach { (letter, coefficient) ->
                merged[letter] = (merged[letter] ?: 0.0) + coefficient
            }
            return Linear(
                terms = merged.filterValues { abs(it) > EPS },
                constant = constant + other.constant,
            )
        }

        operator fun minus(other: Linear): Linear =
            plus(Linear(other.terms.mapValues { -it.value }, -other.constant))

        operator fun times(other: Linear): Linear {
            if (terms.isNotEmpty() && other.terms.isNotEmpty()) {
                throw AlgebraException("Only linear equations")
            }
            return when {
                terms.isEmpty() -> Linear(
                    terms = other.terms.mapValues { it.value * constant },
                    constant = constant * other.constant,
                )
                other.terms.isEmpty() -> Linear(
                    terms = terms.mapValues { it.value * other.constant },
                    constant = constant * other.constant,
                )
                else -> Linear(emptyMap(), constant * other.constant)
            }
        }

        operator fun div(other: Linear): Linear {
            if (other.terms.isNotEmpty()) throw AlgebraException("No letters in denominator")
            if (abs(other.constant) < EPS) throw AlgebraException("Division by zero")
            return Linear(
                terms = terms.mapValues { it.value / other.constant },
                constant = constant / other.constant,
            )
        }
    }

    private class AlgebraException(message: String) : Exception(message)

    private class AlgebraParser(private val source: String) {
        private var position = 0

        fun parse(): Linear {
            val value = expression()
            skipSpaces()
            if (position != source.length) throw AlgebraException("Check the expression")
            return value
        }

        private fun expression(): Linear {
            var value = term()
            while (true) {
                skipSpaces()
                when (peek()) {
                    '+' -> {
                        position++
                        value += term()
                    }
                    '−', '-' -> {
                        position++
                        value -= term()
                    }
                    else -> return value
                }
            }
        }

        private fun term(): Linear {
            var value = factor()
            while (true) {
                skipSpaces()
                when {
                    peek() == '×' || peek() == '*' -> {
                        position++
                        value *= factor()
                    }
                    peek() == '÷' || peek() == '/' -> {
                        position++
                        value /= factor()
                    }
                    startsImplicitFactor(peek()) -> {
                        value *= factor()
                    }
                    else -> return value
                }
            }
        }

        private fun factor(): Linear {
            skipSpaces()
            when (peek()) {
                '+' -> {
                    position++
                    return factor()
                }
                '−', '-' -> {
                    position++
                    val value = factor()
                    return Linear(value.terms.mapValues { -it.value }, -value.constant)
                }
            }
            return primary()
        }

        private fun primary(): Linear {
            skipSpaces()
            if (peek() == '(') {
                position++
                val value = expression()
                skipSpaces()
                if (peek() != ')') throw AlgebraException("Missing )")
                position++
                return value
            }
            if (peek().isLetter()) {
                val letter = peek().lowercaseChar()
                position++
                return Linear(mapOf(letter to 1.0), 0.0)
            }

            val start = position
            while (position < source.length) {
                val char = source[position]
                if (char.isDigit() || char == '.') {
                    position++
                } else {
                    break
                }
            }
            if (start == position) throw AlgebraException("Check the expression")
            val token = source.substring(start, position)
            val number = token.toDoubleOrNull() ?: throw AlgebraException("Check the expression")
            return Linear(emptyMap(), number)
        }

        private fun startsImplicitFactor(char: Char): Boolean =
            char.isDigit() || char == '.' || char.isLetter() || char == '('

        private fun peek(): Char = if (position < source.length) source[position] else '\u0000'

        private fun skipSpaces() {
            while (position < source.length && source[position].isWhitespace()) position++
        }
    }
}

/** UI state for the equation editor. Hoisted in QalSheetApp so it survives tab switches. */
class AlgebraState {
    var equation by mutableStateOf("")
        private set

    var result by mutableStateOf<AlgebraResult?>(null)
        private set

    var variable by mutableStateOf('x')
        private set

    val displayText: String
        get() = when (val current = result) {
            is AlgebraResult.Solved -> "${current.variable} = ${AlgebraEngine.formatSolution(current.value)}"
            is AlgebraResult.Expression -> "${current.variable} = ${current.expression}"
            AlgebraResult.NoSolution -> "No solution"
            AlgebraResult.InfiniteSolutions -> "Infinite solutions"
            is AlgebraResult.Error -> "Check equation"
            null -> equation.ifEmpty { "0" }
        }

    val hintText: String
        get() = when (val current = result) {
            is AlgebraResult.Solved,
            is AlgebraResult.Expression,
            AlgebraResult.NoSolution,
            AlgebraResult.InfiniteSolutions,
            -> equation
            is AlgebraResult.Error -> current.message
            null -> when {
                equation.isEmpty() -> "Type an equation"
                '=' in equation -> "Tap Solve"
                else -> "Use = then Solve"
            }
        }

    val isError: Boolean get() = result is AlgebraResult.Error
    val isSolution: Boolean
        get() = result is AlgebraResult.Solved || result is AlgebraResult.Expression
    val isWarning: Boolean
        get() = result == AlgebraResult.NoSolution || result == AlgebraResult.InfiniteSolutions

    fun press(key: String) {
        result = null
        when (key) {
            "C" -> clear()
            "⌫" -> backspace()
            "=" -> if ('=' in equation) solve() else inputEquals()
            "+", "−", "×", "÷" -> inputOperator(key)
            "." -> inputDecimal()
            "(", ")" -> inputParen(key)
            else -> if (key.length == 1 && key[0].isLetter()) insertVariable() else inputDigit(key)
        }
    }

    fun insertVariable() {
        result = null
        if (equation.length >= 80) return
        if (equation.lastOrNull() == '.') return
        equation += variable
    }

    fun selectVariable(letter: Char) {
        variable = letter.lowercaseChar()
        result = null
    }

    fun solve() {
        val solved = AlgebraEngine.solve(equation, variable)
        result = solved
        when (solved) {
            is AlgebraResult.Solved -> variable = solved.variable
            is AlgebraResult.Expression -> variable = solved.variable
            else -> Unit
        }
    }

    fun clear() {
        equation = ""
        result = null
    }

    private fun backspace() {
        if (equation.isNotEmpty()) {
            equation = equation.dropLast(1)
        }
        result = null
    }

    private fun inputDigit(digit: String) {
        if (digit.length != 1 || !digit[0].isDigit()) return
        if (equation.length >= 80) return
        val current = currentNumber()
        equation = when {
            current == "0" && digit != "0" -> equation.dropLast(1) + digit
            current == "0" && digit == "0" -> equation
            else -> equation + digit
        }
    }

    private fun inputDecimal() {
        if (equation.length >= 80) return
        val last = equation.lastOrNull()
        if (last != null && !last.isDigit() && last != '.' && last !in "+−×÷(=") {
            return
        }
        val current = currentNumber()
        if (current.contains('.')) return
        equation = if (current.isEmpty()) equation + "0." else equation + "."
    }

    private fun inputOperator(operator: String) {
        if (equation.isEmpty()) {
            if (operator == "−") equation = "−"
            return
        }
        val last = equation.last()
        if (last == '=') {
            if (operator == "−") equation += operator
            return
        }
        if (last == '(') {
            if (operator == "−") equation += operator
            return
        }
        if (equation.length >= 80) return
        equation = when {
            last in "+×÷" -> equation.dropLast(1) + operator
            last == '−' -> {
                if (equation.length == 1 || equation[equation.length - 2] in "+−×÷(=") {
                    equation
                } else {
                    equation.dropLast(1) + operator
                }
            }
            last == '.' -> equation
            else -> equation + operator
        }
    }

    private fun inputEquals() {
        if (equation.length >= 80) return
        val last = equation.lastOrNull() ?: return
        if (last == '=' || last in "+−×÷(.") return
        if (equation.count { it == '=' } > 0) return
        equation += "="
    }

    private fun inputParen(paren: String) {
        if (equation.length >= 80) return
        if (paren == "(") {
            val last = equation.lastOrNull()
            equation = if (last != null && (last.isDigit() || last.isLetter() || last == ')')) {
                "$equation×("
            } else {
                "$equation("
            }
            return
        }

        var balance = 0
        for (char in equation) {
            when (char) {
                '(' -> balance++
                ')' -> balance--
            }
        }
        if (balance <= 0) return
        val last = equation.lastOrNull() ?: return
        if (last.isDigit() || last.isLetter() || last == ')') {
            equation += ")"
        }
    }

    private fun currentNumber(): String {
        val start = equation.indexOfLast { it in "=+−×÷(" }
        return if (start == -1) equation else equation.substring(start + 1)
    }
}
