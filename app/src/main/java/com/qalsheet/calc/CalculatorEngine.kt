package com.qalsheet.calc

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import java.math.BigDecimal
import java.math.MathContext
import java.math.RoundingMode
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.acos
import kotlin.math.asin
import kotlin.math.atan
import kotlin.math.cos
import kotlin.math.floor
import kotlin.math.ln
import kotlin.math.log10
import kotlin.math.pow
import kotlin.math.sin
import kotlin.math.sqrt
import kotlin.math.tan

enum class AngleMode(val label: String) {
    Degrees("DEG"),
    Radians("RAD");

    internal fun toRadians(value: Double): Double =
        if (this == Degrees) value * PI / 180.0 else value

    internal fun fromRadians(value: Double): Double =
        if (this == Degrees) value * 180.0 / PI else value
}

enum class ScientificFunction(val label: String) {
    Sin("sin"),
    Cos("cos"),
    Tan("tan"),
    Asin("asin"),
    Acos("acos"),
    Atan("atan"),
    Ln("ln"),
    Log("log"),
    Sqrt("√"),
    Square("x²"),
    Power("x^y"),
    Pi("π"),
    E("e"),
    OpenParen("("),
    CloseParen(")"),
    Factorial("!"),
    Percent("%"),
    Sign("±"),
    Reciprocal("1/x"),
}

/**
 * Dependency-free recursive-descent parser for the watch keypad.
 *
 * Supports + − × ÷, parentheses, decimals, unary signs, right-associative
 * powers, postfix factorial/percent, the constants π/e, implicit
 * multiplication (2π, 2(3+4), 2sin(30)) and the scientific functions exposed
 * by the fx page. Trigonometric input/output respects [AngleMode].
 */
object CalculatorEngine {

    fun evaluate(raw: String, angleMode: AngleMode = AngleMode.Degrees): Double? {
        return try {
            val parser = Parser(
                raw
                    .replace('×', '*')
                    .replace('÷', '/')
                    .replace('−', '-')
                    .replace('–', '-'),
                angleMode,
            )
            val value = parser.parse()
            if (value.isFinite()) value else null
        } catch (_: Exception) {
            null
        }
    }

    /**
     * Removes trailing incomplete operators/parentheses and closes unmatched
     * parentheses, returning an expression that can be evaluated. Returns null
     * when the input cannot be repaired (for example "12)" or "sin(").
     */
    fun evaluable(raw: String): String? {
        var expression = raw.trim()
        if (expression.isEmpty()) return null

        while (expression.isNotEmpty() && expression.last() in "+−×÷(^") {
            expression = expression.dropLast(1)
        }
        if (expression.isEmpty() || expression == ".") return null
        if (expression.all { it.isLetter() } && expression.lowercase() !in setOf("e", "pi")) {
            return null
        }

        var balance = 0
        for (char in expression) {
            when (char) {
                '(' -> balance++
                ')' -> {
                    balance--
                    if (balance < 0) return null
                }
            }
        }
        if (balance > 0) expression += ")".repeat(balance)

        return expression
    }

    fun format(value: Double): String = format(value, grouping = false)

    fun format(value: Double, grouping: Boolean): String {
        if (!value.isFinite()) return "Error"
        val absolute = abs(value)
        val body = if (absolute != 0.0 && (absolute >= 1_000_000_000_000.0 || absolute < 0.00000001)) {
            BigDecimal.valueOf(value)
                .round(MathContext(8, RoundingMode.HALF_UP))
                .stripTrailingZeros()
                .toString()
                .replace("E+", "e")
                .replace("E-", "e-")
        } else {
            BigDecimal.valueOf(value)
                .round(MathContext(12, RoundingMode.HALF_UP))
                .stripTrailingZeros()
                .toPlainString()
        }
        val withMinus = if (body.startsWith('-')) "−${body.substring(1)}" else body
        return if (grouping) groupThousands(withMinus) else withMinus
    }

    fun formatEditable(value: Double): String = format(value, grouping = false).replace('\u00A0', ' ')

