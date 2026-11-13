package com.qalsheet.algebra

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.qalsheet.calc.CalculatorEngine
import java.math.BigDecimal
import java.math.RoundingMode
import kotlin.math.abs
import kotlin.math.hypot
import kotlin.math.round
import kotlin.math.sqrt

sealed interface AlgebraResult {
    data class Solved(val variable: Char, val value: Double) : AlgebraResult
    data class SolvedRoots(val variable: Char, val values: List<Double>) : AlgebraResult
    data class Expression(val variable: Char, val expression: String) : AlgebraResult
    data object NoSolution : AlgebraResult
    data object NoRealSolution : AlgebraResult
    data object InfiniteSolutions : AlgebraResult
    data class Error(val message: String) : AlgebraResult
}

/**
 * Polynomial-equation solver for the Algebra tab.
 *
 * The parser expands integer powers into a multivariate polynomial and keeps
 * rational denominators as constraints. Linear equations with several letters
 * return a symbolic expression for the selected letter. A single-letter
 * equation is solved exactly up to degree two; higher degrees use a numeric
 * real-root finder. Complex roots are reported as "no real solution".
 */
object AlgebraEngine {

    private const val EPS = 1e-10
    private const val MAX_POWER = 12
    private const val MAX_SOLVED_DEGREE = 12

    private val SUPERSCRIPT_DIGITS = mapOf(
        '⁰' to '0',
        '¹' to '1',
        '²' to '2',
        '³' to '3',
        '⁴' to '4',
        '⁵' to '5',
        '⁶' to '6',
        '⁷' to '7',
        '⁸' to '8',
        '⁹' to '9',
    )

