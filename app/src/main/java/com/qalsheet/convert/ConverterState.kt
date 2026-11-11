package com.qalsheet.convert

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

enum class UnitTarget { From, To }

class ConverterState {
    var categoryIndex by mutableIntStateOf(0)
        private set
    var fromIndex by mutableIntStateOf(Units.categories[0].defaultFromIndex)
        private set
    var toIndex by mutableIntStateOf(Units.categories[0].defaultToIndex)
        private set
    var input by mutableStateOf("1")
        private set

    val category: UnitCategory get() = Units.categories[categoryIndex]
    val from: MeasureUnit get() = category.units[fromIndex.coerceIn(category.units.indices)]
    val to: MeasureUnit get() = category.units[toIndex.coerceIn(category.units.indices)]

    val amount: Double
        get() = input.replace('−', '-').toDoubleOrNull() ?: 0.0

    val result: Double
        get() = Units.convert(amount, from, to)

    val inputText: String
        get() = input.ifEmpty { "0" }

    fun selectCategory(index: Int) {
        if (index !in Units.categories.indices) return
        categoryIndex = index
        val selected = Units.categories[index]
        fromIndex = selected.defaultFromIndex
        toIndex = selected.defaultToIndex
    }

    fun selectUnit(target: UnitTarget, index: Int) {
        if (index !in category.units.indices) return
        when (target) {
            UnitTarget.From -> fromIndex = index
            UnitTarget.To -> toIndex = index
        }
    }

    fun swap() {
        val converted = result
        val oldFrom = fromIndex
        fromIndex = toIndex
        toIndex = oldFrom
        input = Units.formatEditable(converted)
    }

    fun appendDigit(digit: String) {
        if (digit.length != 1 || !digit[0].isDigit()) return
        when {
            input == "0" -> input = digit
            input == "−" -> input = "−$digit"
            input.length < 12 -> input += digit
        }
    }

    fun appendDecimal() {
        if (input.contains('.')) return
        input = when {
            input.isEmpty() -> "0."
            input == "−" -> "−0."
            else -> "$input."
        }
    }

    fun backspace() {
        input = input.dropLast(1)
        if (input.isEmpty() || input == "−") input = "0"
    }

    fun clearInput() {
        input = "0"
    }

    fun toggleSign() {
        input = when {
            input.startsWith("−") -> input.drop(1).ifEmpty { "0" }
            input == "0" || input.isEmpty() -> "−"
            else -> "−$input"
        }
    }
}