    /** Formats a value so it can be fed back into the parser, including extremes. */
    fun formatAsExpression(value: Double): String {
        val formatted = format(value)
        if ('e' !in formatted) return formatted
        val parts = formatted.split('e', limit = 2)
        val mantissa = parts.first()
        val exponent = parts.getOrElse(1) { "0" }.replace('-', '−')
        return "${mantissa}×10^$exponent"
    }

    fun usesTrigonometry(expression: String): Boolean =
        expression.contains("sin(") || expression.contains("cos(") || expression.contains("tan(")

    private fun groupThousands(text: String): String {
        if (text.contains('e') || text.contains('E')) return text
        val negative = text.startsWith('−')
        val body = if (negative) text.drop(1) else text
        val parts = body.split('.')
        val integer = parts[0]
        if (integer.length <= 3) return text
        val grouped = integer.reversed().chunked(3).joinToString("\u00A0").reversed()
        val decimal = if (parts.size > 1) ".${parts[1]}" else ""
        return buildString {
            if (negative) append('−')
            append(grouped)
            append(decimal)
        }
    }

    private class Parser(
        private val source: String,
        private val angleMode: AngleMode,
    ) {
        private var position = 0

        fun parse(): Double {
            val value = expression()
            skipSpaces()
            if (position != source.length) {
                throw IllegalArgumentException("Unexpected input at $position")
            }
            if (!value.isFinite()) throw ArithmeticException("Overflow")
            return value
        }

        private fun expression(): Double {
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

        private fun term(): Double {
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
                        val divisor = factor()
                        if (divisor == 0.0) throw ArithmeticException("Division by zero")
                        value /= divisor
                    }
                    startsImplicitFactor(peek()) -> {
                        value *= factor()
                    }
                    else -> return value
                }
            }
        }

        private fun factor(): Double {
            skipSpaces()
            when (peek()) {
                '+' -> {
                    position++
                    return factor()
                }
                '−', '-' -> {
                    position++
                    return -factor()
                }
            }
            var value = postfix()
            skipSpaces()
            if (peek() == '^') {
                position++
                value = value.pow(factor())
            }
            return value
        }

        private fun postfix(): Double {
            var value = primary()
            while (true) {
                skipSpaces()
                when (peek()) {
                    '!' -> {
                        position++
                        value = factorial(value)
                    }
                    '%' -> {
                        position++
                        value /= 100.0
                    }
                    else -> return value
                }
            }
        }

        private fun primary(): Double {
            skipSpaces()
            if (peek() == '(') {
                position++
                val value = expression()
                skipSpaces()
                if (peek() != ')') throw IllegalArgumentException("Missing )")
                position++
                return value
            }
            if (peek() == 'π') {
                position++
                return PI
            }
            if (peek() == '√') {
                position++
                skipSpaces()
                val operand = primary()
                if (operand < 0.0) throw IllegalArgumentException("Square root domain")
                return sqrt(operand)
            }
            if (peek().isLetter()) {
                return namedValue()
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
            if (start == position) throw IllegalArgumentException("Number expected")
            val token = source.substring(start, position)
            if (token == ".") throw IllegalArgumentException("Number expected")
            return token.toDouble()
        }

        private fun namedValue(): Double {
            val nameStart = position
            while (position < source.length && source[position].isLetter()) {
                position++
            }
            var name = source.substring(nameStart, position).lowercase()
            if (name == "e") return Math.E
            if (name == "pi") return PI
            if (name == "log" && source.startsWith("10", position)) {
                position += 2
                name = "log10"
            }
            skipSpaces()
            if (peek() != '(') throw IllegalArgumentException("Function needs parentheses")
            position++
            val argument = expression()
            skipSpaces()
            if (peek() != ')') throw IllegalArgumentException("Missing )")
            position++
            return when (name) {
                "sin" -> cleanTrig(sin(angleMode.toRadians(argument)))
                "cos" -> cleanTrig(cos(angleMode.toRadians(argument)))
                "tan" -> {
                    val radians = angleMode.toRadians(argument)
                    if (abs(cos(radians)) < 1e-12) {
                        throw ArithmeticException("Tangent undefined")
                    }
                    cleanTrig(tan(radians))
                }
                "asin" -> {
                    if (argument < -1.0 || argument > 1.0) throw IllegalArgumentException("asin domain")
                    angleMode.fromRadians(asin(argument))
                }
                "acos" -> {
                    if (argument < -1.0 || argument > 1.0) throw IllegalArgumentException("acos domain")
                    angleMode.fromRadians(acos(argument))
                }
                "atan" -> angleMode.fromRadians(atan(argument))
                "ln" -> {
                    if (argument <= 0.0) throw IllegalArgumentException("ln domain")
                    ln(argument)
                }
                "log", "log10" -> {
                    if (argument <= 0.0) throw IllegalArgumentException("log domain")
                    log10(argument)
                }
                "sqrt" -> {
                    if (argument < 0.0) throw IllegalArgumentException("sqrt domain")
                    sqrt(argument)
                }
                else -> throw IllegalArgumentException("Unknown function $name")
            }
        }

        private fun factorial(value: Double): Double {
            if (value < 0.0 || value != floor(value) || value > 170.0) {
                throw IllegalArgumentException("Factorial domain")
            }
            var result = 1.0
            var index = 2
            val limit = value.toInt()
            while (index <= limit) {
                result *= index
                index++
            }
            if (!result.isFinite()) throw ArithmeticException("Factorial overflow")
            return result
        }

        private fun cleanTrig(value: Double): Double = if (abs(value) < 1e-12) 0.0 else value

        private fun startsImplicitFactor(char: Char): Boolean {
            return char.isDigit() || char == '.' || char == '(' || char == 'π' || char == '√' || char.isLetter()
        }

        private fun peek(): Char = if (position < source.length) source[position] else '\u0000'

        private fun skipSpaces() {
            while (position < source.length && source[position].isWhitespace()) position++
        }
    }
}