    fun solve(raw: String, variable: Char = 'x'): AlgebraResult {
        val equation = normalizeSuperscripts(raw.trim())
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
            solveDifference(left - right, variable)
        } catch (error: AlgebraException) {
            AlgebraResult.Error(error.message ?: "Check the expression")
        } catch (_: Exception) {
            AlgebraResult.Error("Check the expression")
        }
    }

    private fun solveDifference(difference: Rational, selected: Char): AlgebraResult {
        val numerator = difference.numerator
        val denominator = difference.denominator
        val constraints = difference.constraints

        if (numerator.isZero) return AlgebraResult.InfiniteSolutions
        if (numerator.isConstant) return AlgebraResult.NoSolution

        val letters = numerator.letters
        if (letters.size > 1) {
            if (numerator.degree > 1) {
                return AlgebraResult.Error("Use one letter with powers")
            }
            return solveLinearWithLetters(numerator, denominator, constraints, selected)
        }

        val letter = letters.single()
        val coefficients = trim(numerator.coefficientsFor(letter))
        return when (val degree = coefficients.lastIndex) {
            0 -> AlgebraResult.NoSolution
            1 -> {
                val root = -coefficients[0] / coefficients[1]
                if (!denominatorAllows(denominator, constraints, letter, root)) {
                    AlgebraResult.NoSolution
                } else {
                    AlgebraResult.Solved(letter, clean(root))
                }
            }
            2 -> {
                val roots = quadraticRoots(coefficients)
                    ?: return AlgebraResult.NoRealSolution
                finishRoots(roots, letter, denominator, constraints, groupNearby = false)
            }
            in 3..MAX_SOLVED_DEGREE -> {
                val roots = realRoots(coefficients)
                if (roots.isEmpty()) return AlgebraResult.NoRealSolution
                finishRoots(roots, letter, denominator, constraints)
            }
            else -> AlgebraResult.Error("Power too high")
        }
    }

    private fun solveLinearWithLetters(
        numerator: Poly,
        denominator: Poly,
        constraints: List<Poly>,
        selected: Char,
    ): AlgebraResult {
        val constant = clean(numerator.constant)
        val terms = numerator.terms
            .filterKeys { it.length == 1 }
            .mapKeys { it.key[0] }
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
            terms.containsKey(selected) -> selected
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
            if (!denominatorAllows(denominator, constraints, solveFor, solvedConstant)) {
                return AlgebraResult.NoSolution
            }
            return AlgebraResult.Solved(solveFor, clean(solvedConstant))
        }
        return AlgebraResult.Expression(
            variable = solveFor,
            expression = formatExpression(solvedConstant, others),
        )
    }

    private fun finishRoots(
        roots: List<Double>,
        letter: Char,
        denominator: Poly,
        constraints: List<Poly>,
        groupNearby: Boolean = true,
    ): AlgebraResult {
        val validRoots = roots.filter {
            denominatorAllows(denominator, constraints, letter, it)
        }
        if (validRoots.isEmpty()) return AlgebraResult.NoSolution
        val values = if (groupNearby) {
            distinctRoots(validRoots)
        } else {
            validRoots.sorted().distinctBy { formatSolution(it) }
        }
        return if (values.size == 1) {
            AlgebraResult.Solved(letter, values.single())
        } else {
            AlgebraResult.SolvedRoots(letter, values)
        }
    }

    private fun quadraticRoots(coefficients: DoubleArray): List<Double>? {
        val a = coefficients[2]
        val b = coefficients[1]
        val c = coefficients[0]
        val discriminant = b * b - 4 * a * c
        if (discriminant < -EPS) return null
        if (abs(discriminant) <= EPS) {
            return listOf(clean(-b / (2 * a)))
        }
        val root = sqrt(discriminant)
        return listOf(
            clean((-b - root) / (2 * a)),
            clean((-b + root) / (2 * a)),
        ).sorted()
    }

    /**
     * Durand–Kerner root finder for the real roots of a degree >= 3
     * polynomial. Textbooks use small degrees; this keeps the solver useful
     * for x³ and higher without pretending to produce exact symbolic forms.
     */
    private fun realRoots(coefficients: DoubleArray): List<Double> {
        val degree = coefficients.lastIndex
        val leading = coefficients[degree]
        if (abs(leading) < EPS) return emptyList()
        val monic = DoubleArray(coefficients.size) { coefficients[it] / leading }
        val roots = Array(degree) { index ->
            var seed = Complex.One
            repeat(index) { seed *= Complex(0.4, 0.9) }
            seed
        }
        for (iteration in 0 until 240) {
            var largestDelta = 0.0
            for (i in roots.indices) {
                var denominator = Complex.One
                for (j in roots.indices) {
                    if (i != j) denominator *= roots[i] - roots[j]
                }
                if (denominator.abs() < 1e-12) {
                    denominator = Complex(1e-12, 0.0)
                }
                val delta = evaluateComplex(monic, roots[i]) / denominator
                roots[i] -= delta
                largestDelta = maxOf(largestDelta, delta.abs())
            }
            if (largestDelta < 1e-12) break
        }
        return roots
            .filter { abs(it.im) <= 1e-5 * maxOf(1.0, abs(it.re)) }
            .map { clean(it.re) }
            .sorted()
    }

    private fun evaluateComplex(coefficients: DoubleArray, value: Complex): Complex {
        var result = Complex(coefficients.last(), 0.0)
        for (index in coefficients.lastIndex - 1 downTo 0) {
            result = result * value + Complex(coefficients[index], 0.0)
        }
        return result
    }

    private fun trim(coefficients: DoubleArray): DoubleArray {
        var degree = coefficients.lastIndex
        while (degree > 0 && abs(coefficients[degree]) < EPS) degree--
        return coefficients.copyOf(degree + 1)
    }

    private fun distinctRoots(values: List<Double>): List<Double> {
        val groups = mutableListOf<MutableList<Double>>()
        values.sorted().forEach { value ->
            val group = groups.lastOrNull()
            val close = group != null &&
                abs(group.average() - value) <= 1e-4 * maxOf(1.0, abs(value))
            if (close) {
                group += value
            } else {
                groups += mutableListOf(value)
            }
        }
        return groups.map { group ->
            val average = group.average()
            if (abs(average) < 1e-6) 0.0 else clean(average)
        }
    }

    private fun normalizeSuperscripts(text: String): String {
        if (text.none { it in SUPERSCRIPT_DIGITS }) return text
        val result = StringBuilder(text.length + 4)
        var index = 0
        while (index < text.length) {
            val char = text[index]
            if (char !in SUPERSCRIPT_DIGITS) {
                result.append(char)
                index++
                continue
            }
            if (result.lastOrNull() != '^') result.append('^')
            while (index < text.length) {
                val digit = SUPERSCRIPT_DIGITS[text[index]] ?: break
                result.append(digit)
                index++
            }
        }
        return result.toString()
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

    /**
     * A multivariate polynomial. Monomials are stored as sorted lowercase
     * letters ("" is the constant term), so `x·y` and `x²` can be detected as
     * nonlinear without losing information needed for cancellation.
     */
    private data class Poly(val terms: Map<String, Double> = emptyMap()) {
        val isZero: Boolean get() = terms.isEmpty()
        val isConstant: Boolean get() = terms.keys.all { it.isEmpty() }
        val degree: Int get() = terms.keys.maxOfOrNull { it.length } ?: 0
        val constant: Double get() = terms[""] ?: 0.0
        val letters: Set<Char>
            get() = terms.keys.flatMap { it.toList() }.toSet()

        fun coefficientsFor(letter: Char): DoubleArray {
            val coefficients = DoubleArray(degree + 1)
            terms.forEach { (monomial, coefficient) ->
                var power = 0
                for (char in monomial) {
                    if (char != letter) return DoubleArray(0)
                    power++
                }
                coefficients[power] += coefficient
            }
            return coefficients
        }

        fun pow(exponent: Int): Poly {
            if (exponent < 0) throw AlgebraException("Use positive powers")
            var result = One
            var base = this
            var remaining = exponent
            while (remaining > 0) {
                if (remaining and 1 == 1) result *= base
                remaining = remaining shr 1
                if (remaining > 0) base *= base
            }
            return result
        }

        operator fun plus(other: Poly): Poly {
            if (other.isZero) return this
            if (isZero) return other
            val merged = terms.toMutableMap()
            other.terms.forEach { (monomial, coefficient) ->
                merged[monomial] = (merged[monomial] ?: 0.0) + coefficient
            }
            return Poly(merged.filterValues { abs(it) > EPS })
        }

        operator fun minus(other: Poly): Poly = plus(other.scale(-1.0))

        operator fun times(other: Poly): Poly {
            if (isZero || other.isZero) return Zero
            val product = mutableMapOf<String, Double>()
            terms.forEach { (left, leftCoefficient) ->
                other.terms.forEach { (right, rightCoefficient) ->
                    val monomial = (left + right).toCharArray().sorted().joinToString("")
                    product[monomial] =
                        (product[monomial] ?: 0.0) + leftCoefficient * rightCoefficient
                }
            }
            return Poly(product.filterValues { abs(it) > EPS })
        }

        fun scale(factor: Double): Poly {
            if (factor == 0.0 || isZero) return Zero
            return Poly(terms.mapValues { it.value * factor }.filterValues { abs(it) > EPS })
        }

        fun evaluate(values: Map<Char, Double>): Double =
            terms.entries.sumOf { (monomial, coefficient) ->
                var result = coefficient
                for (letter in monomial) {
                    result *= values[letter] ?: 0.0
                }
                result
            }

        companion object {
            val Zero = Poly()
            val One = Poly(mapOf("" to 1.0))

            fun constant(value: Double): Poly =
                if (abs(value) < EPS) Zero else Poly(mapOf("" to value))

            fun variable(letter: Char): Poly = Poly(mapOf(letter.toString() to 1.0))
        }
    }

    /**
     * A quotient of two polynomials, used while parsing divisions. Every
     * denominator encountered on the way is kept as a nonzero constraint so
     * nested fractions cannot introduce a division by zero.
     */
    private data class Rational(
        val numerator: Poly,
        val denominator: Poly,
        val constraints: List<Poly> = emptyList(),
    ) {
        operator fun plus(other: Rational): Rational = Rational(
            numerator = numerator * other.denominator + other.numerator * denominator,
            denominator = denominator * other.denominator,
            constraints = constraints + other.constraints + listOf(denominator, other.denominator),
        )

        operator fun minus(other: Rational): Rational = plus(other.negate())

        operator fun times(other: Rational): Rational = Rational(
            numerator = numerator * other.numerator,
            denominator = denominator * other.denominator,
            constraints = constraints + other.constraints + listOf(denominator, other.denominator),
        )

        operator fun div(other: Rational): Rational {
            if (other.numerator.isZero) throw AlgebraException("Division by zero")
            return Rational(
                numerator = numerator * other.denominator,
                denominator = denominator * other.numerator,
                constraints = constraints + other.constraints +
                    listOf(denominator, other.denominator, other.numerator),
            )
        }

        fun negate(): Rational =
            Rational(numerator.scale(-1.0), denominator, constraints)

        fun pow(exponent: Int): Rational {
            if (exponent < 0) throw AlgebraException("Use positive powers")
            return Rational(
                numerator = numerator.pow(exponent),
                denominator = denominator.pow(exponent),
                constraints = constraints,
            )
        }

        companion object {
            fun of(value: Poly): Rational = Rational(value, Poly.One)

            fun constant(value: Double): Rational =
                Rational(Poly.constant(value), Poly.One)
        }
    }

    private data class Complex(val re: Double, val im: Double) {
        operator fun plus(other: Complex): Complex =
            Complex(re + other.re, im + other.im)

        operator fun minus(other: Complex): Complex =
            Complex(re - other.re, im - other.im)

        operator fun times(other: Complex): Complex = Complex(
            re = re * other.re - im * other.im,
            im = re * other.im + im * other.re,
        )

        operator fun div(other: Complex): Complex {
            val denominator = other.re * other.re + other.im * other.im
            return Complex(
                re = (re * other.re + im * other.im) / denominator,
                im = (im * other.re - re * other.im) / denominator,
            )
        }

        fun abs(): Double = hypot(re, im)

        companion object {
            val One = Complex(1.0, 0.0)
        }
    }

    /**
     * A rational equation is valid only where every denominator is nonzero.
     * When the solution is numeric and a constraint depends on that letter
     * alone, reject an extraneous root that would divide by zero.
     */
    private fun denominatorAllows(
        denominator: Poly,
        constraints: List<Poly>,
        variable: Char,
        value: Double,
    ): Boolean = (constraints + denominator).all { constraint ->
        if (constraint.isZero) return@all false
        if (constraint.isConstant) return@all abs(constraint.constant) > EPS
        val letters = constraint.terms.keys.flatMap { it.toList() }.toSet()
        if (!letters.all { it == variable }) return@all true
        abs(constraint.evaluate(mapOf(variable to value))) > 1e-7
    }

    private class AlgebraException(message: String) : Exception(message)

    private class AlgebraParser(private val source: String) {
        private var position = 0

        fun parse(): Rational {
            val value = expression()
            skipSpaces()
            if (position != source.length) throw AlgebraException("Check the expression")
            return value
        }

        private fun expression(): Rational {
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

        private fun term(): Rational {
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

        private fun factor(): Rational {
            skipSpaces()
            when (peek()) {
                '+' -> {
                    position++
                    return factor()
                }
                '−', '-' -> {
                    position++
                    return factor().negate()
                }
            }
            return power()
        }

        private fun power(): Rational {
            var value = primary()
            while (true) {
                skipSpaces()
                if (peek() != '^') return value
                position++
                val exponent = factor()
                value = value.pow(wholeExponent(exponent))
            }
        }

        private fun wholeExponent(value: Rational): Int {
            if (!value.numerator.isConstant || !value.denominator.isConstant) {
                throw AlgebraException("Use whole-number powers")
            }
            val denominator = value.denominator.constant
            if (abs(denominator) < EPS) throw AlgebraException("Division by zero")
            val number = value.numerator.constant / denominator
            if (!number.isFinite()) throw AlgebraException("Check the power")
            val rounded = round(number)
            if (abs(number - rounded) > 1e-9) {
                throw AlgebraException("Use whole-number powers")
            }
            if (rounded < 0) throw AlgebraException("Use positive powers")
            if (rounded > MAX_POWER) throw AlgebraException("Power too high")
            return rounded.toInt()
        }

        private fun primary(): Rational {
            skipSpaces()
            val open = peek()
            if (open == '(' || open == '[' || open == '{') {
                val close = when (open) {
                    '(' -> ')'
                    '[' -> ']'
                    else -> '}'
                }
                position++
                val value = expression()
                skipSpaces()
                if (peek() != close) throw AlgebraException("Missing $close")
                position++
                return value
            }
            if (peek().isLetter()) {
                val letter = peek().lowercaseChar()
                position++
                return Rational.of(Poly.variable(letter))
            }

            val start = position
            while (position < source.length) {
                val char = source[position]
                if (char.isDigit() || char == '.' || char == ',') {
                    position++
                } else {
                    break
                }
            }
            if (start == position) throw AlgebraException("Check the expression")
            val token = source.substring(start, position)
            if (token.count { it == '.' || it == ',' } > 1) {
                throw AlgebraException("Check the expression")
            }
            val number = token.replace(',', '.')
                .toDoubleOrNull()
                ?: throw AlgebraException("Check the expression")
            return Rational.constant(number)
        }

        private fun startsImplicitFactor(char: Char): Boolean =
            char.isDigit() || char == '.' || char.isLetter() ||
                char == '(' || char == '[' || char == '{'

        private fun peek(): Char = if (position < source.length) source[position] else '\u0000'

        private fun skipSpaces() {
            while (position < source.length && source[position].isWhitespace()) position++
        }
    }
}

private val OPEN_GROUPS = charArrayOf('(', '[', '{')

private fun matchingClose(open: Char): Char = when (open) {
    '(' -> ')'
    '[' -> ']'
    else -> '}'
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
            is AlgebraResult.Solved ->
                "${current.variable} = ${AlgebraEngine.formatSolution(current.value)}"
            is AlgebraResult.SolvedRoots ->
                "${current.variable} = ${current.values.joinToString(", ") { AlgebraEngine.formatSolution(it) }}"
            is AlgebraResult.Expression -> "${current.variable} = ${current.expression}"
            AlgebraResult.NoSolution -> "No solution"
            AlgebraResult.NoRealSolution -> "No real solution"
            AlgebraResult.InfiniteSolutions -> "Infinite solutions"
            is AlgebraResult.Error -> "Check equation"
            null -> equation.ifEmpty { "0" }
        }

    val hintText: String
        get() = when (val current = result) {
            is AlgebraResult.Solved,
            is AlgebraResult.SolvedRoots,
            is AlgebraResult.Expression,
            AlgebraResult.NoSolution,
            AlgebraResult.NoRealSolution,
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
        get() = result is AlgebraResult.Solved ||
            result is AlgebraResult.SolvedRoots ||
            result is AlgebraResult.Expression
    val isWarning: Boolean
        get() = result == AlgebraResult.NoSolution ||
            result == AlgebraResult.NoRealSolution ||
            result == AlgebraResult.InfiniteSolutions

    /**
     * Nested groups follow textbook notation: `(` at the top level, then `[`
     * and `{`. The grouping key labels update to show what the next tap does.
     */
    val nextOpenGroup: Char
        get() = OPEN_GROUPS[unmatchedOpenGroups().size % OPEN_GROUPS.size]

    val nextCloseGroup: Char
        get() = unmatchedOpenGroups().lastOrNull()?.let(::matchingClose) ?: ')'

    fun press(key: String) {
        result = null
        when (key) {
            "C" -> clear()
            "⌫" -> backspace()
            "=" -> if ('=' in equation) solve() else inputEquals()
            "+", "−", "×", "÷" -> inputOperator(key)
            "." -> inputDecimal()
            "(", ")" -> inputParen(key)
            "^" -> insertPower()
            else -> if (key.length == 1 && key[0].isLetter()) insertVariable() else inputDigit(key)
        }
    }

    fun insertPower() {
        result = null
        if (equation.length >= 80) return
        val last = equation.lastOrNull() ?: return
        if (last.isDigit() || last.isLetter() || last in ")]}") {
            equation += "^"
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
            is AlgebraResult.SolvedRoots -> variable = solved.variable
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
        if (last != null && !last.isDigit() && last != '.' && last !in "+−×÷(=[{") {
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
        if (last in "([{^") {
            if (operator == "−" && last != '^') equation += operator
            return
        }
        if (equation.length >= 80) return
        equation = when {
            last in "+×÷" -> equation.dropLast(1) + operator
            last == '−' -> {
                if (equation.length == 1 || equation[equation.length - 2] in "+−×÷(=[{") {
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
        if (last == '=' || last in "+−×÷([{.^") return
        if (equation.count { it == '=' } > 0) return
        equation += "="
    }

    private fun inputParen(paren: String) {
        if (equation.length >= 80) return
        if (paren == "(") {
            val opener = nextOpenGroup
            val last = equation.lastOrNull()
            equation = if (last != null && (last.isDigit() || last.isLetter() || last in ")]}")) {
                "$equation×$opener"
            } else {
                "$equation$opener"
            }
            return
        }

        val openers = unmatchedOpenGroups()
        if (openers.isEmpty()) return
        val last = equation.lastOrNull() ?: return
        if (last.isDigit() || last.isLetter() || last in ")]}") {
            equation += matchingClose(openers.last())
        }
    }

    private fun unmatchedOpenGroups(): List<Char> {
        val openers = ArrayDeque<Char>()
        for (char in equation) {
            when (char) {
                '(', '[', '{' -> openers.addLast(char)
                ')', ']', '}' -> if (openers.isNotEmpty()) openers.removeLast()
            }
        }
        return openers.toList()
    }

    private fun currentNumber(): String {
        val start = equation.indexOfLast { it in "=+−×÷([{" }
        return if (start == -1) equation else equation.substring(start + 1)
    }
}