data class HistoryEntry(
    val expression: String,
    val result: String,
    val angleMode: AngleMode? = null,
    val value: Double? = null,
)

/** All calculator UI state. Kept outside the composable so it survives tab switches. */
class CalculatorState {

    var expression by mutableStateOf("")
        private set
    private var result by mutableStateOf<String?>(null)
    var justEvaluated by mutableStateOf(false)
        private set
    var showScientific by mutableStateOf(false)
        private set
    var angleMode by mutableStateOf(AngleMode.Degrees)
        private set

    private var lastValue: Double? = null

    val history = mutableStateListOf<HistoryEntry>()

    val displayText: String
        get() = result ?: expression.ifEmpty { "0" }

    val topText: String
        get() = when {
            justEvaluated -> expression
            expression.isEmpty() -> "qalsheet"
            preview != null -> "= $preview"
            else -> ""
        }

    val topIsPreview: Boolean
        get() = !justEvaluated && expression.isNotEmpty() && preview != null

    val isError: Boolean
        get() = result == "Error"

    val preview: String?
        get() {
            if (justEvaluated || expression.isEmpty()) return null
            if (expression.none { it in "+−×÷()^!%" }) return null
            val evaluable = CalculatorEngine.evaluable(expression) ?: return null
            val value = CalculatorEngine.evaluate(evaluable, angleMode) ?: return null
            return CalculatorEngine.format(value)
        }

    fun press(key: String) {
        when (key) {
            "C" -> clear()
            "⌫" -> backspace()
            "=" -> calculate()
            "+", "−", "×", "÷" -> inputOperator(key)
            "(" -> inputOpenParen()
            ")" -> inputCloseParen()
            "." -> inputDecimal()
            else -> inputDigit(key)
        }
    }

    fun clear() {
        expression = ""
        result = null
        justEvaluated = false
        lastValue = null
    }

    fun clearHistory() {
        history.clear()
    }

    fun loadHistory(entry: HistoryEntry) {
        if (entry.result == "Error") return
        entry.angleMode?.let { angleMode = it }
        expression = entry.value?.let { CalculatorEngine.formatAsExpression(it) } ?: entry.result
        result = null
        justEvaluated = false
        lastValue = null
        showScientific = false
    }

    fun showScientificPage(show: Boolean) {
        showScientific = show
    }

    fun toggleAngleMode() {
        angleMode = if (angleMode == AngleMode.Degrees) AngleMode.Radians else AngleMode.Degrees
    }

    fun applyScientific(function: ScientificFunction) {
        if (justEvaluated) {
            if (isError) {
                clear()
            } else {
                val continued = continueFromResult()
                if (!continued) {
                    clear()
                }
            }
        }
        closeIncompleteParens()
        when (function) {
            ScientificFunction.Sin -> insertFunction("sin")
            ScientificFunction.Cos -> insertFunction("cos")
            ScientificFunction.Tan -> insertFunction("tan")
            ScientificFunction.Asin -> insertFunction("asin")
            ScientificFunction.Acos -> insertFunction("acos")
            ScientificFunction.Atan -> insertFunction("atan")
            ScientificFunction.Ln -> insertFunction("ln")
            ScientificFunction.Log -> insertFunction("log")
            ScientificFunction.Sqrt -> insertFunction("√")
            ScientificFunction.Square -> if (hasOperand()) appendRaw("^2")
            ScientificFunction.Power -> if (hasOperand()) appendRaw("^")
            ScientificFunction.Reciprocal -> {
                if (hasOperand()) {
                    wrapLastOperand("1/(", ")")
                } else {
                    appendRaw("1/(")
                }
            }
            ScientificFunction.Factorial -> if (hasOperand()) appendRaw("!")
            ScientificFunction.Percent -> if (hasOperand()) appendRaw("%")
            ScientificFunction.Sign -> toggleSign()
            ScientificFunction.Pi -> appendRaw("π")
            ScientificFunction.E -> appendRaw("e")
            ScientificFunction.OpenParen -> inputOpenParen()
            ScientificFunction.CloseParen -> inputCloseParen()
        }
        showScientific = false
    }

    private fun continueFromResult(): Boolean {
        val value = lastValue ?: return false
        expression = CalculatorEngine.formatAsExpression(value)
        result = null
        justEvaluated = false
        lastValue = null
        return true
    }

    private fun insertFunction(name: String) {
        if (hasOperand()) {
            wrapLastOperand("$name(", ")")
        } else {
            appendRaw("$name(")
        }
    }

    private fun closeIncompleteParens() {
        val last = expression.lastOrNull() ?: return
        val complete = last.isDigit() || last == ')' || last == 'π' || last == 'e' ||
            last == '!' || last == '%'
        if (!complete) return
        var balance = 0
        for (char in expression) {
            when (char) {
                '(' -> balance++
                ')' -> balance--
            }
        }
        if (balance <= 0) return
        val closing = ")".repeat(balance)
        if (expression.length + closing.length <= 120) {
            expression += closing
        }
    }

    private fun wrapLastOperand(prefix: String, suffix: String) {
        val start = lastOperandStart()
        if (start < 0) return
        val before = expression.substring(0, start)
        val operand = expression.substring(start)
        val candidate = before + prefix + operand + suffix
        if (candidate.length <= 120) {
            expression = candidate
        }
    }

    private fun appendRaw(text: String) {
        if (expression.length + text.length <= 120) {
            expression += text
        }
    }

    private fun hasOperand(): Boolean = lastOperandStart() >= 0

    private fun lastOperandStart(): Int {
        if (expression.isEmpty()) return -1
        var end = expression.length
        while (end > 0 && (expression[end - 1] == '!' || expression[end - 1] == '%')) {
            end--
        }
        if (end == 0) return -1
        val last = expression[end - 1]
        if (last == ')') {
            var depth = 0
            var index = end - 1
            while (index >= 0) {
                when (expression[index]) {
                    ')' -> depth++
                    '(' -> {
                        depth--
                        if (depth == 0) break
                    }
                }
                index--
            }
            if (index < 0) return -1
            var start = index
            var nameIndex = index - 1
            while (nameIndex >= 0 && (expression[nameIndex].isLetter() || expression[nameIndex] == '√')) {
                nameIndex--
            }
            if (nameIndex < index - 1) start = nameIndex + 1
            return start
        }
        if (last.isDigit() || last == '.') {
            var index = end - 1
            while (index >= 0 && (expression[index].isDigit() || expression[index] == '.')) {
                index--
            }
            return index + 1
        }
        if (last == 'π' || last == 'e') return end - 1
        return -1
    }

    private fun inputDigit(digit: String) {
        if (digit.length != 1 || !digit[0].isDigit()) return
        if (justEvaluated) {
            clear()
        }
        if (expression.length >= 120) return
        val current = currentNumber()
        expression = when {
            current == "0" && digit != "0" -> expression.dropLast(1) + digit
            current == "0" && digit == "0" -> expression
            else -> expression + digit
        }
    }

    private fun inputDecimal() {
        if (justEvaluated) {
            expression = "0."
            result = null
            justEvaluated = false
            lastValue = null
            return
        }
        if (expression.length >= 120) return
        val last = expression.lastOrNull()
        if (last != null && !last.isDigit() && last != '.' && last !in "+−×÷(^") {
            return
        }
        val current = currentNumber()
        if (current.contains('.')) return
        expression = if (current.isEmpty()) expression + "0." else expression + "."
    }

    private fun inputOperator(operator: String) {
        if (justEvaluated) {
            if (isError) {
                clear()
                if (operator == "−") expression = "−"
                return
            }
            expression = (result ?: "0") + operator
            result = null
            justEvaluated = false
            lastValue = null
            return
        }
        if (expression.isEmpty()) {
            if (operator == "−") expression = "−"
            return
        }
        val last = expression.last()
        expression = when {
            last in "+×÷" -> expression.dropLast(1) + operator
            last == '^' -> expression.dropLast(1) + operator
            last == '−' -> {
                if (expression.length == 1) expression
                else if (expression.length >= 2 && expression[expression.length - 2] in "+×÷(^") expression
                else expression.dropLast(1) + operator
            }
            last == '(' || last == '/' -> if (operator == "−") expression + operator else expression
            expression.length >= 120 -> expression
            else -> expression + operator
        }
    }

    private fun inputOpenParen() {
        if (justEvaluated) clear()
        if (expression.length >= 120) return
        val last = expression.lastOrNull()
        expression = when {
            last == null -> "("
            last.isDigit() || last == ')' || last == '.' || last == 'π' || last == 'e' || last == '!' || last == '%' -> "$expression×("
            else -> "$expression("
        }
    }

    private fun inputCloseParen() {
        if (justEvaluated) return
        var balance = 0
        for (char in expression) {
            when (char) {
                '(' -> balance++
                ')' -> balance--
            }
        }
        if (balance <= 0) return
        val last = expression.lastOrNull()
        if (last == null || last in "+−×÷(^" || last == '.') return
        if (expression.length >= 120) return
        expression += ")"
    }

    private fun toggleSign() {
        expression = when {
            expression.isEmpty() -> "−"
            expression.startsWith("−") -> expression.drop(1)
            else -> "−$expression"
        }
    }

    private fun backspace() {
        if (justEvaluated || isError) {
            clear()
            return
        }
        if (expression.isNotEmpty()) {
            expression = expression.dropLast(1)
        }
    }

    private fun calculate() {
        val evaluable = CalculatorEngine.evaluable(expression) ?: run {
            result = "Error"
            justEvaluated = true
            lastValue = null
            return
        }
        val value = CalculatorEngine.evaluate(evaluable, angleMode) ?: run {
            result = "Error"
            justEvaluated = true
            lastValue = null
            return
        }
        val formatted = CalculatorEngine.format(value)
        expression = evaluable
        result = formatted
        justEvaluated = true
        lastValue = value

        val meaningful = evaluable.any { it in "+−×÷()^!%" } || formatted != evaluable
        if (meaningful) {
            val historyMode = if (CalculatorEngine.usesTrigonometry(evaluable)) angleMode else null
            val entry = HistoryEntry(evaluable, formatted, historyMode, value)
            if (history.firstOrNull() != entry) {
                history.add(0, entry)
                if (history.size > 40) history.removeAt(history.lastIndex)
            }
        }
    }

    private fun currentNumber(): String {
        val start = expression.indexOfLast { it in "+−×÷(^" }
        return if (start == -1) expression else expression.substring(start + 1)
    }
}
